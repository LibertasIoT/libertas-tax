#!/usr/bin/env python3
"""Gate oracle-diff summary: changed oracle cases by test, amounts old -> new, added/retired inputs.

Inputs are the oracle list mode directory (cases.jsonl, changed.jsonl), the optional
paired-figures.py output prefix for the changed inputs replayed on both binaries, and the
optional candidate oracle capture (cases/, origins/) from the gate's full run. A changed case is
declared only when every test that produced its input matches a declared pattern (`*` matches
any text, including `::`); undeclared cases are listed again, with their amounts, at the end.
"""
import argparse
import collections
import fnmatch
import gzip
import json
import pathlib

SHOWN = 12  # amounts printed per case; the pairs file keeps every one


def jsonl(path):
    with open(path) as stream:
        return [json.loads(line) for line in stream if line.strip()]


def shards(directory, name):
    # The single file, or every `<name>-<k>.jsonl` shard in shard order.
    directory = pathlib.Path(directory)
    single = directory / f'{name}.jsonl'
    if single.exists():
        return [single]
    found = sorted(directory.glob(f'{name}-*.jsonl'), key=lambda p: int(p.stem.rsplit('-', 1)[1]))
    if not found:
        raise FileNotFoundError(single)
    return found


def jsonl_all(directory, name):
    return [row for path in shards(directory, name) for row in jsonl(path)]


def amounts(pair):
    # Observations of different kinds (an Estimate paired with a Send) are not comparable:
    # a request-shape change, never a money delta.
    if pair['kind_mismatch']:
        return [f"observation kind {pair['old_kind']} -> {pair['new_kind']} (shape change; amounts not comparable)"]
    rows = []
    for change in pair['money_changes']:
        rows.append(f"{change['path']}: {change['old']} -> {change['new']}")
    for change in pair['currency_changes']:
        rows.append(f"{change['path']} Currency: {change['old']} -> {change['new']}")
    if not pair['money_changes'] and pair['numeric_audit']:
        rows.append(f"no typed money changed; {len(pair['numeric_audit'])} Debug number(s) changed, first "
                    + ', '.join(f"{c['path']}: {c['old']} -> {c['new']}" for c in pair['numeric_audit'][:3]))
    return rows


def summarize(list_dir, pairs_prefix=None, capture=None, declared=()):
    list_dir = pathlib.Path(list_dir)
    main_cases = {row['input']: row['tests'] for row in jsonl_all(list_dir, 'cases')}
    changed = jsonl_all(list_dir, 'changed')
    pairs = collections.defaultdict(list)
    unmatched = {}
    if pairs_prefix:
        with gzip.open(pairs_prefix + '.pairs.jsonl.gz', 'rt') as stream:
            for line in stream:
                pair = json.loads(line)
                pairs[pair['test']].append(pair)
        unmatched = json.loads(pathlib.Path(pairs_prefix + '.summary.json').read_text())['unmatched_by_test']
    by_test = collections.defaultdict(list)
    undeclared = []
    failed = 0
    for case in changed:
        lines = []
        if 'Err' in case['actual']:
            failed += 1
            lines.append('replay failed on candidate: ' + case['actual']['Err'].splitlines()[0][:200])
        else:
            for pair in pairs.get(case['input'], []):
                rows = amounts(pair)
                prefix = f"#{pair['index']} " if len(pairs[case['input']]) > 1 else ''
                lines += [prefix + row for row in rows[:SHOWN]]
                if len(rows) > SHOWN:
                    lines.append(f"{prefix}... {len(rows) - SHOWN} more in the pairs file")
            for side, rows in unmatched.get(case['input'], {}).items():
                if rows:
                    lines.append(f"{side} observations: {len(rows)}")
            if not lines:
                lines.append('output digest changed; no figure or Debug number changed')
        kind = next(iter(case['expected']))
        tests = case['tests'] or ['(no recorded test)']
        if not all(any(fnmatch.fnmatchcase(test, p) for p in declared) for test in tests):
            kind += ' UNDECLARED'
            undeclared.append((case['input'], kind, lines, tests))
        for test in tests:
            by_test[test].append((case['input'], kind, lines))
    added, retired = collections.defaultdict(list), collections.defaultdict(list)
    candidate = None
    if capture:
        capture = pathlib.Path(capture)
        candidate = {path.stem for path in (capture / 'cases').glob('*.json')}
        for path in (capture / 'origins').glob('*.json'):
            case, test = json.loads(path.read_text())
            if case not in main_cases:
                added[test].append(case)
        for case, tests in main_cases.items():
            if case not in candidate:
                for test in tests or ['(no recorded test)']:
                    retired[test].append(case)
    out = [f"ORACLE main_cases={len(main_cases)} changed={len(changed)} "
           f"replay_failed={failed} undeclared={len(undeclared)} "
           f"added={'n/a' if candidate is None else len(candidate - main_cases.keys())} "
           f"retired={'n/a' if candidate is None else len(main_cases.keys() - candidate)}",
           'declared figure changes: ' + (', '.join(declared) or 'none')]
    for test in sorted(by_test):
        out.append(f"== {test} ({len(by_test[test])} changed)")
        for case, kind, lines in by_test[test]:
            out.append(f"  {case[:16]} {kind}")
            out += ['    ' + line for line in lines]
    if undeclared:
        out.append(f"== undeclared changed cases ({len(undeclared)})")
        for case, kind, lines, tests in undeclared:
            out.append(f"  {case[:16]} {kind} from {', '.join(tests)}")
            out += ['    ' + line for line in lines]
    for title, groups in (('added', added), ('retired', retired)):
        if groups:
            out.append(f"== {title} inputs by test")
            for test in sorted(groups):
                out.append(f"  {test}: {len(set(groups[test]))}")
    return '\n'.join(out) + '\n'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--list', required=True, help='oracle TAX_ORACLE_LIST directory')
    parser.add_argument('--pairs', help='paired-figures.py --output prefix for the changed inputs')
    parser.add_argument('--capture', help='candidate TAX_ORACLE_CAPTURE directory')
    parser.add_argument('--declared', default='none',
                        help='comma-separated declared test patterns, or none')
    parser.add_argument('--output', required=True)
    args = parser.parse_args()
    declared = [] if args.declared.strip() == 'none' else [
        pattern.strip() for pattern in args.declared.split(',') if pattern.strip()]
    text = summarize(args.list, args.pairs, args.capture, declared)
    pathlib.Path(args.output).write_text(text)
    print(text, end='')


if __name__ == '__main__':
    main()
