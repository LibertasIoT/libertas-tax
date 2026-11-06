#!/usr/bin/env python3
"""Focused source-tool regressions; all writes stay in temporary Rust trees."""

import pathlib
import runpy
import tempfile
import unittest


SPLIT = runpy.run_path(str(pathlib.Path(__file__).with_name('split-hubs')))
MARKER = SPLIT['MARKER']
END_MARKER = SPLIT['END_MARKER']
REGISTRATION = '#[path = "../domain/sample_model.rs"]\nmod sample_model;\n'
AUTHORED = '// Authored form owner.\nuse super::*;\n\npub(super) struct SampleOwned;\n'


class SplitHubsOwnershipTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = pathlib.Path(self.temp.name)
        self.hub = self.root / 'src/shared/hub.rs'
        self.child = self.root / 'src/domain/sample_model.rs'
        self.hub.parent.mkdir(parents=True)
        self.hub.write_text('pub(crate) struct Shared;\n')
        self.write_plan(self.plan())

    def plan(self):
        return SPLIT['split_hub'](str(self.root), 'src/shared/hub.rs', 'model',
                                  'pub(crate)', [('sample', 'domain', 'Sample*')])

    def write_plan(self, plan):
        for path, text in plan:
            path = pathlib.Path(path)
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text)

    def test_registered_authored_owner_is_unchanged(self):
        self.child.write_text(AUTHORED)
        self.assertEqual(self.plan(), [])
        self.assertEqual(self.child.read_text(), AUTHORED)

    def test_extraction_appends_to_authored_owner_and_is_idempotent(self):
        self.child.write_text(AUTHORED)
        self.hub.write_text('struct SampleAdded;\n' + self.hub.read_text())
        plan = self.plan()
        self.assertEqual(dict(plan)[str(self.child)], AUTHORED + 'pub(super) struct SampleAdded;\n')
        self.assertNotIn('struct SampleAdded;', dict(plan)[str(self.hub)])
        self.assertEqual(self.child.read_text(), AUTHORED)
        self.write_plan(plan)
        self.assertEqual(self.plan(), [])

    def test_unrelated_authored_collisions_are_refused(self):
        self.child.write_text(AUTHORED)
        blocks = {
            'unregistered': '',
            'wrong name': REGISTRATION.replace('mod sample_model;', 'mod other_model;'),
            'wrong path': REGISTRATION.replace('../domain/', '../other/'),
            'comment': '/*\n' + REGISTRATION + '*/\n',
            'string': 'const TEXT: &str = r#"' + REGISTRATION + '"#;\n',
        }
        for case, block in blocks.items():
            with self.subTest(case=case):
                self.hub.write_text(MARKER + '\n' + block + END_MARKER + '\n')
                with self.assertRaisesRegex(SystemExit, 'exists and is not a split-hubs module'):
                    self.plan()
                self.assertEqual(self.child.read_text(), AUTHORED)
        self.hub.write_text(REGISTRATION + MARKER + '\n' + END_MARKER + '\n')
        with self.assertRaisesRegex(SystemExit, 'exists and is not a split-hubs module'):
            self.plan()

    def test_whole_fake_blocks_are_not_module_registrations(self):
        self.child.write_text(AUTHORED)
        block = MARKER + '\n' + REGISTRATION + END_MARKER + '\n'
        wrapped = {
            'raw string': 'const TEXT: &str = r#"\n' + block + '"#;\n',
            'block comment': '/*\n' + block + '*/\n',
            'function': 'fn example() {\n' + block + '}\n',
            'fake start marker': '/*\n' + MARKER + '\n*/\n' + REGISTRATION
                + END_MARKER + '\n',
            'fake end marker': MARKER + '\n' + REGISTRATION + '/*\n'
                + END_MARKER + '\n*/\n',
        }
        for case, source in wrapped.items():
            with self.subTest(case=case):
                self.hub.write_text(source)
                with self.assertRaisesRegex(SystemExit, 'exists and is not a split-hubs module'):
                    self.plan()
                self.assertEqual(self.child.read_text(), AUTHORED)

    def test_generated_header_still_supports_initial_registration(self):
        generated = self.child.read_text()
        self.hub.write_text('pub(crate) struct Shared;\n')
        self.write_plan(self.plan())
        self.assertEqual(self.child.read_text(), generated)
        self.assertEqual(self.plan(), [])


if __name__ == '__main__':
    unittest.main()
