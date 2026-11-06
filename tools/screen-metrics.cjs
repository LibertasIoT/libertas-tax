#!/usr/bin/env node
/* Count declared page designs and schema fields using Studio's compiled schema and presentation owners. */
'use strict';

const fs = require('node:fs');
const path = require('node:path');
const { parseArgs } = require('node:util');

function loadStudio(directory) {
    const out = path.join(path.resolve(directory), 'out');
    return {
        ...require(path.join(out, 'schema/schema_value.js')),
        ...require(path.join(out, 'panels/shared/DisplayTemplatePresentation.js')),
        ...require(path.join(out, 'panels/shared/DisplayResolution.js')),
        ...require(path.join(out, 'panels/shared/EndpointLanding.js')),
        ...require(path.join(out, 'panels/shared/EditorPresentation.js')),
        ...require(path.join(out, 'panels/shared/SchemaAvroValue.js')),
    };
}

// A field is a member of a Table: a top-level type, an inline Union variant payload or an inline List
// element. Each declaration counts once, however often its type is used. A field is visible when some
// path from an endpoint protocol reaches it without passing a field, variant or referenced declaration
// marked Hidden or ExcludeUI.
function countFields(schema, protocolRoots) {
    const key = node => `${node.Namespace || ''}::${node.NativeName}`;
    const named = new Map(schema.Types.map(type => [key(type), type]));
    const target = node => node.BuiltinType === false ? named.get(`${node.Namespace || ''}::${node.Type}`) : undefined;
    const marked = (node, name) => (node?.Attributes || []).some(a => a.Name === name && a.Value === 'true');
    const hidden = node => [node, target(node)].some(n => marked(n, 'Hidden') || marked(n, 'ExcludeUI'));
    function walk(node, path, visit) {
        for (const child of node.Children || []) {
            const at = node.Type === 'List' ? `${path}[]` : node.Type === 'Union' ? `${path}<${child.NativeName}>` : `${path}.${child.NativeName}`;
            if (visit(child, at, node.Type === 'Table')) walk(child, at, visit);
        }
    }
    const all = new Set(), visible = new Set(), reached = new Set();
    schema.Types.forEach(type => walk(type, key(type), (child, at, isField) => { if (isField) all.add(at); return true; }));
    function reach(type) {
        if (reached.has(key(type))) return;
        reached.add(key(type));
        walk(type, key(type), (child, at, isField) => {
            if (hidden(child)) return false;
            if (isField) visible.add(at);
            const declared = target(child);
            if (declared) reach(declared);
            return true;
        });
    }
    protocolRoots.forEach(reach);
    for (const at of visible) if (!all.has(at)) throw Error(`Visible field outside the schema: ${at}`);
    return { schemaFields: all.size, visibleFields: visible.size };
}

