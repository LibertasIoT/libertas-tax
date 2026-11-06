/* Run against ordinary Studio extraction with TAX_SCHEMA_PREFIX and LIBERTAS_STUDIO_PATH. */
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const Module = require('node:module');
const studio = process.env.LIBERTAS_STUDIO_PATH;
const prefix = process.env.TAX_SCHEMA_PREFIX;
assert(studio && prefix, 'Set LIBERTAS_STUDIO_PATH and TAX_SCHEMA_PREFIX to the Studio checkout and extracted schema prefix.');
const originalLoad = Module._load;
Module._load = function (name, ...args) {
    if (name === 'vscode') return { env: { language: 'en' }, window: {} };
    return originalLoad.call(this, name, ...args);
};
const { SchemaPackage, SchemaValue, SchemaType } = require(path.join(studio, 'out/schema/schema_value.js'));
const selectedFindings = new Set(process.argv.slice(2));
const pkg = new SchemaPackage(JSON.parse(fs.readFileSync(prefix + 'schema.json')), JSON.parse(fs.readFileSync(prefix + 'document.json')));
pkg.userDefaultContext = { currentUserId: 7n };

function auditCentsBounds() {
    const { parseMoneyAttribute } = require(path.join(studio, 'out/schema/format.js'));
    const cap = 100000000000n;
    // Each path is a reviewed aggregate, projection, or shared source/aggregate
    // carrier. No ReadOnly exemption: copied source facts still need bounds.
    // Shared carriers remain wide until source inputs have distinct types.
    const wideFields = {
        AnnualizedPeriod: 'agi itemized qbi additional self_employment social_wages',
        ArcherContributor: 'personal employer',
        BondPremiumInstrument: 'computed_deduction',
        CanceledDebtDocumentRow: 'preview',
        CareBenefitPeriod: 'received',
        CareBenefitPeriodInput: 'received',
        CareBenefitRow: 'box10',
        CreditIncomeSource: 'amount',
        EstateTrustNiitAdjustment: 'line_7',
        FarmTaxInstallments: 'applicable first remaining current',
        FarmingDocumentRow: 'preview',
        FederalAmount: 'amount money',
        FederalEducationExpenses: 'tuition assistance refunds',
        FederalEstateTrustPortfolioK1: 'gross_farming_fishing_income amt_box12a amt_box12b amt_box12c amt_box12d amt_box12e amt_box12f backup_withholding allocated_estimated_tax tax_exempt_interest minimum_tax_exclusion',
        FederalEstimate: 'tax refundable_credits payments refund balance',
        FederalHsa: 'personal employer',
        FederalSaver: 'taxpayer_contributions spouse_contributions',
        Form1041K1: 'gross_farming_fishing_income amt_box12a amt_box12b amt_box12c amt_box12d amt_box12e amt_box12f backup_withholding allocated_estimated_tax tax_exempt_interest minimum_tax_exclusion',
        GamblingDocumentRow: 'preview',
        GovernmentPaymentDocumentRow: 'preview',
        HealthEmployerBinding: 'accepted_amount',
        HealthEmployerSourceRow: 'reported',
        HealthFailedDepositFact: 'amount',
        HealthReturnDocumentChoice: 'gross earnings',
        IncomeRecordItem: 'preview',
        InjuredAmountAnswered: 'joint minimum maximum injured',
        InjuredAmountInitial: 'joint minimum maximum',
        InjuredAmountPage: 'joint minimum maximum',
        InjuredLtcPeriodPayment: 'amount',
        InjuredLtcPeriodPaymentInput: 'amount',
        InjuredPrintedRow: 'joint injured other',
        InjuredSpouse: 'joint_estimated student_loan_deduction known_child_credit_injured known_child_credit_other child_credit_cap_injured child_credit_cap_other estimated_share',
        InjuredSpouseInitial: 'joint_estimated student_loan_deduction known_child_credit_injured known_child_credit_other child_credit_cap_injured child_credit_cap_other estimated_share',
        InterestDividendDocumentRow: 'preview',
        InvestmentCapitalCap: 'short long amt_short amt_long',
        InvestmentInterest: 'paid carryover amt_carryover private_activity',
        InvestmentSaleDocumentRow: 'preview',
        IraContributor: 'form_8606_value',
        IraExcess: 'traditional_value',
        K1ActivityPartnerBenefits: 'medical_insurance pensions cash_distributions',
        K1DocumentRow: 'preview',
        K1EstateTrust: 'gross_farming_fishing_income',
        K1EstateTrustAmt: 'CodeAOnly.a Components.a Components.b Components.c Components.d Components.e Components.f',
        K1EstateTrustEstimatedTax: 'Allocated.amount',
        K1EstateTrustOtherBoxes: 'minimum_tax_exclusion backup_withholding tax_exempt_interest',
        K3Assets: 'us',
        K3CategoryAmounts: 'deductions interest reduction',
        K3Country: 'income qualified tax',
        LtcSourceChoice: 'gross',
        MilitaryMove: 'reimbursed',
        MinimumTaxGains: 'preferential combined regular_zero regular_fifteen',
        OtherIncomeDocumentRow: 'preview',
        PartnerActivityRecord: 'exempt nondeductible',
        PartnerBasisRecord: 'opening_liabilities ending_liabilities distributions',
        PartnerBenefits: 'medical_insurance pensions cash_distributions',
        PartnerIncomeTotal: 'amount',
        PartnerK1: 'reported_se health_guaranteed pensions deferred_compensation_tax crp_payments',
        PartnerSaleParts: 'ordinary collectibles unrecaptured',
        PartnerSeGross: 'Farm.gross Nonfarm.gross',
        PartnerSeparateTotals: 'interest dividends qualified_dividends royalties short long collectibles unrecaptured_partnership unrecaptured_fund cash_charity backup_withholding exempt_interest foreign_taxes',
        PartnershipK3Source: 'us_gross partner_deductions',
        PrintedSocialBenefitYearPage: 'received',
        RefundChoice: 'credit_capacity',
        RelatedFormSummary: 'amount',
        RetirementDocumentRow: 'preview',
        SaverContribution: 'amount',
        ScheduleK3: 'us_gross partner_deductions',
        ScorpIncome: 'amount',
        ScorpK1: 'withholding exempt_interest exempt distributions nondeductible cash_charity foreign_taxes',
        SelfEmploymentDocumentRow: 'preview',
        SocialBenefitYear: 'received',
        StudentCreditView: 'tuition scholarships insurance_reimbursements',
        TaxInterviewProtocol: 'IncomePreview.preview',
        TrumpExcessDocumentView: 'principal earnings',
        WageDocumentRow: 'preview',
        Withheld: 'w2 forms_1099 other total',
    };
    const exceptions = new Set(Object.entries(wideFields).flatMap(([owner, fields]) =>
        fields.split(' ').map(field => `${owner}.${field}`)));
    const names = new Set();
    for (const declared of pkg.types) {
        const name = declared.type.NativeName;
        assert(!names.has(name), `Ambiguous declaration name ${name}; qualify the audit paths`);
        names.add(name);
    }
    // Ordinary App discovery publishes the protocol's reachable types, not
    // internal stored drafts. Audit that real graph without inventing a root.
    assert(names.has('TaxInterviewProtocol'),
        'Missing TaxInterviewProtocol: expected the ordinary Tax App extraction');
    const failures = [];
    const check = (condition, message) => { if (!condition) failures.push(message); };
    const seenExceptions = new Set();
    let cents = 0;
    let bounded = 0;
    const bound = (node, name, declarationPath) => {
        const attribute = node.getAttribute(name, pkg.types);
        if (!attribute) return undefined;
        const text = String(attribute.Value);
        assert(/^-?\d+$/.test(text), `${declarationPath}: ${name} must be an exact integer`);
        return BigInt(text);
    };
    function visit(node, declarationPath, root = false) {
        const money = node.getAttribute('Money', pkg.types);
        if (money && parseMoneyAttribute(String(money.Value)).scale === 2) {
            cents++;
            check(node.getUnderlyingType(pkg.types) === 'Number', `${declarationPath}: cents must be a Number`);
            const minimum = bound(node, 'NumberMin', declarationPath);
            const maximum = bound(node, 'NumberMax', declarationPath);
            if (minimum !== undefined && maximum !== undefined)
                check(minimum <= maximum, `${declarationPath}: reversed bounds`);
            if (exceptions.has(declarationPath)) {
                seenExceptions.add(declarationPath);
                check(maximum === undefined || maximum > cap,
                    `${declarationPath}: stale wide-carrier exception; review its capacity`);
            } else {
                check(minimum !== undefined && minimum >= -cap &&
                    maximum !== undefined && maximum <= cap,
                `${declarationPath}: source cents require NumberMin/NumberMax within +/-${cap}`);
                bounded++;
            }
        }
        // Named targets are scanned as their own declarations. This checks the
        // reference's effective attributes without expanding its cloned graph.
        // Resolved Table children include inherited fields and local overrides.
        if (root || node.type.BuiltinType !== false) {
            for (const child of node.children)
                visit(child, `${declarationPath}.${child.type.NativeName}`);
        }
    }
    for (const declared of pkg.types) visit(declared, declared.type.NativeName, true);
    const unexported = [];
    for (const exception of exceptions) {
        if (!names.has(exception.split('.')[0])) unexported.push(exception);
        else check(seenExceptions.has(exception), `${exception}: stale exception path or missing cents metadata`);
    }
    assert(cents > 0 && bounded > 0, 'No bounded cents declarations were checked');
    assert(failures.length === 0, `CentsBounds metadata audit failed (${failures.length}):\n${failures.join('\n')}`);
    console.log(`CentsBounds metadata audit passed: ${bounded} bounded declarations, ${seenExceptions.size} exact wide carriers, ${cents} cents declarations`);
    if (unexported.length)
        console.log(`Not present in this schema (not audited): ${unexported.sort().join(', ')}`);
}

