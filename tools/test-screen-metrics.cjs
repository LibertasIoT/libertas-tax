'use strict';
const assert = require('node:assert/strict');
const test = require('node:test');
const { countScreens, loadStudio, markdown } = require('./screen-metrics.cjs');

if (!process.env.LIBERTAS_STUDIO_ROOT) throw Error('Set LIBERTAS_STUDIO_ROOT to a compiled Studio checkout.');
const studio = loadStudio(process.env.LIBERTAS_STUDIO_ROOT);
const a = (Name, Value = 'true') => ({ Name, Value });
const field = (NativeName, Type, Attributes = [], Children = [], BuiltinType = true) =>
    ({ ID: 0, NativeName, Type, BuiltinType, Attributes, Children, Namespace: '' });
const named = (name, type, attributes = []) => field(name, type, attributes, [], false);
const table = (name, children, attrs = []) => field(name, 'Table', attrs, children);
const list = (name, item, attrs = []) => field(name, 'List', attrs, [item]);
const request = (name, children, replies, attrs = []) => table(name, children,
    [a('Request'), ...(replies === undefined ? [] : [a('NextResponse', replies)]), ...attrs]);
const response = (name, children = [], attrs = []) => table(name, children, [a('Response'), ...attrs]);
function fixture(variants, types = [], rules = []) {
    const protocol = field('Protocol', 'Union', [], variants);
    return {
        schema: { AppDependencies: [], Types: [protocol, ...types], Attributes: [],
            Functions: [{ NativeName: 'app', Attributes: [], Parameters: [field('endpoint', 'Endpoint',
                [a('EndpointServer'), a('ProtocolSchema', 'Protocol')])] }] },
        document: { DocumentVersion: '0.0.2', Locale: 'en', DisplayName: '', Description: '', StringResources: {}, List: [], Types: [] },
        display: { DisplayVersion: 1, defaults: [], rules }
    };
}
function measure(value) {
    return studio.withSchemaAttributeCache(() => countScreens(value, studio, 'test-screens'));
}

test('a delivered scalar editor and terminal view count; the workflow carrier does not', () => {
    const result = measure(fixture([
        request('Enter', [], 'Begin', [a('DefaultRequest')]),
        response('Begin', [], [a('WorkflowRequest', 'Save')]),
        request('Save', [field('answer', 'Boolean')], 'Done'),
        response('Done'),
    ]));
    assert.equal(result.counts.protocolEditors, 1);
    assert.equal(result.counts.protocolViews, 1);
    assert.equal(result.counts.compactRequestPopups, 0);
    assert.deepEqual(result.protocolPages.map(p => p.name), ['Save', 'Done']);
});

test('offered scalar requests are popups; visible read-only companions require a full editor', () => {
    const result = measure(fixture([
        request('Enter', [], 'Home', [a('DefaultRequest')]),
        response('Home', [], [a('NextRequest', 'Choose,ChooseWithContext')]),
        request('Choose', [field('answer', 'Boolean')], 'Ack'),
        request('ChooseWithContext', [field('answer', 'Boolean'), field('context', 'String', [a('ReadOnly')])], 'Ack'),
        response('Ack', [field('accepted', 'Boolean', [a('ReadOnly')])]),
    ]));
    assert.equal(result.counts.protocolEditors, 1);
    assert.equal(result.counts.compactRequestPopups, 1);
    assert.deepEqual(result.compactRequestPopups, ['Protocol.Choose']);
    assert.deepEqual(result.excluded.acknowledgements, ['Protocol.Ack']);
});

test('calculation replies and Error dialogs do not become screens', () => {
    const result = measure(fixture([
        request('Enter', [], 'Home,Problem', [a('DefaultRequest')]),
        response('Home', [], [a('NextRequest', 'Edit')]),
        request('Edit', [field('amount', 'Number'), field('other', 'Number')], 'Home', [a('CalculationRoot', 'Calculate')]),
        request('Calculate', [field('amount', 'Number')], 'Calculated'),
        response('Calculated', [field('amount', 'Number')]),
        response('Problem', [field('message', 'String')], [a('Error')]),
    ]));
    assert.equal(result.counts.protocolViews, 1);
    assert.equal(result.counts.calculationRequests, 1);
    assert.equal(result.counts.errorDialogs, 1);
    assert(!result.protocolPages.some(p => ['Calculate', 'Calculated', 'Problem'].includes(p.name)));
});

test('a stacked link response is a viewer while a plain link acknowledgement is excluded', () => {
    const result = measure(fixture([
        request('Enter', [], 'Home', [a('DefaultRequest')]),
        response('Home', [field('view', 'String', [a('Link', 'Open,stacked=true')])]),
        request('Open', [], 'Details', [a('LinkOnly')]),
        response('Details', [field('text', 'String')]),
    ]));
    assert.deepEqual(result.protocolPages.map(p => p.name), ['Details', 'Home']);
});

