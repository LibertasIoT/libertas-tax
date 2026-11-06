// Provider regression through Studio's ordinary AI Import owner. The caller
// supplies an empty, disposable Document center editor on the candidate App.
// LIBERTAS_STUDIO_PATH=/path/to/vscode LIBERTAS_SCHEMA_LAUNCHER=/path/to/launcher.cjs \
//   node tests/k1_classification.cjs editor-args.json PDF_DIRECTORY result.json
// editor-args.json: {"session":"...","editor":"...","path":"...","revision":1}
// The PDF directory contains partner-k1.pdf and scorp-k1.pdf matching the full
// synthetic paper text below. This invokes the configured Hub provider; it is
// separate from Rust tests and must run after candidate schema validation.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { createRequire } = require('node:module');
const { execFileSync } = require('node:child_process');

async function main() {
    const [editorArgs, pdfDirectory, resultPath] = process.argv.slice(2);
    const studio = process.env.LIBERTAS_STUDIO_PATH;
    const launcher = process.env.LIBERTAS_SCHEMA_LAUNCHER;
    assert(editorArgs && pdfDirectory && resultPath && studio && launcher,
        'Supply editor arguments, original synthetic PDF directory, result path and Studio environment.');
    const args = JSON.parse(fs.readFileSync(editorArgs, 'utf8'));
    assert(args.session && args.editor && args.path, 'An offered import host is required.');
    const cases = [
        { name: 'partner-k1', kind: 'K1Partnership' },
        { name: 'scorp-k1', kind: 'K1Scorp' },
    ];
    args.files = cases.map(({ name }) => {
        const pdf = path.join(pdfDirectory, name + '.pdf');
        const text = execFileSync('pdftotext', ['-layout', pdf, '-'], { encoding: 'utf8' });
        assert.equal(text, fs.readFileSync(path.join(__dirname, 'fixtures/ai-import/k1', name + '.txt'), 'utf8'),
            name + ': use the synthetic fiscal-year paper matching the complete text fixture');
        return { name: name + '.pdf', sourceMimeType: 'application/pdf', mimeType: 'application/pdf',
            contentBase64: fs.readFileSync(pdf).toString('base64') };
    });
    const studioRequire = createRequire(path.join(studio, 'package.json'));
    const { Client } = studioRequire('@modelcontextprotocol/sdk/client/index.js');
    const { StdioClientTransport } = studioRequire('@modelcontextprotocol/sdk/client/stdio.js');
    const client = new Client({ name: 'k1-classification-regression', version: '1' });
    try {
        await client.connect(new StdioClientTransport({ command: 'node', args: [launcher], stderr: 'inherit' }));
        const result = await client.callTool({ name: 'qa_editor_import', arguments: args }, undefined,
            { timeout: 900000 });
        fs.writeFileSync(resultPath, JSON.stringify(result, null, 2));
        assert(!result.isError, 'AI Import tool failed; inspect the saved result.');
        const body = result.structuredContent ?? JSON.parse(result.content
            .filter(item => item.type === 'text').map(item => item.text).join('\n'));
        const imported = body.import;
        assert(imported, 'Missing ordinary AI Import review result.');
        assert.deepEqual(imported.unclassified, [], 'Every paper must be classified.');
        assert.deepEqual(imported.notImported, [], 'Every paper must be imported.');
        assert.equal(imported.duplicates, 0);
        assert.equal(imported.rows.length, cases.length, 'Keep all attachments with their one K-1.');
        cases.forEach(({ name, kind }, index) => {
            const rows = imported.rows.filter(row => row.parts.some(part => part.file === index + 1));
            assert.equal(rows.length, 1, name + ': one classified document');
            const row = rows[0];
            assert.equal(JSON.parse(row.kind).at(-1), kind, name + ': ordinary fiscal-year K-1 kind');
            assert.deepEqual(row.parts, [{ file: index + 1, pages: [1, 2, 3] }], name + ': all attached pages');
            assert.equal(row.status, 'Ready', name + ': fiscal-year import must succeed');
            assert(!row.error, name + ': no year-conflict refusal');
        });
        console.log('PASS: both fiscal-year K-1s classified and imported with their attachments.');
    } finally {
        await client.close();
    }
}
main().catch(error => { console.error(error); process.exitCode = 1; });
