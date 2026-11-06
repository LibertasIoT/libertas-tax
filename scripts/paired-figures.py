#!/usr/bin/env python3
"""Ordered provenance comparison; no hash sorting, resynchronization or auto-ignore."""
import argparse
import collections
import contextlib
import copy
import gzip
import json
import io
import pathlib
import re
import shutil
import sqlite3
import subprocess
import sys
import tempfile
import zlib

EXCLUDED_CAPACITY_PREFIX = "tests::core_tests::unit53_capacity::"

TOKEN = re.compile(r'\s*("(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])*\'|-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?|[A-Za-z_]\w*(?:::\w+)*|[{}\[\](),:])')

class DebugParser:
    def __init__(self, text):
        self.tokens = []
        offset = 0
        while offset < len(text):
            match = TOKEN.match(text, offset)
            if not match:
                if not text[offset:].strip():
                    break
                raise ValueError(f"unknown Debug syntax at {offset}: {text[offset:offset+100]!r}")
            self.tokens.append(match[1])
            offset = match.end()
        self.pos = 0

    def peek(self):
        return self.tokens[self.pos] if self.pos < len(self.tokens) else None

    def pop(self, expected=None):
        value = self.peek()
        if value is None or (expected is not None and value != expected):
            raise ValueError(f"expected {expected!r}, got {value!r} at token {self.pos}")
        self.pos += 1
        return value

    def sequence(self, close):
        items = []
        while self.peek() != close:
            items.append(self.value())
            if self.peek() != close:
                self.pop(',')
        self.pop(close)
        return items

    def value(self):
        token = self.pop()
        if token in ('[', '('):
            return {'$items': self.sequence(']' if token == '[' else ')')}
        if token == '{':
            entries = []
            while self.peek() != '}':
                key = self.value()
                entries.append([key, self.value()] if self.peek() == ':' and self.pop(':') else [key])
                if self.peek() != '}':
                    self.pop(',')
            self.pop('}')
            return {'$entries': entries}
        if token.startswith(('"', "'")):
            return {'$text': token}  # Rust escapes retained verbatim, never parsed as numbers.
        if re.fullmatch(r'-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?', token):
            return {'$number': token}  # Preserve integer width and decimal spelling exactly.
        if not re.fullmatch(r'[A-Za-z_]\w*(?:::\w+)*', token):
            raise ValueError(f"unexpected token {token!r}")
        result = {'$tag': token}
        if self.peek() == '(':
            self.pop('(')
            result['$items'] = self.sequence(')')
        elif self.peek() == '{':
            self.pop('{')
            fields = {}
            while self.peek() != '}':
                name = self.pop()
                self.pop(':')
                if name in fields:
                    raise ValueError(f"duplicate Debug field {name}")
                fields[name] = self.value()
                if self.peek() != '}':
                    self.pop(',')
            self.pop('}')
            result['$fields'] = fields
        return result

    def parse(self):
        result = self.value()
        if self.peek() is not None:
            raise ValueError(f"unconsumed Debug tokens: {self.tokens[self.pos:self.pos+10]}")
        return result


def numeric(node, path='$'):
    if isinstance(node, dict):
        if '$number' in node:
            yield path, node['$number']
        else:
            for key, value in node.items():
                if key != '$text':
                    yield from numeric(value, path + '.' + key)
    elif isinstance(node, list):
        for index, value in enumerate(node):
            yield from numeric(value, f'{path}[{index}]')



def currencies(node, path='$'):
    if isinstance(node, dict):
        if node.get('$tag', '').split('::')[-1] == 'Currency':
            fields = node['$fields']
            yield path, {key: fields[key] for key in ('currency_code', 'scale', 'minor_units')}
        for key, value in node.items():
            yield from currencies(value, path + '.' + key)
    elif isinstance(node, list):
        for index, value in enumerate(node):
            yield from currencies(value, f'{path}[{index}]')


def structural_values(node, path='$'):
    """Include tags, text and empty containers, not just numeric leaves."""
    if isinstance(node, dict) and node:
        for key, value in node.items():
            yield from structural_values(value, path + '.' + key)
    elif isinstance(node, list) and node:
        for index, value in enumerate(node):
            yield from structural_values(value, f'{path}[{index}]')
    else:
        yield path, node