test('concrete form editors replace the wrapper, deduplicate reuse, and include nested full child pages', () => {
    const doc = a('LLMDocTrans', JSON.stringify({ instruction: 'Read the printed amount.', doc_type: 'application/pdf',
        accept_multiple_inputs: false, validation: null }));
    const amount = () => field('amount', 'Number', [a('LLMDocField')]);
    const forms = field('Forms', 'Union', [], [named('A', 'PaperA'), named('AlsoA', 'PaperA'),
        named('B', 'PaperB'), field('None', 'Nil')]);
    const row = table('Row', [field('id', 'String', [a('Hidden')]), named('document', 'Forms', [a('AskOnAdd')])]);
    const types = [forms, row, table('PaperA', [amount(), list('parts', named('', 'Part'))], [doc]),
        table('PaperB', [amount()], [doc]), table('Part', [field('description', 'String')]),
        table('HiddenRow', [field('secret', 'String')])];
    const variants = [request('Enter', [], 'Home', [a('DefaultRequest')]),
        response('Home', [], [a('NextRequest', 'Edit,EditAgain')]),
        ...['Edit', 'EditAgain'].map(name => request(name, [list('rows', named('', 'Row')),
            list('hidden', named('', 'HiddenRow'), [a('Hidden')])], 'Home'))];
    const rules = [
        { select: '^<Edit>.rows', layout: 'summary', paths: ['document'], when: { mode: 'edit' } },
        { select: '^<EditAgain>.rows', layout: 'summary', paths: ['document'], when: { mode: 'edit' } },
        { select: '^<PaperA>.parts', layout: 'summary', paths: ['description'] },
        { select: '^<Edit>.hidden', layout: 'summary', paths: ['secret'] },
    ];
    const result = measure(fixture(variants, types, rules));
    assert.equal(result.counts.protocolEditors, 2);
    assert.equal(result.counts.listItemEditors, 3);
    assert.equal(result.counts.printedFormEditors, 2);
    assert.deepEqual(result.listItemPages.map(p => p.body).sort(), ['PaperA', 'PaperB', 'Part']);
    assert(result.listItemPages.filter(p => p.member === 'Row').every(p => p.uses.length === 2));
    assert(!result.listItemPages.some(p => p.member === 'HiddenRow'));
});

test('mode-specific cascade separates summary viewers from inline editor rows', () => {
    const result = measure(fixture([
        request('Enter', [], 'Home', [a('DefaultRequest')]),
        response('Home', [named('group', 'Group')], [a('NextRequest', 'Edit')]),
        request('Edit', [named('group', 'Group')], 'Home'),
    ], [table('Group', [list('items', named('', 'Item'))]), table('Item', [field('name', 'String')])], [
        { select: '^<Group>.items', layout: 'summary', paths: ['name'] },
        { select: '^<Group>.items', layout: 'rows', when: { mode: 'edit' } },
    ]));
    assert.equal(result.counts.listItemEditors, 0);
    assert.equal(result.counts.listItemViewers, 1);
    assert.equal(result.listItemPages[0].mode, 'view');
    assert.match(markdown(result), /3 distinct full-page designs/);
});

test('read-only List membership still permits a full editor for editable member fields', () => {
    const result = measure(fixture([
        request('Enter', [], 'Begin', [a('DefaultRequest')]),
        response('Begin', [], [a('WorkflowRequest', 'Edit')]),
        request('Edit', [list('items', named('', 'Item'), [a('ReadOnly')])], 'Done'),
        response('Done'),
    ], [table('Item', [field('name', 'String')])], [
        { select: '^<Edit>.items', layout: 'summary', paths: ['name'], when: { mode: 'view' } },
    ]));
    assert.equal(result.counts.listItemEditors, 1);
    assert.equal(result.counts.listItemViewers, 0);
});

test('fields count each declaration once; a hidden step hides what only it reaches', () => {
    const choice = field('choice', 'Union', [], [table('Open', [field('x', 'Boolean')], [a('Discriminator', '0')]),
        table('Closed', [field('y', 'Boolean')], [a('Hidden'), a('Discriminator', '1')])]);
    const result = measure(fixture([
        request('Enter', [], 'Home', [a('DefaultRequest')]),
        response('Home', [field('shown', 'Boolean', [a('ReadOnly')]), named('detail', 'Detail'),
            list('rows', named('', 'Detail')), named('secret', 'Secret', [a('Hidden')]), choice]),
    ], [table('Detail', [field('amount', 'Number')]), table('Secret', [field('note', 'String')])]));
    // Home: shown, detail, rows, secret, choice, x, y; Detail.amount; Secret.note.
    assert.equal(result.counts.schemaFields, 9);
    // Not visible: secret and Secret.note behind it, y in the Hidden variant.
    assert.equal(result.counts.visibleFields, 6);
    assert.match(markdown(result), /Fields a filer can see \| 6 \|/);
});