function countScreens(artifacts, studio, packageName) {
    const decode = value => typeof value === 'string' ? JSON.parse(value) : value;
    const schema = decode(artifacts.schema);
    const document = decode(artifacts.document);
    const display = decode(artifacts.display);
    if (schema.AppDependencies?.length) throw Error('Supply a self-contained compiled App schema.');
    const pkg = new studio.SchemaPackage(schema, document, 1n);
    studio.attachDisplayTemplates(pkg, { primary: packageName,
        packages: new Map([[packageName, { template: display }]]) });
    const attr = (type, name) => type.getAttribute(name, pkg.types)?.Value;
    const marked = (type, name) => attr(type, name) === 'true';
    const hidden = type => marked(type, 'Hidden') || marked(type, 'ExcludeUI');
    const kind = type => type.getUnderlyingType(pkg.types);
    const names = value => String(value || '').split(',').map(name => name.trim()).filter(Boolean);
    const qualified = type => [type.type.Namespace, type.type.NativeName].filter(Boolean).join('::');
    function declaration(type) {
        const seen = new Set();
        while (type.type.BuiltinType === false && !seen.has(type)) {
            seen.add(type);
            const target = studio.findNamedSchemaType(pkg.types, type.type.Type, type.type.Namespace);
            if (!target) throw Error(`Unresolved schema type: ${type.type.Type}`);
            type = target;
        }
        if (pkg.types.includes(type)) return qualified(type);
        return type.parent ? `${declaration(type.parent)}.${type.type.NativeName || '[]'}` : qualified(type);
    }
    function value(type, parent, union = type) {
        type.resolveChildren(pkg.types);
        const node = new studio.SchemaValue(pkg, parent, type, union, null);
        node.typeInfo.underlyingType = kind(type);
        return node;
    }
    const itemPages = new Map(), protocolPages = new Map(), popups = new Set();
    const actions = new Set(), acknowledgements = new Set(), dialogs = new Set();
    const calculationRequests = new Set(), calculationReplies = new Set();
    function collectCalculations(node) {
        for (const a of node.Attributes || []) if (['CalculationRoot', 'CalculationRequest'].includes(a.Name)) {
            names(a.Value).forEach(name => calculationRequests.add(name));
        }
        (node.Children || []).forEach(collectCalculations);
    }
    schema.Types.forEach(collectCalculations);
    const protocols = new Set();
    for (const fn of pkg.functions) for (const parameter of fn.parameters) {
        const protocol = attr(parameter, 'ProtocolSchema');
        if (protocol) {
            const union = studio.findNamedSchemaType(pkg.types, protocol, parameter.type.Namespace);
            if (!union || kind(union) !== 'Union') throw Error(`Invalid protocol root: ${protocol}`);
            protocols.add(union);
        }
    }
    if (!protocols.size) throw Error('The schema contains no endpoint protocol.');

    function possiblePageLayout(node, mode) {
        const candidates = studio.matchDisplayCandidates(node);
        const pageCandidates = candidates.filter(c => ['summary', 'grid'].includes(c.members.layout));
        if (!pageCandidates.length) return false;
        // Only dimensions named by a matching rule can affect this decision.
        const dimensions = new Set(candidates.flatMap(c => Object.keys(c.when || {})).filter(k => k !== 'mode'));
        let devices = [{ ...studio.DEFAULT_DEVICE }];
        for (const dimension of dimensions) {
            const choices = studio.DEVICE_CONDITION_VALUES[dimension];
            if (choices) devices = devices.flatMap(device => choices.map(v => ({ ...device, [dimension]: v })));
        }
        // Data-conditioned summaries are potential surfaces, independent of an empty draft.
        return pageCandidates.some(target => devices.some(device => {
            const possible = candidates.map(c => ({ ...c, conditionHeld: c.condition ? c === target : true }));
            const resolved = studio.resolveDisplayPresentation(possible, { device, mode, kind: { list: true } });
            return ['summary', 'grid'].includes(resolved.layout);
        }));
    }
    function alternatives(type, seen = new Set()) {
        if (hidden(type) || kind(type) === 'Nil') return [];
        const identity = declaration(type);
        if (seen.has(identity)) return [];
        const next = new Set([...seen, identity]);
        type.resolveChildren(pkg.types);
        if (kind(type) === 'Union') return type.children.flatMap(child => alternatives(child, next));
        if (kind(type) === 'Table') {
            const fields = type.children.filter(child => !hidden(child));
            const asked = fields.filter(child => marked(child, 'AskOnAdd') && kind(child) === 'Union');
            const unions = fields.filter(child => kind(child) === 'Union');
            const primary = asked.length === 1 ? asked[0] : unions.length === 1 &&
                fields.every(child => child === unions[0] || marked(child, 'ReadOnly') || attr(child, 'Fixed') !== undefined)
                ? unions[0] : undefined;
            if (primary) return alternatives(primary, next);
        }
        const config = attr(type, 'LLMDocTrans');
        const printedForm = !!config && !!JSON.parse(config).doc_type;
        return [{ declaration: identity, printedForm }];
    }
    function requestInput(type, union) {
        const root = new studio.SchemaValue(pkg, undefined, type, union);
        const presented = studio.serializeEditorValue(root, { includeItemTemplate: false, omitExcludedSubtrees: true });
        const editable = studio.editableProtocolFields(presented);
        if (!editable.length) return 'action';
        if (presented.hierarchicalChoice) return 'popup';
        const visible = studio.visibleProtocolFields(presented);
        return editable.length === 1 && visible.length === 1 &&
            !['List', 'Table', 'Union'].includes(editable[0].node.qwhaType) ? 'popup' : 'page';
    }

    for (const protocol of protocols) {
        const variants = new Map();
        function addVariants(union, chain = new Set()) {
            if (chain.has(union)) return;
            const next = new Set([...chain, union]);
            union.resolveChildren(pkg.types);
            for (const variant of union.children) {
                if (attr(variant, 'Request') || attr(variant, 'Response') || attr(variant, 'SubscriptionRequest')) {
                    if (variants.has(variant.type.NativeName)) throw Error('Ambiguous protocol variant name.');
                    variants.set(variant.type.NativeName, { type: variant, union });
                } else {
                    variant.resolveChildren(pkg.types);
                    if (kind(variant) === 'Union') addVariants(variant, next);
                    else if (variant.children.length === 1 && kind(variant.children[0]) === 'Union') addVariants(variant.children[0], next);
                }
            }
        }
        addVariants(protocol);
        const prefix = declaration(protocol);
        const pending = [], visited = new Set(), inputKinds = new Map();
        function enqueue(name, entry = 'offered') {
            if (variants.has(name)) pending.push({ name, entry });
        }
        const defaults = [...variants].filter(([, v]) => marked(v.type, 'DefaultRequest') && !hidden(v.type));
        for (const [name] of defaults.length ? defaults : [...variants].filter(([name, v]) =>
            attr(v.type, 'Request') && !hidden(v.type) && !marked(v.type, 'LinkOnly') && !calculationRequests.has(name))) {
            enqueue(name, 'landing');
        }
        for (const name of calculationRequests) {
            const request = variants.get(name);
            if (request) names(attr(request.type, 'NextResponse')).forEach(reply => calculationReplies.add(`${prefix}.${reply}`));
        }
        function inspect(type, mode, page, parent, trail = new Set(), occurrence = '', union = type) {
            if ((hidden(type) && parent) || marked(type, 'Hidden')) return;
            const node = value(type, parent, union);
            const identity = declaration(type);
            if (trail.has(identity)) return;
            const next = new Set([...trail, identity]);
            const currentKind = kind(type);
            const locked = mode === 'view' || marked(type, 'ReadOnly') || attr(type, 'Fixed') !== undefined;
            const ownMode = locked ? 'view' : 'edit';
            const link = studio.declaredLinkValue(node, pkg.types);
            if (link) enqueue(link.target, link.stacked ? 'stacked' : 'link');
            if (currentKind === 'List' && possiblePageLayout(node, ownMode) && type.children[0]) {
                const member = type.children[0];
                const body = declaration(member);
                const memberInput = mode === 'edit' && attr(type, 'Fixed') === undefined &&
                    studio.hasEditorInput(new studio.SchemaValue(pkg, undefined, member, member));
                const pageMode = memberInput ? 'edit' : 'view';
                for (const alternative of alternatives(member)) {
                    const key = `${pageMode}:${body}:${alternative.declaration}`;
                    let entry = itemPages.get(key);
                    if (!entry) itemPages.set(key, entry = { key, mode: pageMode, member: body,
                        body: alternative.declaration, printedForm: alternative.printedForm, uses: [] });
                    const use = { page, path: occurrence };
                    if (!entry.uses.some(old => old.page === page && old.path === occurrence)) entry.uses.push(use);
                }
            }
            if (currentKind === 'Union') {
                for (const branch of type.children) inspect(branch, mode, page, parent, next,
                    `${occurrence}<${branch.type.NativeName}>`, type);
            } else {
                // ReadOnly on a List locks membership, not its member fields.
                const childMode = currentKind === 'List' && mode === 'edit' && attr(type, 'Fixed') === undefined ? mode : ownMode;
                for (const child of type.children) inspect(child, childMode, page, node, next,
                    occurrence + (currentKind === 'List' ? '[]' : `.${child.type.NativeName}`));
            }
        }
        while (pending.length) {
            const { name, entry } = pending.shift();
            const visitKey = `${name}:${entry}`;
            if (visited.has(visitKey)) continue;
            visited.add(visitKey);
            const { type, union } = variants.get(name);
            const id = `${prefix}.${name}`;
            if (marked(type, 'Hidden')) continue;
            if (!attr(type, 'Request') && !attr(type, 'SubscriptionRequest')) continue;
            if (entry === 'offered' && hidden(type)) continue;
            let input = inputKinds.get(name);
            if (!input) inputKinds.set(name, input = requestInput(type, union));
            const full = entry === 'delivered' || input === 'page';
            if (full) {
                protocolPages.set(`edit:${id}`, { name, protocol: prefix, mode: 'edit', label: type.doc?.Name || name });
                inspect(type, 'edit', id, undefined, new Set(), name, union);
                names(attr(type, 'PrevRequest')).forEach(previous => enqueue(previous, 'previous'));
            } else if (input === 'popup') popups.add(id);
            else actions.add(id);
            const replies = attr(type, 'NextResponse');
            const permitted = replies === undefined ? [...variants.keys()] : names(replies);
            for (const replyName of permitted) {
                const reply = variants.get(replyName);
                if (!reply) continue;
                if (attr(reply.type, 'Request')) { enqueue(replyName, 'delivered'); continue; }
                if (!attr(reply.type, 'Response')) continue;
                if (marked(reply.type, 'Error')) { dialogs.add(`${prefix}.${replyName}`); continue; }
                const workflow = attr(reply.type, 'WorkflowRequest');
                if (workflow) { names(workflow).forEach(target => enqueue(target, 'delivered')); continue; }
                const shown = ['landing', 'delivered', 'previous', 'stacked'].includes(entry) || attr(reply.type, 'NextRequest');
                if (!shown) { acknowledgements.add(`${prefix}.${replyName}`); continue; }
                const responseId = `${prefix}.${replyName}`;
                protocolPages.set(`view:${responseId}`, { name: replyName, protocol: prefix, mode: 'view', label: reply.type.doc?.Name || replyName });
                inspect(reply.type, 'view', responseId, undefined, new Set(), replyName, reply.union);
                names(attr(reply.type, 'NextRequest')).forEach(target => enqueue(target));
            }
        }
    }
    const pages = [...protocolPages.values()].sort((a, b) => a.mode.localeCompare(b.mode) || a.name.localeCompare(b.name));
    const items = [...itemPages.values()].sort((a, b) => a.key.localeCompare(b.key));
    const count = (array, mode) => array.filter(item => item.mode === mode).length;
    const fields = countFields(schema, [...protocols].map(protocol => protocol.type));
    return { formatVersion: 1, packageName, schemaRevision: schema.Revision, displaySha256: artifacts.displaySha256,
        counts: { protocolEditors: count(pages, 'edit'), protocolViews: count(pages, 'view'),
            listItemEditors: count(items, 'edit'), listItemViewers: count(items, 'view'),
            totalFullPages: pages.length + items.length, compactRequestPopups: popups.size,
            actionOnlyRequests: actions.size, errorDialogs: dialogs.size,
            calculationRequests: calculationRequests.size, calculationReplies: calculationReplies.size,
            acknowledgements: acknowledgements.size,
            printedFormEditors: items.filter(item => item.mode === 'edit' && item.printedForm).length, ...fields },
        protocolPages: pages, listItemPages: items, compactRequestPopups: [...popups].sort(),
        excluded: { actionOnlyRequests: [...actions].sort(), errorDialogs: [...dialogs].sort(),
            calculationRequests: [...calculationRequests].sort(), calculationReplies: [...calculationReplies].sort(),
            acknowledgements: [...acknowledgements].sort() } };
}

