// Run with Studio's installed Jest and LIBERTAS_STUDIO pointing to its vscode directory.
// This exercises the compiled App schema through the existing shared client owners.
jest.mock('vscode', () => ({ env: { language: 'en' }, window: {} }), { virtual: true });
const path = require('path');
const studio = process.env.LIBERTAS_STUDIO;
const { generateSchemaAndDocument } = require(path.join(studio, 'src/schema/schemaCompilation'));
const { SchemaPackage, SchemaValue } = require(path.join(studio, 'src/schema/schema_value'));
const { CalculationSession } = require(path.join(studio, 'src/panels/shared/CalculationSession'));
const { ProtocolEditorSession, serializeEditorValue } = require(path.join(studio, 'src/panels/shared/ProtocolEditorSession'));
const { draftSnapshot, restoreDraft } = require(path.join(studio, 'src/panels/shared/WorkflowDraft'));
const { QWHAScriptTaskData } = require(path.join(studio, 'src/front_end/QWHAScriptTaskData'));
const { hasEditorInput } = require(path.join(studio, 'src/panels/shared/EditorPresentation'));
let pkg, protocol, compiled;
beforeAll(() => {
  compiled = generateSchemaAndDocument(path.resolve(__dirname, '../..'), 'src/lib.rs', { log: () => {} });
  pkg = new SchemaPackage(JSON.parse(compiled.schema));
  pkg.userDefaultContext = { currentUserId: 7 };
  protocol = pkg.types.find(type => type.type.NativeName === 'TaxInterviewProtocol');
  expect(protocol).toBeDefined();
});
const field = (value, name) => {
  const found = value.children.find(child => child.unionTypeInfo.type.NativeName === name);
  if (!found) throw new Error(`Missing ${name} in ${value.getNativeName()}`);
  return found;
};
const variant = name => protocol.children.find(type => type.type.NativeName === name);
function put(parent, name, json) {
  const old = field(parent, name);
  const value = SchemaValue.deserializeJSON(pkg, json, old.unionTypeInfo, parent);
  parent.children[parent.children.indexOf(old)] = value;
  return value;
}
function append(list) {
  const row = new SchemaValue(pkg, list, list.getEffectiveType().children[0]);
  list.children.push(row);
  return row;
}
function select(parent, name, choice) {
  const old = field(parent, name);
  const type = old.unionTypeInfo.children.find(type => type.type.NativeName === choice);
  if (!type) throw new Error(`Missing choice ${choice}`);
  const value = new SchemaValue(pkg, parent, type, old.unionTypeInfo);
  parent.children[parent.children.indexOf(old)] = value;
  return value;
}
function editor(host, revision = 4, parts = []) {
  const root = new SchemaValue(pkg, undefined, variant(host), protocol);
  put(root, 'cookie', 'synthetic-payment-parts');
  put(root, 'revision', revision);
  put(root, 'topic', 'SelfEmployment');
  if (host === 'SaveIncomes') put(root, 'review', false);
  const question = append(field(root, 'followups'));
  put(question, 'key', 'income/42/parts');
  const answer = put(question, 'answer', { PaymentParts: { value: parts } });
  const discriminator = answer.typeInfo.getAttribute('Discriminator', pkg.types);
  put(question, 'allowed_answers', [Number(discriminator.Value)]);
  return { root, question };
}
const partsOf = question => field(field(question, 'answer'), 'value');
function nodePath(value) {
  const steps = [];
  while (value.parent) {
    steps.unshift(`.children[${value.parent.children.indexOf(value)}]`);
    value = value.parent;
  }
  return 'root' + steps.join('');
}
function pendingBusiness(root, name) {
  const row = append(field(field(root, 'documents'), 'documents'));
  put(row, 'id', 0);
  const business = select(row, 'document', 'Business');
  const index = business.typeInfo.getAttribute('Discriminator', pkg.types);
  put(field(root, 'source_choices'), 'record', [Number(index.Value)]);
  put(business, 'name', name);
  return business;
}
const returnedPart = { id: 100, amount: 40000, belongs: null, refunded: 0 };
function acceptedReply(revision = 5, part = returnedPart) {
  const reply = SchemaValue.deserializeJSON(pkg, { value: [part], revision }, variant('PaymentPartsSaved'));
  reply.unionTypeInfo = protocol;
  return reply;
}