class StructureMap:
    """Declared source-field transformations; payloads are never rewritten."""

    def __init__(self, path):
        def unique_fields(pairs):
            result = {}
            for key, value in pairs:
                if key in result:
                    raise ValueError('duplicate structural mapping key: ' + key)
                result[key] = value
            return result
        spec = json.loads(pathlib.Path(path).read_text(), object_pairs_hook=unique_fields)
        if set(spec) != {'version', 'variant_paths', 'source_fields', 'catalogs', 'field_paths'} or spec['version'] != 1:
            raise ValueError('unsupported structural mapping')
        self.paths = spec['variant_paths']
        def valid_wrapper(wrapper):
            return ((isinstance(wrapper, str) and bool(wrapper))
                    or (isinstance(wrapper, dict) and set(wrapper) == {'tag', 'field'}
                        and all(isinstance(value, str) and value for value in wrapper.values())))
        if (not self.paths or any(not isinstance(path, list) or not path
                                 or any(not valid_wrapper(wrapper) for wrapper in path)
                                 for path in self.paths.values())
                or len({json.dumps(path, sort_keys=True) for path in self.paths.values()}) != len(self.paths)):
            raise ValueError('variant paths must be nonempty and one-to-one')
        paths = list(self.paths.values())
        if any(a == b[:len(a)] for a in paths for b in paths if a != b):
            raise ValueError('variant paths must not overlap by prefix')
        self.fields = {}
        for rule in spec['source_fields']:
            if set(rule) != {'type', 'field'} or (rule['type'], rule['field']) in self.fields:
                raise ValueError('duplicate or invalid source-field mapping')
            self.fields[rule['type'], rule['field']] = True
        self.catalogs = spec['catalogs']
        self.field_paths = spec['field_paths']
        self.applied = collections.Counter()

    def apply(self, original, *, side='old'):
        if side not in ('old', 'new'):
            raise ValueError('unknown structural mapping side')
        tree = copy.deepcopy(original)

        def visit(node):
            if isinstance(node, list):
                for item in node:
                    visit(item)
            elif isinstance(node, dict):
                tag, fields = node.get('$tag'), node.get('$fields', {})
                # Visit original children first: generated wrappers cannot match
                # another rule and accidentally transform the same value twice.
                for child in list(node.values()):
                    visit(child)
                for field, value in list(fields.items()) if side == 'old' else []:
                    if (tag, field) not in self.fields:
                        continue
                    if (set(value) != {'$tag', '$items'} or len(value['$items']) != 1
                            or value['$tag'] not in self.paths):
                        raise ValueError(f'unknown source variant at {tag}.{field}')
                    mapped = value['$items'][0]
                    for wrapper in reversed(self.paths[value['$tag']]):
                        mapped = ({'$tag': wrapper, '$items': [mapped]} if isinstance(wrapper, str)
                                  else {'$tag': wrapper['tag'], '$fields': {wrapper['field']: mapped}})
                    fields[field] = mapped
                    self.applied[f'{tag}.{field}'] += 1
                for rule in self.catalogs if side == 'old' else []:
                    if tag != rule['type']:
                        continue
                    self.catalog(fields, rule)
                for rule in self.field_paths if side == 'new' else []:
                    if tag == rule['type'] and rule['field'] in fields:
                        self.field_path(fields[rule['field']], rule)
        visit(tree)
        return tree

    def field_path(self, value, rule):
        if set(rule) != {'type', 'field', 'prefixes', 'inserted_field'}:
            raise ValueError('invalid field-path mapping')
        if value == {'$tag': 'None'}:
            return
        if (set(value) != {'$tag', '$items'} or value['$tag'] != 'Some'
                or len(value['$items']) != 1 or set(value['$items'][0]) != {'$text'}):
            raise ValueError('expected optional field-path text')
        text = value['$items'][0]
        path = json.loads(text['$text'])
        if not isinstance(path, str) or not path.startswith('[') or not path.endswith(']'):
            raise ValueError('expected square field path')
        segments = path[1:-1].split('/')
        for prefix in rule['prefixes']:
            if len(segments) <= len(prefix) or not all(
                    actual.isascii() and actual.isdigit() if expected == '#'
                    else actual == expected for actual, expected in zip(segments, prefix)):
                continue
            if segments[len(prefix)] == rule['inserted_field']:
                del segments[len(prefix)]
                text['$text'] = json.dumps('[' + '/'.join(segments) + ']')
                self.applied[f"{rule['type']}.{rule['field']}.path.{rule['inserted_field']}"] += 1
                return

    def catalog(self, fields, rule):
        """Derive only chooser indices from the declared original kind catalog."""
        if set(rule) != {'type', 'source', 'target', 'tag', 'kinds', 'fields'}:
            raise ValueError('invalid catalog mapping')
        source = fields.get(rule['source'])
        if not isinstance(source, dict) or set(source) != {'$items'}:
            raise ValueError('missing original kind catalog')
        if rule['target'] in fields:
            raise ValueError('catalog mapping would overwrite a field')
        outputs = {field: [] for field in rule['fields']}
        for value in source['$items']:
            if set(value) != {'$number'} or value['$number'] not in rule['kinds']:
                raise ValueError('unknown original kind catalog index')
            kind = rule['kinds'][value['$number']]
            if not kind or not set(kind) <= set(outputs):
                raise ValueError('invalid kind catalog mapping')
            for field, index in kind.items():
                if index is not None:
                    if not isinstance(index, int) or isinstance(index, bool) or index < 0:
                        raise ValueError('invalid kind catalog index')
                    number = {'$number': str(index)}
                    if number not in outputs[field]:
                        outputs[field].append(number)
        fields[rule['target']] = {'$tag': rule['tag'], '$fields': {
            field: {'$items': sorted(values, key=lambda value: int(value['$number']))}
            for field, values in outputs.items()}}
        self.applied[f"{rule['type']}.{rule['source']}->{rule['target']}"] += 1