function markdown(result) {
    const c = result.counts;
    const n = value => value.toLocaleString('en-US');
    return `<!-- screen-metrics:start -->
## Screens

**${c.totalFullPages} distinct full-page designs** are declared by the reachable endpoint workflow and compiled Display Template, including **${c.listItemEditors} full list-child editor designs**. Of those list-child editors, **${c.printedFormEditors}** directly edit a printed-form declaration with a nonempty \`LLMDocTrans.doc_type\`.

| Screen kind | Designs |
| --- | ---: |
| Top-level Request editors | ${c.protocolEditors} |
| Top-level Response views | ${c.protocolViews} |
| Full list-child editors, including concrete form alternatives | ${c.listItemEditors} |
| Full list-child read-only viewers | ${c.listItemViewers} |
| **Total full pages** | **${c.totalFullPages}** |
| Compact Request popups, separate from full pages | ${c.compactRequestPopups} |

A screen is a reusable declaration design, not a record instance. Repeated children, Add versus Edit, conditional field values, tabs, folds, device widths and pane placement do not multiply it. A summary/grid List opens a complete child page; nested summary/grid Lists add their own pages. An inline Table, ordinary table row, Union chooser or form subsection adds no page by itself.

The counter follows DefaultRequest, NextRequest, NextResponse, WorkflowRequest, PrevRequest and declared links. WorkflowRequest carrier replies, calculation-only traffic, acknowledgement replies, confirmations and error dialogs are excluded. A server-delivered Request retains its full editor. The count describes possible declared surfaces across conditional branches; a particular return can visit fewer pages, and automatic forwarding can skip a declared landing.

List-child designs are deduplicated by resolved member declaration, its primary concrete Union alternative, and edit/view mode. An AskOnAdd document Union supplies concrete alternatives of the complete child editor; the generic wrapper and each form are not added together. Inline choices inside a form stay within that form's design. The JSON inventory records every page/path using each design.

The [page inventory](PAGE-INVENTORY.md) lists the top-level pages and full list-child designs.

### Fields

| Field count | Fields |
| --- | ---: |
| Fields in the compiled App schema | ${n(c.schemaFields)} |
| Fields a filer can see | ${n(c.visibleFields)} |

A field is a member of a Table: a top-level type, a Union variant payload, a protocol payload or a List element. Each declaration counts once, regardless of how often its type is used; List element descriptors and Union choice names add no fields. A filer can see a field when some path from an endpoint protocol variant reaches it without passing a field, variant or referenced declaration marked Hidden or ExcludeUI. Visibility is static: data-dependent presentation and Display Template layout do not change it.

Regenerate with the compiled Libertas Studio checkout:

\`\`\`sh
node tools/screen-metrics.cjs --studio /path/to/libertas/vscode --update --json /tmp/tax-screen-metrics.json
\`\`\`

The tool uses Studio's authoritative compiler, effective-attribute resolution, Request input classification and Display Template cascade. It reads saved source and updates this report section, including the field counts, and \`PAGE-INVENTORY.md\`. Schema content identity: \`${result.schemaRevision?.ContentSha256 || 'not supplied'}\`.
<!-- screen-metrics:end -->`;
}

