#!/usr/bin/env python3
"""Synthetic control for the gate oracle-diff summary; CARGO_TARGET_DIR is the on-disk scratch root."""
import contextlib
import importlib.util
import io
import json
import os
import pathlib
import tempfile
import types
import unittest


def load(name, file):
    spec = importlib.util.spec_from_file_location(name, pathlib.Path(__file__).with_name(file))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


DIFF = load('oracle_diff', 'oracle-diff.py')
FIGURES = load('paired_figures', 'paired-figures.py')


def write_jsonl(path, rows):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(''.join(json.dumps(row) + '\n' for row in rows))


def event(case, tax):
    return {'version': 1, 'test': case, 'index': 0, 'kind': 'Estimate',
            'debug': f'FederalResult {{ lines: [CalculationLine {{ id: "tax", amount: {tax} }}] }}',
            'money': {'top': {'tax': tax}, 'return_rows': [],
                      'lines': [{'id': 'tax', 'kind': 'Money', 'amount': tax}]}}


class OracleDiffSummary(unittest.TestCase):
    def test_changed_failed_added_and_retired_cases(self):
        scratch = pathlib.Path(os.environ['CARGO_TARGET_DIR'])
        scratch.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix='oracle-diff-', dir=scratch) as temp:
            root = pathlib.Path(temp)
            write_jsonl(root / 'list/cases.jsonl', [
                {'input': 'a' * 64, 'tests': ['t::same']},
                {'input': 'b' * 64, 'tests': ['t::figure']},
                {'input': 'c' * 64, 'tests': ['t::gone']}])
            write_jsonl(root / 'list/changed.jsonl', [
                {'input': 'b' * 64, 'tests': ['t::figure'],
                 'expected': {'Estimate': {'result': 'old'}},
                 'actual': {'Ok': {'Estimate': {'result': 'new'}}}},
                {'input': 'c' * 64, 'tests': ['t::gone'],
                 'expected': {'Estimate': {'result': 'old'}},
                 'actual': {'Err': 'missing oracle input\nbacktrace'}}])
            write_jsonl(root / 'old/events.jsonl', [event('b' * 64, 10001)])
            write_jsonl(root / 'new/events.jsonl', [event('b' * 64, 10002)])
            args = types.SimpleNamespace(
                old=str(root / 'old/events.jsonl'), new=str(root / 'new/events.jsonl'),
                output=str(root / 'pairs'), test_shard=[0, 1], line_map=None, reasons=None,
                index=None)
            with contextlib.redirect_stdout(io.StringIO()):
                FIGURES.compare(args)
            for case in ['a', 'b', 'd']:
                (root / 'capture/cases').mkdir(parents=True, exist_ok=True)
                (root / 'capture/cases' / (case * 64 + '.json')).write_text('{}')
            (root / 'capture/origins').mkdir()
            (root / 'capture/origins/o1.json').write_text(json.dumps(['d' * 64, 't::new']))
            (root / 'capture/origins/o2.json').write_text(json.dumps(['a' * 64, 't::same']))
            text = DIFF.summarize(root / 'list', str(root / 'pairs'), root / 'capture', ['t::fig*'])
            undeclared = DIFF.summarize(root / 'list', str(root / 'pairs'), None, ['t::*'])
        self.assertEqual(text.splitlines()[0], 'ORACLE main_cases=3 changed=2 replay_failed=1 '
                         'undeclared=1 added=1 retired=1')
        self.assertIn('== t::figure (1 changed)\n  bbbbbbbbbbbbbbbb Estimate\n', text)
        self.assertIn('== undeclared changed cases (1)\n  cccccccccccccccc Estimate UNDECLARED '
                      'from t::gone\n    replay failed on candidate: missing oracle input\n', text)
        self.assertTrue(undeclared.startswith('ORACLE main_cases=3 changed=2 replay_failed=1 '
                                              'undeclared=0 added=n/a retired=n/a\n'))
        self.assertIn('    line.tax#0: 10001 -> 10002\n', text)
        self.assertIn('    top.tax: 10001 -> 10002\n', text)
        self.assertIn('    replay failed on candidate: missing oracle input\n', text)
        self.assertIn('== added inputs by test\n  t::new: 1\n', text)
        self.assertIn('== retired inputs by test\n  t::gone: 1\n', text)
        self.assertNotIn('t::same', text)


if __name__ == '__main__':
    unittest.main()