def normalize_lines(tree):
    if tree.get('$tag') == 'FederalResult':
        # CalculationLine.id is the owner's stable identifier. A retired line
        # must not shift every following amount/explanation onto another line.
        # Test observations and every other collection retain capture order.
        fields = tree['$fields']
        lines = fields['lines']['$items']
        by_id = {}
        for line in lines:
            if line.get('$tag') != 'CalculationLine':
                raise ValueError('unexpected FederalResult calculation line')
            identity = line['$fields']['id']['$text']
            # Some existing reports repeat an ID at distinct calculation
            # stages. Retain every occurrence in its original order; never
            # overwrite, aggregate or sort duplicate amounts.
            by_id.setdefault(identity, []).append(line)
        fields['lines'] = {'$by_line_id': by_id}
    return tree


DEBUG_CACHE_BYTES = 16 * 1024 * 1024
_debug_views_cache = collections.OrderedDict()
_debug_views_cache_bytes = 0


def _debug_view_bytes(value):
    size = sys.getsizeof(value)
    if isinstance(value, dict):
        size += sum(_debug_view_bytes(key) + _debug_view_bytes(item)
                    for key, item in value.items())
    elif isinstance(value, (tuple, list)):
        size += sum(_debug_view_bytes(item) for item in value)
    return size


def debug_views(debug):
    global _debug_views_cache_bytes
    cached = _debug_views_cache.get(debug)
    if cached is not None:
        _debug_views_cache.move_to_end(debug)
        return cached[1]
    tree = normalize_lines(DebugParser(debug).parse())
    views = dict(numeric(tree)), dict(currencies(tree))
    del tree
    # Bound retained keys and views, not observations. Oversized payloads are
    # completely evaluated and returned without retaining their expanded paths.
    size = _debug_view_bytes((debug, views))
    if size <= DEBUG_CACHE_BYTES:
        while _debug_views_cache and (len(_debug_views_cache) >= 32
                                     or _debug_views_cache_bytes + size > DEBUG_CACHE_BYTES):
            _, (evicted_size, _) = _debug_views_cache.popitem(last=False)
            _debug_views_cache_bytes -= evicted_size
        _debug_views_cache[debug] = size, views
        _debug_views_cache_bytes += size
    return views


@contextlib.contextmanager
def capture_stream(path):
    """Read raw, gzip, or concatenated zstd frames without expanding a file."""
    if not str(path).endswith('.zst'):
        opener = gzip.open if str(path).endswith('.gz') else open
        with opener(path, 'rt', encoding='utf-8', newline='') as stream:
            yield stream
        return
    process = subprocess.Popen(['zstd', '-qdc', '--', str(path)], stdout=subprocess.PIPE)
    try:
        with io.TextIOWrapper(process.stdout, encoding='utf-8', newline='') as stream:
            yield stream
        if process.wait() != 0:
            raise ValueError(f'invalid zstd capture: {path}')
    finally:
        if process.poll() is None:
            process.kill()
        process.wait()