function inventoryMarkdown(result) {
    const rows = result.protocolPages.map(p => `| \`${p.name}\` | ${p.mode === 'edit' ? 'Editor' : 'View'} | ${p.label.replaceAll('|', '\\|')} |`).join('\n');
    const groups = new Map();
    for (const item of result.listItemPages) {
        const key = `${item.mode}:${item.member}`;
        let group = groups.get(key);
        if (!group) groups.set(key, group = { mode: item.mode, member: item.member, count: 0, forms: 0 });
        group.count++; if (item.printedForm) group.forms++;
    }
    const itemRows = [...groups.values()].map(g => `| \`${g.member}\` | ${g.mode === 'edit' ? 'Editor' : 'Viewer'} | ${g.count} | ${g.forms} |`).join('\n');
    return `# Tax app page inventory

Declared reusable page designs from the compiled App schema and Display Template. [Code metrics](CODE-METRICS.md#screens) owns the totals, counting method and regeneration command.

**Schema content identity:** \`${result.schemaRevision?.ContentSha256 || 'not supplied'}\`.

## Top-level page inventory

| Protocol variant | Mode | Label |
| --- | --- | --- |
${rows}

## Full list-child page inventory

| Member declaration | Mode | Concrete designs | Printed-form designs |
| --- | --- | ---: | ---: |
${itemRows}
`;
}