if (selectedFindings.has('CentsBounds')) {
    assert.equal(selectedFindings.size, 1, 'Run CentsBounds alone; it never loads value fixtures');
    auditCentsBounds();
} else {
const { QWHAScriptTaskData } = require(path.join(studio, 'out/front_end/QWHAScriptTaskData.js'));
const { AvroDecoder, decodeAvroFully, getEffectiveChildren } = require(path.join(studio, 'out/panels/shared/SchemaAvroValue.js'));
const { allowedFieldChoices } = require(path.join(studio, 'out/schema/runtimeConstrainedBy.js'));
const { evaluateValueValidationRules } = require(path.join(studio, 'out/schema/validationRules.js'));
const type = name => {
    const protocol = pkg.types.find(t => t.type.NativeName === 'TaxInterviewProtocol');
    const result = pkg.types.find(t => t.type.NativeName === name) || protocol?.children.find(t => t.type.NativeName === name);
    assert(result, `Missing type ${name}`);
    return result;
};

// Complete unrelated required leaves with legal synthetic answers. Each fixture
// supplies the values participating in its rule, including explicit nulls.
function build(declared, supplied, parent, depth = 0) {
    assert(depth < 64, 'Recursive fixture needs an explicit terminating value');
    let selected = declared;
    const kind = declared.getUnderlyingType(pkg.types);
    if (supplied === undefined) {
        const initial = declared.getAttribute('Fixed', pkg.types) || declared.getAttribute('Default', pkg.types);
        if (initial?.Value === '$null' || declared.isNullable(pkg.types)) supplied = null;
        else if (initial && kind !== 'Table' && kind !== 'List') supplied = initial.Value;
    }
    const nullHost = supplied === null;
    if (kind === 'Union' && !nullHost) {
        const fields = getEffectiveChildren(declared, pkg.types);
        const name = supplied && typeof supplied === 'object' ? Object.keys(supplied)[0] : supplied;
        selected = name === undefined ? fields[0] : fields.find(t => t.type.NativeName === name);
        assert(selected, `Unknown fixture union variant ${name}`);
        supplied = supplied && typeof supplied === 'object' ? supplied[name] : undefined;
    }
    const value = new SchemaValue(pkg, parent, selected, declared, null);
    if (nullHost && declared.isNullable(pkg.types)) {
        value.value = null;
        value.children = [];
        return value;
    }
    const actual = value.getQWHAType();
    const fields = getEffectiveChildren(selected, pkg.types);
    if (actual === 'Table') {
        value.value = '';
        value.children = fields.map(field => build(field, supplied?.[field.type.NativeName], value, depth + 1));
    } else if (actual === 'List') {
        value.value = '';
        const minimum = Number(value.getAttribute('SizeMin')?.Value || 0);
        value.children = (supplied || Array.from({ length: minimum })).map(item => build(fields[0], item, value, depth + 1));
    } else if (actual === 'Nil') value.value = '';
    else if (actual === 'Enumeration') value.value = value.resolveEnumValue(supplied ?? 0);
    else if (actual === 'Boolean') value.value = supplied === true || supplied === 'true' ? 'true' : 'false';
    else if (actual === 'String') value.value = supplied === undefined ? 'x'.repeat(Number(value.getAttribute('SizeMin')?.Value || 0)) : String(supplied);
    else if (actual === 'Bytes') value.value = supplied === undefined ? new Uint8Array() : supplied;
    else {
        const minimum = Number(value.getAttribute('NumberMin')?.Value || 0);
        const date = actual === 'DateOnly' || value.getAttribute('DateOnly');
        value.value = String(supplied ?? (date ? Math.max(20260101, minimum) : Math.max(0, minimum)));
    }
    return value;
}

let count = 0;
const files = fs.readdirSync(path.join(__dirname, 'fixtures')).filter(name => /^validation-audit-f\d+\.json$/.test(name)).sort();
for (const file of files) {
    const fixtures = JSON.parse(fs.readFileSync(path.join(__dirname, 'fixtures', file)));
    if (selectedFindings.size && !selectedFindings.has(fixtures.finding)) continue;
    for (const fixture of fixtures.cases) {
        const declared = type(fixture.type);
        const hostType = fixture.nullable ? new SchemaType({ ...declared.type, Attributes: [...(declared.type.Attributes || []), { Name: 'Nullable', Value: 'true' }] }) : declared;
        if (hostType !== declared) { hostType.children = declared.children; hostType.underlyingType = declared.underlyingType; }
        const node = build(hostType, fixture.value);
        const violation = fixture.rulesOnly ? evaluateValueValidationRules(node, hostType.getValidationRules(pkg.types)) : undefined;
        const errors = fixture.rulesOnly ? violation ? [violation.render(pkg.doc.StringResources, pkg.doc.Locale)] : [] : QWHAScriptTaskData.validateValues(pkg, [node]);
        const expected = fixture.expectedErrors || [];
        if (!expected.length) assert.deepEqual(errors, [], `${fixtures.finding}: ${fixture.name}`);
        else {
            const unmatched = [...errors];
            for (const message of expected) {
                const localized = pkg.doc.StringResources?.[message] || message;
                const index = unmatched.findIndex(error => error.includes(localized));
                assert(index >= 0, `${fixtures.finding}: ${fixture.name}: expected ${message}, got ${JSON.stringify(errors)}`);
                unmatched.splice(index, 1);
            }
            assert.deepEqual(unmatched, [], `${fixtures.finding}: ${fixture.name}: unexpected additional errors`);
        }
        for (const [fieldPath, expectedChoices] of Object.entries(fixture.expectedChoices || {})) {
            const field = node.resolveCopyPath(fieldPath);
            assert(field, `${fixtures.finding}: ${fixture.name}: missing ${fieldPath}`);
            const choices = allowedFieldChoices(field);
            assert(choices, `${fixtures.finding}: ${fixture.name}: missing allowed choices for ${fieldPath}`);
            assert.deepEqual([...choices].map(Number).sort(), [...expectedChoices].sort(), `${fixtures.finding}: ${fixture.name}: allowed choices for ${fieldPath}`);
        }
        count++;
    }
}
if (!selectedFindings.size || selectedFindings.has('F01')) {
    const initializers = JSON.parse(fs.readFileSync(path.join(__dirname, 'fixtures/validation-audit-f01-list-initializers.json')));
    for (const fixture of initializers) {
        let field = type(fixture.root);
        for (const index of fixture.indices) field = field.children[index];
        assert.equal(field.getUnderlyingType(pkg.types), 'List', fixture.path);
        const ordinary = new SchemaValue(pkg, undefined, field);
        assert(ordinary.value !== null && ordinary.value !== undefined, fixture.path);
        assert.deepEqual(ordinary.children, [], `${fixture.path} starts as an empty List`);
        count++;
    }
    const delivered = decodeAvroFully(new AvroDecoder(new Uint8Array(fs.readFileSync(path.join(__dirname, 'fixtures/validation-audit-save-hsa.avro')))), pkg, undefined, type('TaxInterviewProtocol'));
    assert.deepEqual(QWHAScriptTaskData.validateValues(pkg, [delivered]), [], 'Server-delivered HSA page with explicit null excess details');
    count++;
}
console.log(`Validation audit generated-schema cases passed: ${count}`);
}