def records(path, database, side):
    counts = collections.Counter()
    byte_totals = collections.Counter()
    database.execute('CREATE TABLE IF NOT EXISTS capture_bytes '
                     '(side, test, bytes INTEGER, PRIMARY KEY(side,test))')
    with capture_stream(path) as stream:
        for line in stream:
            row = json.loads(line)
            excluded = row['test'].startswith(EXCLUDED_CAPACITY_PREFIX)
            if not excluded and (row['version'] != 1 or row['index'] != counts[row['test']]):
                raise ValueError(f"nonsequential/unsupported capture: {row['test']} #{row['index']}")
            counts[row['test']] += 1
            byte_totals[row['test']] += len(line.encode('utf-8'))
            database.execute('INSERT INTO events VALUES (?, ?, ?, ?, ?)',
                             (side, row['test'], counts[row['test']] - 1, row['kind'],
                              sqlite3.Binary(zlib.compress(line.encode('utf-8'), level=1))))
    if not counts:
        raise ValueError('empty capture')
    database.executemany('INSERT OR REPLACE INTO capture_bytes VALUES (?, ?, ?)',
                         ((side, test, size) for test, size in byte_totals.items()))
    database.commit()
    return counts


def delta(old, new):
    return [{'path': key, 'old': old.get(key), 'new': new.get(key),
             'old_present': key in old, 'new_present': key in new}
            for key in sorted(old.keys() | new.keys()) if old.get(key) != new.get(key)
            or (key in old) != (key in new)]


def money(row, mapping):
    if row['kind'] != 'Estimate':
        return {}
    values = {'top.' + key: value for key, value in row['money']['top'].items()}
    for index, line in enumerate(row['money']['return_rows']):
        if line['money'] is not None:
            values[f'return_rows[{index}].amount'] = line['amount']
            values[f'return_rows[{index}].money'] = line['money']
    occurrences = collections.Counter()
    mapped_sources = {}
    for line in row['money']['lines']:
        if line['kind'] != 'Money':
            continue
        key = 'line.' + mapping.get(line['id'], line['id'])
        if mapped_sources.setdefault(key, line['id']) != line['id']:
            raise ValueError(f"different line identities map to the same line: {key}")
        ordinal = occurrences[key]
        occurrences[key] += 1
        values[f'{key}#{ordinal}'] = line['amount']
    return values


def compare(args):
    if args.index:
        # Reuse complete, validated captures after an analysis correction;
        # never rewrite or silently recapture them.
        database = sqlite3.connect(pathlib.Path(args.index).resolve().as_uri() + '?mode=ro', uri=True)
        try:
            return compare_indexed(args, database)
        finally:
            database.close()
    # Index every event on disk with lossless compressed records; never retain
    # two expanded capture streams again in the temporary index.
    with tempfile.TemporaryDirectory(prefix='tax-figures-', dir=pathlib.Path(args.output).parent) as temp:
        database = sqlite3.connect(str(pathlib.Path(temp) / 'events.sqlite3'))
        try:
            database.execute('CREATE TABLE events(side, test, ordinal INTEGER, kind, data, PRIMARY KEY(side,test,ordinal))')
            return compare_indexed(args, database)
        finally:
            database.close()


def capture_byte_totals(database):
    """Read new metadata or recover exact sizes from a retained legacy index."""
    if database.execute("SELECT 1 FROM sqlite_master WHERE name='capture_bytes'").fetchone():
        return {(side, test): size for side, test, size in database.execute(
            'SELECT side,test,bytes FROM capture_bytes')}
    totals = collections.Counter()
    for side, test, data in database.execute('SELECT side,test,data FROM events'):
        totals[side, test] += len(zlib.decompress(data))
    return totals