function main() {
    const { values } = parseArgs({ options: { studio: { type: 'string' }, artifacts: { type: 'string' },
        project: { type: 'string', default: path.resolve(__dirname, '..') }, json: { type: 'string' },
        update: { type: 'boolean', default: false }, help: { type: 'boolean', short: 'h' } } });
    if (values.help) {
        console.log('Usage: node tools/screen-metrics.cjs --studio STUDIO_DIR [--project APP_DIR] [--artifacts COMPILED_JSON] [--json OUTPUT] [--update]');
        return;
    }
    if (!values.studio) throw Error('--studio must name a compiled Libertas Studio checkout.');
    const studioDirectory = path.resolve(values.studio);
    const project = path.resolve(values.project);
    const { generateSchemaAndDocument, latestRelevantSourceTimestamp } = require(path.join(studioDirectory, 'out/schema/schemaCompilation.js'));
    const { getPackageName } = require(path.join(studioDirectory, 'out/utils.js'));
    const artifacts = values.artifacts ? JSON.parse(fs.readFileSync(values.artifacts, 'utf8')) :
        generateSchemaAndDocument(project, 'src/lib.rs', { log() {} });
    const schema = typeof artifacts.schema === 'string' ? JSON.parse(artifacts.schema) : artifacts.schema;
    if (schema.Revision?.SourceTimestampUtcMs !== latestRelevantSourceTimestamp(project, true)) {
        throw Error('The supplied schema is stale for this project. Compile the saved source again.');
    }
    const studio = loadStudio(studioDirectory);
    const result = studio.withSchemaAttributeCache(() => countScreens(artifacts, studio, getPackageName(project)));
    if (values.json) fs.writeFileSync(values.json, JSON.stringify(result, null, 2) + '\n');
    if (values.update) {
        const file = path.join(project, 'CODE-METRICS.md');
        const text = fs.readFileSync(file, 'utf8');
        const block = markdown(result);
        const marker = /<!-- screen-metrics:start -->[\s\S]*?<!-- screen-metrics:end -->/;
        if (!marker.test(text) && !text.includes('\n## App categories')) throw Error('CODE-METRICS.md has no insertion point.');
        fs.writeFileSync(file, marker.test(text) ? text.replace(marker, () => block) : text.replace('\n## App categories', `\n${block}\n\n## App categories`));
        fs.writeFileSync(path.join(project, 'PAGE-INVENTORY.md'), inventoryMarkdown(result));
    }
    console.log(JSON.stringify(result.counts, null, 2));
}

module.exports = { loadStudio, countScreens, markdown, inventoryMarkdown };
if (require.main === module) {
    try { main(); } catch (error) { console.error(error.message); process.exitCode = 1; }
}