test.each(['SaveIncomes', 'EstimateDocuments'])('%s opens the first split composer and applies its return', host => {
  const { root, question } = editor(host);
  const business = host === 'SaveIncomes' ? pendingBusiness(root, 'Unfinished consulting') : undefined;
  const session = new ProtocolEditorSession(1, 2, 3, 'tax-splits', pkg, protocol, root);
  // Normal session initialization precedes the person's link action. Snapshot
  // actual values/selection identity, allowing dependent presentation to refresh.
  const pending = business && draftSnapshot(business);
  // The parent remains incomplete; an empty copied list is editor state, never a sent request.
  const { request, transaction } = session.openLink(nodePath(field(question, 'answer')), 'SavePaymentParts');
  expect(field(request, 'value').children).toHaveLength(0);
  expect(hasEditorInput(request)).toBe(true);
  put(request, 'user', 7);
  put(request, 'value', [{ ...returnedPart, id: 0 }]);
  expect(QWHAScriptTaskData.validateValues(pkg, [request])).toEqual([]);
  session.calculations.applyLinkReturn(transaction, acceptedReply());
  expect(String(field(root, 'revision').value)).toBe('5');
  expect(String(field(partsOf(question).children[0], 'id').value)).toBe('100');
  if (business) {
    expect(field(field(field(root, 'documents'), 'documents').children[0], 'document')).toBe(business);
    expect(draftSnapshot(business)).toEqual(pending);
  }
  // The returned source view remains service-owned down to each amount cell.
  const amount = field(partsOf(question).children[0], 'amount');
  const changed = serializeEditorValue(amount);
  changed.value = '50000';
  expect(() => session.change(changed, nodePath(amount))).toThrow(/read-only/);
});

test('restoring a pending business retains its saved source revision rather than adopting a newer one', () => {
  const saved = editor('SaveIncomes', 5, [returnedPart]);
  pendingBusiness(saved.root, 'Pending business name');
  const snapshot = draftSnapshot(saved.root);
  const fresh = editor('SaveIncomes', 6, [{ ...returnedPart, id: 200 }]);
  pendingBusiness(fresh.root, 'Fresh business name');
  const restored = restoreDraft(fresh.root, snapshot, new CalculationSession(fresh.root).assignedDestinations());
  expect(String(field(restored, 'revision').value)).toBe('5');
  const question = field(restored, 'followups').children[0];
  expect(String(field(partsOf(question).children[0], 'id').value)).toBe('100');
  const business = field(field(field(restored, 'documents'), 'documents').children[0], 'document');
  expect(field(business, 'name').value).toBe('Pending business name');
  expect(String(field(fresh.root, 'revision').value)).toBe('6');
  expect(String(field(partsOf(fresh.question).children[0], 'id').value)).toBe('200');
});

test('compiled payroll import requires its printed identity label', () => {
  const payroll = pkg.types.find(type => type.type.NativeName === 'PayrollW2');
  expect(payroll).toBeDefined();
  const label = payroll.children.find(type => type.type.NativeName === 'label');
  expect(label).toBeDefined();
  expect(label.getAttribute('LLMDocField', pkg.types).Value).toBe('false');
  expect(label.getAttribute('Default', pkg.types)).toBeNull();
  expect(label.getAttribute('SizeMin', pkg.types).Value).toBe('1');
  expect(label.getAttribute('Key', pkg.types)).toBeDefined();
  const instruction = JSON.parse(payroll.getAttribute('LLMDocTrans', pkg.types).Value).instruction;
  expect(instruction).toContain('employee name and employer name in label');
  expect(instruction).toContain('Never infer a missing name');
});

test('compiled farm code choices use the human label as their title', () => {
  const display = JSON.parse(compiled.display);
  expect(display.rules).toEqual(expect.arrayContaining([
    expect.objectContaining({ select: '^<FarmCodeChoice>', title: { field: 'label' } }),
  ]));
  const choice = pkg.types.find(type => type.type.NativeName === 'FarmCodeChoice');
  expect(choice).toBeDefined();
  expect(choice.children.find(type => type.type.NativeName === 'label')).toBeDefined();
});