def compare_indexed(args, database):
    structure_path = getattr(args, 'structure_map', None)
    structure_map = StructureMap(structure_path) if structure_path else None
    if structure_map and (args.reasons or args.line_map):
        raise ValueError('strict structural comparison disallows reasons and line-ID overrides')
    if args.index:
        old, new = (collections.Counter(dict(database.execute(
            'SELECT test,count(*) FROM events WHERE side=? GROUP BY test', (side,))))
                    for side in ['old', 'new'])
        if not old or not new:
            raise ValueError('reused index must contain both complete captures')
    else:
        old, new = records(args.old, database, 'old'), records(args.new, database, 'new')
    estimates = collections.Counter({(side, test): count for side, test, count in database.execute(
        "SELECT side,test,count(*) FROM events WHERE kind='Estimate' GROUP BY side,test")})
    excluded = {test: {'old_observations': old[test], 'new_observations': new[test],
                       'old_estimates': estimates['old', test],
                       'new_estimates': estimates['new', test]}
                for test in sorted(old.keys() | new.keys())
                if test.startswith(EXCLUDED_CAPACITY_PREFIX)}
    mappings = json.loads(pathlib.Path(args.line_map).read_text()) if args.line_map else {}
    reasons = json.loads(pathlib.Path(args.reasons).read_text()) if args.reasons else {}
    summary = collections.Counter()
    unmatched = {}
    selected = {}
    shard_index, shard_count = args.test_shard
    if shard_count <= 0 or not 0 <= shard_index < shard_count:
        raise ValueError('test shard requires 0 <= INDEX < COUNT')
    with gzip.open(args.output + '.pairs.jsonl.gz', 'wt') as stream:
        eligible_tests = sorted((old.keys() | new.keys()) - excluded.keys())
        for test_index, test in enumerate(eligible_tests):
            # Partition whole tests only. Every observation within a selected
            # test retains its exact capture ordinal and comparison behavior.
            if test_index % shard_count != shard_index:
                continue
            selected[test] = {'old': old[test], 'new': new[test]}
            paired = min(old[test], new[test])
            unmatched[test] = {}
            for side in ['old', 'new']:
                unmatched[test][side + '_only'] = [
                    {'index': index, 'kind': kind}
                    for index, kind in database.execute(
                        'SELECT ordinal,kind FROM events WHERE side=? AND test=? AND ordinal>=? ORDER BY ordinal',
                        (side, test, paired))]
            for index in range(paired):
                a, b = (json.loads(zlib.decompress(database.execute(
                    'SELECT data FROM events WHERE side=? AND test=? AND ordinal=?',
                    (side, test, index)).fetchone()[0])) for side in ['old', 'new'])
                summary['pairs_compared'] += 1
                item = {'test': test, 'index': index, 'old_kind': a['kind'], 'new_kind': b['kind']}
                item['kind_mismatch'] = a['kind'] != b['kind']
                # Equality of the entire payload proves all contained numbers
                # unchanged; unequal payloads are fully parsed, never sampled.
                item['identical_debug'] = a['debug'] == b['debug']
                if item['identical_debug']:
                    summary['pairs_with_identical_debug'] += 1
                    item['numeric_audit'] = []
                    item['currency_changes'] = []
                    item['structural_changes'] = []
                else:
                    try:
                        if structure_map:
                            old_tree = normalize_lines(structure_map.apply(DebugParser(a['debug']).parse()))
                            new_tree = normalize_lines(structure_map.apply(DebugParser(b['debug']).parse(), side='new'))
                            old_numbers, old_currency = dict(numeric(old_tree)), dict(currencies(old_tree))
                            new_numbers, new_currency = dict(numeric(new_tree)), dict(currencies(new_tree))
                            item['structural_changes'] = delta(
                                dict(structural_values(old_tree)), dict(structural_values(new_tree)))
                        else:
                            old_numbers, old_currency = debug_views(a['debug'])
                            new_numbers, new_currency = debug_views(b['debug'])
                            item['structural_changes'] = []
                    except ValueError as error:
                        raise ValueError(f"{test}#{index}: {error}") from error
                    item['numeric_audit'] = delta(old_numbers, new_numbers)
                    item['currency_changes'] = delta(old_currency, new_currency)
                old_money = money(a, mappings.get(test, mappings.get('*', {})))
                new_money = money(b, {})
                item['money_keys_compared'] = len(old_money.keys() | new_money.keys())
                item['money_changes'] = delta(old_money, new_money)
                changed = (item['kind_mismatch'] or bool(item['money_changes'])
                           or bool(item['currency_changes']) or bool(item['numeric_audit'])
                           or bool(item['structural_changes']))
                summary['pairs_with_estimate_money_changes'] += bool(item['money_changes'])
                summary['pairs_with_currency_changes'] += bool(item['currency_changes'])
                summary['typed_money_keys_compared'] += item['money_keys_compared']
                summary['pairs_with_numeric_audit_changes'] += bool(item['numeric_audit'])
                summary['pairs_with_kind_mismatch'] += item['kind_mismatch']
                summary['pairs_with_structural_changes'] += bool(item['structural_changes'])
                item['declared_reason'] = reasons.get(f'{test}#{index}')
                if changed and not item['declared_reason']:
                    summary['unexplained_pairs'] += 1
                stream.write(json.dumps(item, separators=(',', ':')) + '\n')
    unmatched = {test: rows for test, rows in unmatched.items() if rows['old_only'] or rows['new_only']}
    summary['old_only_events'] = sum(len(rows['old_only']) for rows in unmatched.values())
    summary['new_only_events'] = sum(len(rows['new_only']) for rows in unmatched.values())
    report = {'counts': dict(summary), 'unmatched_by_test': unmatched,
              'structure_mapping': {'path': structure_path, 'applied': dict(structure_map.applied)}
              if structure_map else None,
              'test_shard': {'index': shard_index, 'count': shard_count},
              'tests_selected': selected,
              'limitations': ['Byte-identical complete Debug payloads prove all contained numbers unchanged without parsing.',
                              'Debug numeric audit is a superset: IDs, counts, dates, encoded text bytes are NOT automatically tax figures.',
                              'Only explicit Estimate Money lines and top fields are typed money; nested financial carriers remain in the complete numeric audit.',
                              'Pairing is strictly same test and completion ordinal; kind mismatches are never resynchronized.',
                              'CalculationLine numeric and currency subtrees match by exact stable id and ordered occurrence; no duplicate amount is dropped.',
                              'Every other collection retains its order; explicit mappings cannot merge distinct line identities.',
                              'Added/removed events need review; changed test call sequences can shift later ordinal pairs.',
                              'No figure-equivalence claim until every delta, mapping and unmatched event is reviewed.']}
    if excluded:
        report['excluded_capacity_tests'] = excluded
    pathlib.Path(args.output + '.summary.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report['counts'], sort_keys=True))
    return 1 if summary['unexplained_pairs'] or summary['old_only_events'] or summary['new_only_events'] else 0


def overlay(destination):
    source = pathlib.Path(__file__).resolve().parents[1]
    target = pathlib.Path(destination).resolve()
    if (target / '.git').exists():
        raise ValueError('overlay accepts exported archives only; never a checkout/worktree')
    oracle = target / 'src/tests/cross/oracle.rs'
    text = oracle.read_text()
    if 'mod monetary_capture;' in text:
        raise ValueError('capture overlay already installed')
    anchors = [('extern crate std;', '\n\n#[path = "monetary_capture.rs"]\nmod monetary_capture;'),
               ('pub(crate) fn record_estimate(draft: &FederalDraft, result: &FederalResult) {', '\n    monetary_capture::estimate(result);'),
               ('pub(crate) fn finish_send(capture: Option<InputCapture>, session: &Interview, reply: &Reply) {', '\n    monetary_capture::send(session, reply);')]
    for anchor, addition in anchors:
        if text.count(anchor) != 1:
            raise ValueError('baseline instrumentation anchor missing/ambiguous: ' + anchor)
        text = text.replace(anchor, anchor + addition, 1)
    shutil.copyfile(source / 'src/tests/cross/monetary_capture.rs', oracle.with_name('monetary_capture.rs'))
    oracle.write_text(text)


def main():
    parser = argparse.ArgumentParser(__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    install = commands.add_parser('overlay', help='Instrument an already exported baseline (never Tax main).')
    install.add_argument('directory')
    check = commands.add_parser('compare')
    for name in ['old', 'new', 'output']:
        check.add_argument('--' + name, required=True)
    check.add_argument('--index', help='Reuse an already complete events SQLite index, read-only; old/new identify its capture streams.')
    check.add_argument('--test-shard', nargs=2, type=int, default=[0, 1], metavar=('INDEX', 'COUNT'),
                       help='Disjoint partition of whole tests for parallel analysis; merge every shard before claiming coverage.')
    check.add_argument('--line-map', help='JSON {test: {exact_old_line_id: exact_new_line_id}}; * default.')
    check.add_argument('--reasons', help='JSON {"test#completion_index": "declared reason for this pair"}.')
    check.add_argument('--structure-map', help='Strict declared one-to-one source type/path mapping; all other structural changes fail.')
    args = parser.parse_args()
    if args.command == 'overlay':
        overlay(args.directory)
        return 0
    return compare(args)

if __name__ == '__main__':
    raise SystemExit(main())
