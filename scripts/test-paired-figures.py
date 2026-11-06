#!/usr/bin/env python3
"""Synthetic controls; set CARGO_TARGET_DIR to an absolute on-disk directory."""
import contextlib
import copy
import gzip
import importlib.util
import io
import json
import os
import pathlib
import sqlite3
import subprocess
import sys
import tempfile
import types
import unittest
from unittest import mock

SPEC = importlib.util.spec_from_file_location(
    'paired_figures', pathlib.Path(__file__).with_name('paired-figures.py'))
FIGURES = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(FIGURES)


def observation(test='case', index=0, debug='Reply { value: 100 }', lines=()):
    return {'version': 1, 'test': test, 'index': index, 'kind': 'Estimate',
            'debug': debug, 'money': {'top': {'tax': 100}, 'return_rows': [],
                                    'lines': [{'id': name, 'kind': 'Money', 'amount': value}
                                              for name, value in lines]}}


def federal(lines):
    return 'FederalResult { lines: [' + ', '.join(
        'CalculationLine { id: ' + json.dumps(name) + ', amount: ' + str(value) + ' }'
        for name, value in lines) + '] }'


class PairedFigureControls(unittest.TestCase):
    def setUp(self):
        artifact_root = pathlib.Path(os.environ['CARGO_TARGET_DIR'])
        if not artifact_root.is_absolute():
            raise ValueError('CARGO_TARGET_DIR must be absolute')
        artifact_root.mkdir(parents=True, exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(prefix='paired-controls-', dir=artifact_root)
        self.addCleanup(self.temp.cleanup)
        self.root = pathlib.Path(self.temp.name)
        self.run_number = 0

    def capture(self, name, rows):
        path = self.root / name
        path.write_text(''.join(json.dumps(row) + '\n' for row in rows))
        return str(path)

    def compare(self, old, new, *, shard=(0, 1), mapping=None, index=None, structure=None):
        self.run_number += 1
        output = str(self.root / f'result-{self.run_number}')
        line_map = None
        if mapping is not None:
            line_map = self.capture('mapping.json', [mapping])
        structure_map = self.capture('structure.json', [structure]) if structure else None
        args = types.SimpleNamespace(
            old=self.capture('old.jsonl', old), new=self.capture('new.jsonl', new),
            output=output, test_shard=shard, line_map=line_map, reasons=None, index=index,
            structure_map=structure_map)
        with contextlib.redirect_stdout(io.StringIO()):
            status = FIGURES.compare(args)
        with gzip.open(output + '.pairs.jsonl.gz', 'rt') as stream:
            pairs = [json.loads(line) for line in stream]
        self.assert_rust_compare(args, status)
        return status, pairs, json.loads(pathlib.Path(output + '.summary.json').read_text())

    def assert_rust_compare(self, args, status):
        binary = os.environ.get('TAX_FIGURE_COMPARATOR_BIN')
        # Existing SQLite indexes belong only to the retained Python fallback.
        if binary and args.index is None:
            rust_output = args.output + '-rust'
            command = [binary, 'compare', '--old', args.old, '--new', args.new,
                       '--output', rust_output, '--test-shard', *map(str, args.test_shard)]
            for flag, value in [('--line-map', args.line_map), ('--structure-map', args.structure_map)]:
                if value:
                    command += [flag, value]
            result = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            self.assertEqual(result.returncode, status, result.stderr.decode())
            self.assertEqual(pathlib.Path(rust_output + '.summary.json').read_bytes(),
                             pathlib.Path(args.output + '.summary.json').read_bytes())
            rust_pairs_path = next((rust_output + suffix for suffix in
                                    ('.pairs.jsonl.zst', '.pairs.jsonl.gz')
                                    if pathlib.Path(rust_output + suffix).is_file()), None)
            self.assertIsNotNone(rust_pairs_path, result.stderr.decode())
            with gzip.open(args.output + '.pairs.jsonl.gz', 'rt', encoding='utf-8') as python_pairs:
                with FIGURES.capture_stream(rust_pairs_path) as rust_pairs:
                    self.assertEqual(rust_pairs.read(), python_pairs.read())

    def test_capture_volume_never_selects_tests_and_capacity_exclusion_is_exact(self):
        names = ['ordinary', 'tests::health::hsa_capacity',
                 'tests::core_tests::unit53_capacity_other::ordinary']
        skipped = 'tests::core_tests::unit53_capacity::stress'
        rows = [observation(test=name) for name in names + [skipped]]
        send = dict(observation(test=skipped, index=1), kind='Send', debug='opaque @')
        old_only, new_only = (FIGURES.EXCLUDED_CAPACITY_PREFIX + suffix
                              for suffix in ('old_only', 'new_only'))
        old = rows + [send, {'test': old_only, 'kind': 'Estimate', 'debug': 'opaque @'}]
        new = rows + [send, {'version': 2, 'test': skipped, 'kind': 'Estimate', 'debug': None},
                      {'version': 2, 'test': new_only, 'kind': 'Send', 'debug': None}]
        status, pairs, report = self.compare(old, new)
        self.assertEqual(status, 0)
        self.assertEqual(set(report['tests_selected']), set(names))
        self.assertEqual(len(pairs), len(names))
        expected = {
            skipped: {'old_observations': 2, 'new_observations': 3,
                      'old_estimates': 1, 'new_estimates': 2},
            old_only: {'old_observations': 1, 'new_observations': 0,
                       'old_estimates': 1, 'new_estimates': 0},
            new_only: {'old_observations': 0, 'new_observations': 1,
                       'old_estimates': 0, 'new_estimates': 0}}
        self.assertEqual(report['excluded_capacity_tests'], expected)
        binary = os.environ.get('TAX_FIGURE_COMPARATOR_BIN')
        if binary:
            final = subprocess.run([binary, 'final', str(self.root / 'old.jsonl'),
                                    str(self.root / 'new.jsonl')],
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            self.assertEqual(final.returncode, 0, final.stderr.decode())
            self.assertEqual(json.loads(final.stdout)['excluded_capacity_tests'], expected)
        # Raw byte statistics remain diagnostic, including retained indexes.
        unicode_row = observation(test='é')
        path = self.root / 'utf8.jsonl'
        raw = json.dumps(unicode_row, ensure_ascii=False) + '\r\n'
        path.write_bytes(raw.encode('utf-8'))
        database = sqlite3.connect(':memory:')
        self.addCleanup(database.close)
        database.execute('CREATE TABLE events(side,test,ordinal,kind,data)')
        FIGURES.records(path, database, 'old')
        self.assertEqual(FIGURES.capture_byte_totals(database)[('old', 'é')],
                         len(raw.encode('utf-8')))
        database.execute('DROP TABLE capture_bytes')
        self.assertEqual(FIGURES.capture_byte_totals(database)[('old', 'é')],
                         len(raw.encode('utf-8')))

    def test_ordinary_capture_above_16mb_keeps_every_pair_and_final_money_change(self):
        # Many small observations cross the historical capture mask without
        # creating a large Debug document or retaining a large fixture in RAM.
        count = 128
        for side in ('old', 'new'):
            path = self.root / (side + '.jsonl')
            with path.open('w') as stream:
                for index in range(count):
                    row = observation(index=index)
                    row['padding'] = 'x' * 128_000
                    if side == 'new' and index == count - 1:
                        row['money']['top']['tax'] = 101
                    stream.write(json.dumps(row, separators=(',', ':')) + '\n')
            self.assertGreater(path.stat().st_size, 16_000_000)
        args = types.SimpleNamespace(
            old=str(self.root / 'old.jsonl'), new=str(self.root / 'new.jsonl'),
            output=str(self.root / 'large'), test_shard=(0, 1), line_map=None,
            reasons=None, index=None, structure_map=None)
        with contextlib.redirect_stdout(io.StringIO()):
            status = FIGURES.compare(args)
        self.assertEqual(status, 1)
        report = json.loads(pathlib.Path(args.output + '.summary.json').read_text())
        self.assertEqual(report['tests_selected'], {'case': {'old': count, 'new': count}})
        self.assertEqual(report['counts']['pairs_compared'], count)
        self.assertEqual(report['counts']['pairs_with_estimate_money_changes'], 1)
        self.assertNotIn('excluded_oversize_tests', report)
        self.assert_rust_compare(args, status)
        binary = os.environ.get('TAX_FIGURE_COMPARATOR_BIN')
        if binary:
            final = subprocess.run([binary, 'final', args.old, args.new],
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            self.assertEqual(final.returncode, 1, final.stderr.decode())
            result = json.loads(final.stdout)
            self.assertEqual(result['tests_compared'], 1)
            self.assertEqual(result['final_figures_differ'], 1)

    @unittest.skipUnless(os.environ.get('TAX_FIGURE_COMPARATOR_BIN'), 'Rust binary not supplied')
    def test_missing_capture_facts_and_invalid_line_targets_cannot_prove_equivalence(self):
        for case in ('debug', 'return_money', 'line_amount', 'map_target',
                     'map_root', 'map_null', 'reason_root'):
            with self.subTest(case=case):
                row = observation(lines=[('tax', 100)])
                mapping = {}
                if case == 'debug':
                    del row['debug']
                elif case == 'return_money':
                    row['money']['return_rows'] = [{'amount': 100}]
                elif case == 'line_amount':
                    del row['money']['lines'][0]['amount']
                elif case == 'map_target':
                    mapping = {'*': {'tax': 123}}
                elif case == 'map_root':
                    mapping = []
                elif case == 'map_null':
                    mapping = {'*': None}
                old = self.capture('old.jsonl', [row])
                new = self.capture('new.jsonl', [row])
                line_map = self.capture('map.json', [mapping])
                common = ['compare', '--old', old, '--new', new, '--line-map', line_map]
                if case == 'reason_root':
                    common += ['--reasons', self.capture('reasons.json', [[]])]
                commands = [[os.environ['TAX_FIGURE_COMPARATOR_BIN']],
                            [sys.executable, str(pathlib.Path(FIGURES.__file__))]]
                for index, command in enumerate(commands):
                    result = subprocess.run(command + common + ['--output', str(self.root / f'bad-{index}')],
                                            capture_output=True)
                    self.assertEqual(result.returncode, 1, result.stdout.decode())

    def test_single_cent_change_is_never_rounded_away(self):
        old = observation(lines=[('tax', 10001)])
        new = copy.deepcopy(old)
        new['money']['lines'][0]['amount'] = 10002
        status, pairs, summary = self.compare([old], [new])
        self.assertEqual(status, 1)
        self.assertEqual(pairs[0]['money_changes'], [
            {'path': 'line.tax#0', 'old': 10001, 'new': 10002,
             'old_present': True, 'new_present': True}])
        self.assertEqual(summary['counts']['unexplained_pairs'], 1)

    def test_compressed_capture_formats_preserve_the_complete_pair_verdict(self):
        old = [observation(index=0), observation(index=1, lines=[('tax', 10001)])]
        new = [observation(index=0), observation(index=1, lines=[('tax', 10002)])]
        expected_status, expected_pairs, expected_report = self.compare(old, new)
        for suffix in ['gz', 'zst']:
            with self.subTest(format=suffix):
                paths = []
                for side, rows in [('old', old), ('new', new)]:
                    path = self.root / f'{side}.jsonl.{suffix}'
                    payloads = [(json.dumps(row) + '\n').encode() for row in rows]
                    if suffix == 'gz':
                        path.write_bytes(gzip.compress(b''.join(payloads)))
                    else:
                        # The writer finishes one independent frame per event.
                        path.write_bytes(b''.join(subprocess.run(
                            ['zstd', '-q', '-c'], input=payload, stdout=subprocess.PIPE,
                            check=True).stdout for payload in payloads))
                    paths.append(str(path))
                output = str(self.root / f'result-{suffix}')
                args = types.SimpleNamespace(
                    old=paths[0], new=paths[1], output=output, index=None,
                    test_shard=(0, 1), line_map=None, reasons=None, structure_map=None)
                with contextlib.redirect_stdout(io.StringIO()):
                    status = FIGURES.compare(args)
                with gzip.open(output + '.pairs.jsonl.gz', 'rt') as stream:
                    pairs = [json.loads(line) for line in stream]
                self.assert_rust_compare(args, status)
                self.assertEqual((status, pairs), (expected_status, expected_pairs))
                self.assertEqual(json.loads(pathlib.Path(output + '.summary.json').read_text()),
                                 expected_report)

    def test_final_json_value_can_end_without_a_newline_or_on_a_frame_boundary(self):
        payload = json.dumps(observation()).encode()
        for format in ('raw', 'gzip', 'zstd', 'split-zstd'):
            with self.subTest(format=format):
                path = self.root / ('unterminated.jsonl' +
                                    ('.gz' if format == 'gzip' else '.zst' if 'zstd' in format else ''))
                data = payload
                if format == 'gzip':
                    data = gzip.compress(payload)
                elif 'zstd' in format:
                    chunks = [b'', payload[:len(payload)//2], payload[len(payload)//2:]] if format == 'split-zstd' else [payload]
                    data = b''.join(subprocess.run(['zstd', '-qc'], input=chunk,
                                   stdout=subprocess.PIPE, check=True).stdout for chunk in chunks)
                path.write_bytes(data)
                args = types.SimpleNamespace(old=str(path), new=str(path),
                    output=str(self.root / ('unterminated-' + format)), index=None,
                    test_shard=(0, 1), line_map=None, reasons=None, structure_map=None)
                with contextlib.redirect_stdout(io.StringIO()):
                    status = FIGURES.compare(args)
                self.assertEqual(status, 0)
                self.assert_rust_compare(args, status)

    def test_truncated_zstd_capture_is_an_error_not_a_partial_success(self):
        payload = (json.dumps(observation()) + '\n').encode()
        frame = subprocess.run(['zstd', '-q', '-c'], input=payload,
                               stdout=subprocess.PIPE, check=True).stdout
        path = self.root / 'truncated.jsonl.zst'
        path.write_bytes(frame[:-3])
        with self.assertRaisesRegex(ValueError, 'invalid zstd capture'):
            with FIGURES.capture_stream(path) as stream:
                stream.read()

    def test_currency_code_scale_and_minor_units_are_all_audited(self):
        template = 'Reply { cash: Currency { currency_code: %s, scale: %s, minor_units: %s } }'
        for replacement in [('EUR', 2, 10001), ('USD', 3, 10001), ('USD', 2, 10002)]:
            with self.subTest(replacement=replacement):
                old = observation(debug=template % ('USD', 2, 10001))
                new = observation(debug=template % replacement)
                status, pairs, _ = self.compare([old], [new])
                self.assertEqual(status, 1)
                changes = pairs[0]['currency_changes']
                self.assertEqual(len(changes), 1)
                self.assertEqual(changes[0]['old']['minor_units'], {'$number': '10001'})
                self.assertNotEqual(changes[0]['old'], changes[0]['new'])

    def test_nested_currency_projection_preserves_source_order_without_metadata(self):
        template = ('Reply { cash: Currency { currency_code: Code { z: "USD", a: 7 }, '
                    'scale: Some(Scale { z: 2, a: 0 }), '
                    'minor_units: Units([Part { z: %s, a: 1 }]) } }')
        for structure in (None, self.source_mapping()):
            with self.subTest(structure=structure):
                old = observation(debug=template % 10001)
                new = observation(debug=template % 10002)
                status, pairs, _ = self.compare([old], [new], structure=structure)
                self.assertEqual(status, 1)
                money = pairs[0]['currency_changes'][0]['old']
                self.assertEqual(list(money['currency_code']), ['$tag', '$fields'])
                self.assertEqual(list(money['currency_code']['$fields']), ['z', 'a'])
                self.assertNotIn('$field_order', json.dumps(pairs))

    def test_cache_byte_budget_never_omits_an_oversized_observation(self):
        with mock.patch.object(FIGURES, '_debug_views_cache', FIGURES.collections.OrderedDict()), \
             mock.patch.object(FIGURES, '_debug_views_cache_bytes', 0):
            first = FIGURES.debug_views('Reply { value: 100 }')
            self.assertIs(FIGURES.debug_views('Reply { value: 100 }'), first)
            self.assertLessEqual(FIGURES._debug_views_cache_bytes, FIGURES.DEBUG_CACHE_BYTES)
        with mock.patch.object(FIGURES, '_debug_views_cache', FIGURES.collections.OrderedDict()), \
             mock.patch.object(FIGURES, '_debug_views_cache_bytes', 0), \
             mock.patch.object(FIGURES, 'DEBUG_CACHE_BYTES', 1):
            first = FIGURES.debug_views('Reply { value: 100 }')
            second = FIGURES.debug_views('Reply { value: 101 }')
            self.assertEqual(first[0], {'$.$fields.value': '100'})
            self.assertEqual(second[0], {'$.$fields.value': '101'})
            self.assertFalse(FIGURES._debug_views_cache)
            self.assertEqual(FIGURES._debug_views_cache_bytes, 0)
        with mock.patch.object(FIGURES, '_debug_views_cache', FIGURES.collections.OrderedDict()), \
             mock.patch.object(FIGURES, '_debug_views_cache_bytes', 0), \
             mock.patch.object(FIGURES, 'DEBUG_CACHE_BYTES', 2000):
            for value in range(100):
                text = 'Reply { value: %s }' % value
                views = FIGURES.debug_views(text)
                self.assertEqual(views[0], {'$.$fields.value': str(value)})
                self.assertLessEqual(FIGURES._debug_views_cache_bytes, 2000)
            self.assertNotIn('Reply { value: 0 }', FIGURES._debug_views_cache)
            self.assertIn('Reply { value: 99 }', FIGURES._debug_views_cache)
            self.assertIs(FIGURES.debug_views('Reply { value: 99 }'), views)

    def test_large_debug_keeps_duplicate_line_audit_and_invalid_currency_failure(self):
        padding = 'x' * (1024 * 1024 + 1)
        old_debug = federal([('tax', 100), ('tax', 200)])
        old_debug = old_debug[:-2] + ', text: "' + padding + '" }'
        old = observation(debug=old_debug, lines=[('tax', 100), ('tax', 200)])
        new = observation(debug=old_debug.replace('amount: 200', 'amount: 201'),
                          lines=[('tax', 100), ('tax', 201)])
        status, pairs, _ = self.compare([old], [new])
        self.assertEqual(status, 1)
        self.assertEqual(len(pairs[0]['numeric_audit']), 1)
        self.assertEqual(pairs[0]['numeric_audit'][0]['old'], '200')
        self.assertEqual(pairs[0]['numeric_audit'][0]['new'], '201')
        invalid = 'Reply { text: "' + padding + '", cash: Currency { scale: 2 } }'
        with self.assertRaises(KeyError):
            self.compare([observation(debug=invalid)], [observation(debug=invalid + ' ')])
        binary = os.environ.get('TAX_FIGURE_COMPARATOR_BIN')
        if binary:
            old_path = self.capture('invalid-old.jsonl', [observation(debug=invalid)])
            new_path = self.capture('invalid-new.jsonl', [observation(debug=invalid + ' ')])
            result = subprocess.run([binary, 'compare', '--old', old_path, '--new', new_path,
                                     '--output', str(self.root / 'invalid-large')],
                                    stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            self.assertNotEqual(result.returncode, 0)

    def test_typed_money_preserves_large_integer_and_python_numeric_boundaries(self):
        cases = [(2**63 - 1, 2**63 - 2, True),
                 (-(2**63), -(2**63) + 1, True),
                 (2**53, 2**53 + 1, True),
                 (2**53 + 1, float(2**53), True),
                 (2**53, float(2**53), False),
                 (1, True, False), (0, False, False),
                 (1e-6, 2e-6, True), (1e16, 2e16, True),
                 (1e-5, 2e-5, True), (1e-4, 2e-4, True),
                 (-1e-5, -2e-5, True), (1.234e-5, 2.345e-5, True),
                 (1e15, 2e15, True), (1.0, 2.0, True),
                 (-0.0, 0.1, True)]
        for before, after, changed in cases:
            with self.subTest(before=before, after=after):
                old, new = observation(), observation()
                old['money']['top']['tax'] = before
                new['money']['top']['tax'] = after
                status, pairs, _ = self.compare([old], [new])
                self.assertEqual(status, int(changed))
                self.assertEqual(bool(pairs[0]['money_changes']), changed)

    def test_exact_line_ids_align_without_shifting_after_retirement(self):
        old = observation(debug=federal([('retired', 7), ('Tax', 100), ('tax', 200)]))
        new = observation(debug=federal([('tax', 200), ('Tax', 100)]))
        _, pairs, _ = self.compare([old], [new])
        changes = pairs[0]['numeric_audit']
        self.assertEqual(len(changes), 1)
        self.assertIn('retired', changes[0]['path'])
        self.assertEqual((changes[0]['old_present'], changes[0]['new_present']), (True, False))

    def test_duplicate_line_occurrences_preserve_order_and_each_amount(self):
        old_lines = [('tax', 100), ('tax', 200), ('tax', 300)]
        for new_lines in [[('tax', 101), ('tax', 199), ('tax', 300)],
                          [('tax', 200), ('tax', 100), ('tax', 300)]]:
            with self.subTest(new_lines=new_lines):
                old = observation(debug=federal(old_lines), lines=old_lines)
                new = observation(debug=federal(new_lines), lines=new_lines)
                status, pairs, _ = self.compare([old], [new])
                self.assertEqual(status, 1)
                self.assertEqual([d['path'] for d in pairs[0]['money_changes']],
                                 ['line.tax#0', 'line.tax#1'])
                self.assertEqual([(d['old'], d['new']) for d in pairs[0]['numeric_audit']],
                                 [('100', str(new_lines[0][1])), ('200', str(new_lines[1][1]))])
                self.assertEqual(pairs[0]['money_keys_compared'], 4)

    def test_removed_duplicate_occurrence_is_reported(self):
        old = observation(debug=federal([('tax', 100), ('tax', 200)]),
                          lines=[('tax', 100), ('tax', 200)])
        new = observation(debug=federal([('tax', 100)]), lines=[('tax', 100)])
        _, pairs, _ = self.compare([old], [new])
        self.assertEqual(pairs[0]['money_changes'][0]['path'], 'line.tax#1')
        self.assertFalse(pairs[0]['money_changes'][0]['new_present'])
        self.assertEqual(pairs[0]['numeric_audit'][0]['old'], '200')
        self.assertFalse(pairs[0]['numeric_audit'][0]['new_present'])

    def test_mapping_may_rename_but_never_merge_distinct_ids(self):
        old = observation(lines=[('old', 100), ('other', 200)])
        new = observation(lines=[('new', 100), ('other', 200)])
        self.assertEqual(self.compare([old], [new], mapping={'*': {'old': 'new'}})[0], 0)
        for mapping in [{'old': 'other'}, {'old': 'merged', 'other': 'merged'}]:
            with self.subTest(mapping=mapping):
                with self.assertRaisesRegex(ValueError, 'different line identities'):
                    self.compare([old], [new], mapping={'*': mapping})

    def test_byte_identical_complete_debug_proves_all_numbers_without_parsing(self):
        row = observation(debug='complete opaque Debug @ [10001, 999999999999999999999]')
        with mock.patch.object(FIGURES, 'debug_views', side_effect=AssertionError('parsed')):
            status, pairs, summary = self.compare([row], [row])
        self.assertEqual(status, 0)
        self.assertTrue(pairs[0]['identical_debug'])
        self.assertEqual(pairs[0]['numeric_audit'], [])
        self.assertEqual(summary['counts']['pairs_with_identical_debug'], 1)
        # Equal numeric prefixes cannot conceal a changed number at the end.
        new = observation(debug='Reply { values: [10001, 999999999999999999998] }')
        old = observation(debug='Reply { values: [10001, 999999999999999999999] }')
        self.assertEqual(self.compare([old], [new])[1][0]['numeric_audit'][0]['new'],
                         '999999999999999999998')

    def test_malformed_capture_and_unequal_debug_fail_explicitly(self):
        for bad in [[], [dict(observation(), version=2)],
                    [dict(observation(), index=1)],
                    [observation(), observation()]]:
            with self.subTest(capture=bad):
                with self.assertRaises(ValueError):
                    self.compare(bad, [observation()])
        with self.assertRaisesRegex(ValueError, 'case#0: unknown Debug syntax'):
            self.compare([observation(debug='Reply { x: @ }')], [observation()])
        path = self.root / 'malformed.jsonl'
        path.write_text('{broken JSON\n')
        with contextlib.closing(sqlite3.connect(':memory:')) as database:
            with self.assertRaises(json.JSONDecodeError):
                FIGURES.records(path, database, 'old')

    def test_whole_test_shards_have_exact_unsharded_census(self):
        old = [observation(test, index) for test, count in
               [('alpha', 3), ('beta', 1), ('delta', 2)] for index in range(count)]
        new = old + [observation('beta', 1), observation('new-only', 0)]
        full_status, full_pairs, full = self.compare(old, new)
        self.assertEqual(full_status, 1)
        shards = [self.compare(old, new, shard=(i, 3)) for i in range(3)]
        selected = [name for _, _, summary in shards for name in summary['tests_selected']]
        self.assertEqual(sorted(selected), sorted(full['tests_selected']))
        self.assertEqual(len(selected), len(set(selected)))
        self.assertEqual(sorted((p['test'], p['index']) for _, pairs, _ in shards for p in pairs),
                         sorted((p['test'], p['index']) for p in full_pairs))
        for key, value in full['counts'].items():
            self.assertEqual(sum(summary['counts'].get(key, 0) for _, _, summary in shards), value)
        self.assertEqual({name: rows for _, _, summary in shards
                          for name, rows in summary['unmatched_by_test'].items()},
                         full['unmatched_by_test'])
        for shard in [(-1, 3), (3, 3), (0, 0)]:
            with self.subTest(shard=shard), self.assertRaisesRegex(ValueError, 'test shard'):
                self.compare(old, new, shard=shard)

    def test_reused_index_is_opened_read_only_and_never_recaptured(self):
        path = self.root / 'events.sqlite3'
        row = observation()
        with contextlib.closing(sqlite3.connect(path)) as database:
            database.execute('CREATE TABLE events(side, test, ordinal INTEGER, kind, data, '
                             'PRIMARY KEY(side,test,ordinal))')
            for side in ['old', 'new']:
                FIGURES.records(self.capture(side + '.jsonl', [row]), database, side)
        original = path.read_bytes()
        connect = sqlite3.connect
        with mock.patch.object(FIGURES.sqlite3, 'connect', wraps=connect) as opened, \
                mock.patch.object(FIGURES, 'records', side_effect=AssertionError('recaptured')):
            status, pairs, _ = self.compare([], [], index=str(path))
        opened.assert_called_once_with(path.resolve().as_uri() + '?mode=ro', uri=True)
        self.assertEqual((status, len(pairs)), (0, 1))
        self.assertEqual(path.read_bytes(), original)
        with contextlib.closing(connect(path.resolve().as_uri() + '?mode=ro', uri=True)) as database:
            with self.assertRaisesRegex(sqlite3.OperationalError, 'readonly'):
                database.execute('DELETE FROM events')

    def source_mapping(self):
        return json.loads(pathlib.Path(__file__).with_name('income-source-structure-map.json').read_text())

    def test_every_declared_source_move_preserves_payload_and_detects_a_cent(self):
        spec = self.source_mapping()
        self.assertEqual(len(spec['variant_paths']), 90)
        self.assertEqual(sum(path[0] == {'tag': 'Record', 'field': 'value'} for path in spec['variant_paths'].values()), 46)
        old, new, changed = [], [], []
        for name, path in spec['variant_paths'].items():
            payload = 'Source { cents: -10001, label: "same", values: [] }'
            wrapped = payload
            for tag in reversed(path):
                wrapped = (tag + '(' + wrapped + ')' if isinstance(tag, str) else
                           tag['tag'] + ' { ' + tag['field'] + ': ' + wrapped + ' }')
            template = 'IncomeDocumentItem { id: 123, owner: Taxpayer, document: %s }'
            old.append(observation(name, debug=template % (name + '(' + payload + ')')))
            new.append(observation(name, debug=template % wrapped))
            changed.append(observation(name, debug=template % wrapped.replace('-10001', '-10002')))
        status, pairs, _ = self.compare(old, new, structure=spec)
        self.assertEqual((status, len(pairs)), (0, 90))
        self.assertTrue(all(not p['structural_changes'] and not p['numeric_audit'] for p in pairs))
        status, pairs, _ = self.compare(old, changed, structure=spec)
        self.assertEqual(status, 1)
        self.assertTrue(all(len(p['numeric_audit']) == 1 and p['structural_changes'] for p in pairs))

    def test_unmapped_text_tags_empty_containers_and_identity_fail(self):
        spec = self.source_mapping()
        old = observation(debug='IncomeDocumentItem { id: 123, owner: Taxpayer, document: W2(Source { label: "a", values: [] }) }')
        expected = 'IncomeDocumentItem { id: 123, owner: Taxpayer, document: Wage { value: W2(Source { label: "a", values: [] }) } }'
        for modified in [expected.replace('"a"', '"b"'), expected.replace('Taxpayer', 'Spouse'),
                         expected.replace('123', '124'), expected.replace('values: []', 'extra: []'),
                         expected.replace('value: W2(', 'value: Int1099(')]:
            with self.subTest(modified=modified):
                status, pairs, _ = self.compare([old], [observation(debug=modified)], structure=spec)
                self.assertEqual(status, 1)
                self.assertTrue(pairs[0]['structural_changes'])
        # A matching variant name outside the declared field is never transformed.
        row = observation(debug='Unrelated { document: W2(Source { cents: 100 }) }')
        altered = observation(debug=row['debug'].replace('W2(', 'Wage(W2(').replace(' })', ' }))'))
        self.assertEqual(self.compare([row], [altered], structure=spec)[0], 1)

    def test_declared_catalog_preserves_exact_category_and_kind_choices(self):
        spec = self.source_mapping()
        old = observation(debug='SaveIncomes { allowed_documents: [59, 0, 27, 85, 21, 22] }')
        new = observation(debug='SaveIncomes { allowed_documents: [59, 0, 27, 85, 21, 22], source_choices: IncomeSourceChoices { sources: [0, 5, 8, 9], wage: [0, 1], interestdividend: [], investmentsale: [], retirement: [], selfemployment: [], governmentpayment: [], otherincome: [], k1: [1, 4], canceleddebt: [], record: [] } }')
        self.assertEqual(self.compare([old], [new], structure=spec)[0], 0)
        for bad in [new['debug'].replace('k1: [1, 4]', 'k1: [1, 3]'),
                    new['debug'].replace('sources: [0, 5, 8, 9]', 'sources: [0, 5, 9]'),
                    new['debug'].replace('wage: [0, 1]', 'wage: [1, 0]')]:
            self.assertEqual(self.compare([old], [observation(debug=bad)], structure=spec)[0], 1)
        with self.assertRaisesRegex(ValueError, 'unknown original kind'):
            self.compare([observation(debug='SaveIncomes { allowed_documents: [88] }')], [new], structure=spec)

    def test_structure_mapping_rejects_collisions_unknown_variants_and_overrides(self):
        spec = self.source_mapping()
        old = observation(debug='IncomeDocumentItem { document: W2(Source { cents: 100 }) }')
        new = observation(debug='IncomeDocumentItem { document: Wage { value: W2(Source { cents: 100 }) } }')
        duplicate = copy.deepcopy(spec)
        duplicate['variant_paths']['Int1099'] = duplicate['variant_paths']['W2']
        with self.assertRaisesRegex(ValueError, 'one-to-one'):
            self.compare([old], [new], structure=duplicate)
        with self.assertRaisesRegex(ValueError, 'unknown source variant'):
            self.compare([observation(debug=old['debug'].replace('W2(', 'Unknown('))], [new], structure=spec)
        with self.assertRaisesRegex(ValueError, 'overwrite'):
            self.compare([observation(debug='SaveIncomes { allowed_documents: [], source_choices: None }')], [new], structure=spec)
        with self.assertRaisesRegex(ValueError, 'overrides'):
            self.compare([old], [new], structure=spec, mapping={'*': {'old': 'new'}})

    def test_strict_mapping_keeps_event_identity_count_kind_and_order(self):
        spec = self.source_mapping()
        a = observation('same', 0, 'Reply { amount: 100 }')
        b = observation('same', 1, 'Reply { amount: 200 }')
        for new in [[a], [a, b, observation('same', 2)],
                    [dict(a, kind='Send'), b],
                    [dict(b, index=0), dict(a, index=1)],
                    [dict(a, test='different'), dict(b, test='different')]]:
            with self.subTest(new=new):
                self.assertEqual(self.compare([a, b], new, structure=spec)[0], 1)

    def test_only_declared_error_path_moves_preserve_indices_and_leaf(self):
        spec = self.source_mapping()
        for variant in ['Business', 'Farm', 'InstallmentPayments', 'InstallmentSale']:
            for prefix in ['documents/documents/123/document/', 'document/']:
                before = '[' + prefix + variant + '/assets/17/basis]'
                after = '[' + prefix + 'value/' + variant + '/assets/17/basis]'
                old = observation(debug='Problem { field: Some(' + json.dumps(before) + ') }')
                new = observation(debug='Problem { field: Some(' + json.dumps(after) + ') }')
                self.assertEqual(self.compare([old], [new], structure=spec)[0], 0)
                for bad in [after.replace('/17/', '/18/'), after.replace('/basis]', '/cost]')]:
                    row = observation(debug='Problem { field: Some(' + json.dumps(bad) + ') }')
                    self.assertEqual(self.compare([old], [row], structure=spec)[0], 1)
        # Generic implicit traversal, another field and unrelated text are unchanged.
        for unchanged in ['Problem { field: None }',
                          'Problem { field: Some("[documents/documents/1/document/more/basis]") }',
                          'Unrelated { field: Some("[document/Farm/assets]") }']:
            row = observation(debug=unchanged)
            self.assertEqual(self.compare([row], [row], structure=spec)[0], 0)


if __name__ == '__main__':
    unittest.main()
