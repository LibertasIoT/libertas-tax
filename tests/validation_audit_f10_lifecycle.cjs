/* Run with LIBERTAS_STUDIO_PATH and TAX_SCHEMA_PREFIX from ordinary Studio extraction. */
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const Module = require('node:module');
const studio = process.env.LIBERTAS_STUDIO_PATH;
const prefix = process.env.TAX_SCHEMA_PREFIX;
assert(studio && prefix, 'Set LIBERTAS_STUDIO_PATH and TAX_SCHEMA_PREFIX.');
// SchemaValue's development traces are verbose; assertion failures remain visible.
const report = console.log.bind(console);
console.log = () => {};
const originalLoad = Module._load;
Module._load = function (name, ...args) {
    if (name === 'vscode') return { env: { language: 'en' }, window: {} };
    return originalLoad.call(this, name, ...args);
};
const { ReferenceContext, SchemaPackage, SchemaValue } = require(path.join(studio, 'out/schema/schema_value.js'));
const { ProtocolEditorSession, serializeEditorValue, encodeProtocolValue } = require(path.join(studio, 'out/panels/shared/ProtocolEditorSession.js'));
const { AvroDecoder, decodeAvroFully, getEffectiveChildren } = require(path.join(studio, 'out/panels/shared/SchemaAvroValue.js'));
const { Writer } = require(path.join(studio, 'out/protocol/libertas_codec.js'));
const { encodeAvro } = require(path.join(studio, 'out/panels/ControlPanel.js'));
const { allowedFieldChoices } = require(path.join(studio, 'out/schema/runtimeConstrainedBy.js'));
const { QWHAScriptTaskData } = require(path.join(studio, 'out/front_end/QWHAScriptTaskData.js'));
const pkg = new SchemaPackage(JSON.parse(fs.readFileSync(prefix + 'schema.json')), JSON.parse(fs.readFileSync(prefix + 'document.json')));
pkg.userDefaultContext = { currentUserId: 7n };
const protocol = pkg.types.find(t => t.type.NativeName === 'TaxInterviewProtocol');
const saveType = protocol.children.find(t => t.type.NativeName === 'SaveIncomes');
const fixtures = JSON.parse(fs.readFileSync(path.join(__dirname, 'fixtures/validation-audit-f10.json'))).cases;

// Hydrate only the delivered synthetic page. New rows below use the actual
// editor itemTemplate and choices use ProtocolEditorSession.selectVariant.
function delivered(declared, supplied, parent) {
    let selected = declared;
    const kind = declared.getUnderlyingType(pkg.types);
    if (supplied === undefined) {
        const initial = declared.getAttribute('Fixed', pkg.types) || declared.getAttribute('Default', pkg.types);
        if (initial?.Value === '$null' || declared.isNullable(pkg.types)) supplied = null;
        else if (initial && kind !== 'Table' && kind !== 'List') supplied = initial.Value;
    }
    if (kind === 'Union' && supplied !== null) {
        const fields = getEffectiveChildren(declared, pkg.types);
        const name = supplied && typeof supplied === 'object' ? Object.keys(supplied)[0] : supplied;
        selected = name === undefined ? fields[0] : fields.find(t => t.type.NativeName === name);
        assert(selected, `Unknown union variant ${name}`);
        supplied = supplied && typeof supplied === 'object' ? supplied[name] : undefined;
    }
    const value = new SchemaValue(pkg, parent, selected, declared, null);
    if (supplied === null && declared.isNullable(pkg.types)) return value;
    const actual = value.getQWHAType(), fields = getEffectiveChildren(selected, pkg.types);
    if (actual === 'Table') {
        value.value = '';
        value.children = fields.map(field => delivered(field, supplied?.[field.type.NativeName], value));
    } else if (actual === 'List') {
        value.value = '';
        value.children = (supplied || []).map(item => delivered(fields[0], item, value));
    } else if (actual === 'Nil') value.value = '';
    else if (actual === 'Enumeration') value.value = value.resolveEnumValue(supplied ?? 0);
    else if (actual === 'Boolean') value.value = supplied === true || supplied === 'true' ? 'true' : 'false';
    else if (actual === 'String') value.value = supplied === undefined ? 'x'.repeat(Number(value.getAttribute('SizeMin')?.Value || 0)) : String(supplied);
    else if (actual === 'Bytes') value.value = supplied ?? new Uint8Array();
    else value.value = String(supplied ?? Math.max(value.getAttribute('DateOnly') || actual === 'DateOnly' ? 20260101 : 0, Number(value.getAttribute('NumberMin')?.Value || 0)));
    return value;
}
const at = (root, copyPath) => {
    const value = root.resolveCopyPath(copyPath);
    assert(value, `Missing ${copyPath}`);
    return value;
};
function editor(fixture, empty = false) {
    const json = structuredClone(fixture.value);
    if (empty) json.documents.documents = [];
    const value = delivered(saveType, json);
    value.unionTypeInfo = protocol;
    value.setReferenceContext(new ReferenceContext(value));
    return new ProtocolEditorSession(1, 2, 3, 'validation-f10', pkg, protocol, value);
}
function uiPath(node) {
    const indices = [];
    while (node.parent) { indices.unshift(node.parent.children.indexOf(node)); node = node.parent; }
    return 'root' + indices.map(index => `.children[${index}]`).join('');
}
const clone = value => JSON.parse(JSON.stringify(value));
function choose(session, copyPath, variant) {
    session.selectVariant(serializeEditorValue(session.value), uiPath(at(session.value, copyPath)), variant);
}
function assertChoices(field) {
    assert.deepEqual([...allowedFieldChoices(field)].map(Number).sort(), [1, 2]);
}

let cases = 0;
for (const entity of ['K1Scorp', 'K1Partnership']) {
    const fixture = fixtures.find(f => f.name === `${entity} unknown history`);
    assert(fixture);
    const session = editor(fixture, true);
    const rows = at(session.value, '$.documents.documents');
    const proposed = serializeEditorValue(session.value);
    const listPath = uiPath(rows);
    const indices = [...listPath.matchAll(/\[(\d+)\]/g)].map(match => Number(match[1]));
    const list = indices.reduce((node, index) => node.children[index], proposed);
    list.children.push(clone(list.itemTemplate));
    session.change(proposed, listPath);
    assert.equal(rows.children.length, 1, `${entity}: actual itemTemplate adds a fresh row`);
    const documentPath = '$.documents.documents[0].document';
    choose(session, documentPath, entity);
    choose(session, `${documentPath}.basis.start`, 'Continuing');
    const riskPath = `${documentPath}.basis.start.risk_history`;
    assertChoices(at(session.value, riskPath));
    assert.equal(at(session.value, riskPath).getNativeName(), 'Unknown', 'default stays permitted');
    const before = at(session.value, riskPath);
    assert.throws(() => choose(session, riskPath, 'New'), /not permitted/);
    assert.equal(at(session.value, riskPath), before, 'rejected selection preserves the live value');
    choose(session, riskPath, 'Detailed');
    assertChoices(at(session.value, riskPath));
    choose(session, riskPath, 'Unknown');
    assert.equal(session.pending, undefined, 'editing does not submit the page');
    cases++;

    // The complete delivered page uses the real calculation scheduler. Every
    // request must carry the context catalog and validate with its bound K-1.
    const completed = editor(fixture);
    assert.deepEqual(QWHAScriptTaskData.validateValues(pkg, [completed.value]), []);
    const changed = serializeEditorValue(completed.value);
    const name = at(completed.value, '$.documents.documents[0].document.name');
    const namePath = uiPath(name);
    [...namePath.matchAll(/\[(\d+)\]/g)].reduce((node, match) => node.children[Number(match[1])], changed).value = 'Synthetic edited entity';
    completed.change(changed, namePath);
    const wanted = new Set(['PreviewDocument', 'EstimateDocuments', 'DocumentPropertyOptions']);
    for (let attempts = 0; attempts < 100 && wanted.size; attempts++) {
        const [attempt] = completed.calculations.ready();
        if (!attempt) break;
        if (wanted.has(attempt.name)) {
            assert.deepEqual(at(attempt.request, '$.allowed_continuing_risk').children.map(child => Number(child.value)), [1, 2], `${entity}: ${attempt.name} carries the server catalog`);
            assert.deepEqual(QWHAScriptTaskData.validateValues(pkg, [attempt.request]), [], `${entity}: ${attempt.name}`);
            const boundRisk = at(attempt.request, attempt.name === 'EstimateDocuments' ? '$.documents.documents[0].document.basis.start.risk_history' : '$.document.basis.start.risk_history');
            assertChoices(boundRisk);
            wanted.delete(attempt.name);
        }
        // Supply the ordinary upstream reply shapes so the scheduler may release
        // dependent estimates; these synthetic results do not test tax amounts.
        const reply = attempt.name === 'DocumentPropertyOptions'
            ? ['DocumentPropertyChoices', { allowed_community: [0] }]
            : attempt.name === 'PreviewDocument'
                ? ['IncomePreview', { investment_caps: null, preview: 0 }]
                : undefined;
        const response = reply && delivered(protocol.children.find(type => type.type.NativeName === reply[0]), reply[1]);
        if (response) response.unionTypeInfo = protocol;
        completed.calculations.complete(attempt.id, response);
    }
    assert.deepEqual([...wanted], [], `${entity}: all three calculation projections were ready`);
    cases++;

    const writer = new Writer();
    encodeProtocolValue(writer, completed.value, encodeAvro);
    const restored = decodeAvroFully(new AvroDecoder(writer.finish()), pkg, undefined, protocol);
    restored.setReferenceContext(new ReferenceContext(restored));
    const reopened = new ProtocolEditorSession(2, 2, 3, 'validation-f10', pkg, protocol, restored);
    assert.deepEqual(QWHAScriptTaskData.validateValues(pkg, [reopened.value]), [], `${entity}: complete reopened request validates`);
    assertChoices(at(reopened.value, riskPath));
    assert.equal(at(reopened.value, riskPath).getNativeName(), 'Unknown');
    assert.throws(() => choose(reopened, riskPath, 'New'), /not permitted/);
    cases++;
}
report(`Validation audit F10 actual-editor lifecycle/projection cases passed: ${cases}`);
