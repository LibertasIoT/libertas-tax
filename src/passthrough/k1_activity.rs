// K1RES actual statement/source partitions. One parent retains one entity basis.
// This module owns source input/identity/validation, not basis/risk/PAL/179/credit
// arithmetic. The ordinary child adapter supplies the bounded partnership
// financial-owner feed; other multi-source cases retain their named boundary.
// No raw-box fallback or synthetic allowed result is provided.
// Source coverage: actual component/group/QBI/property identities use normal
// global allocation and pre-edit authorization. Original signed controls and
// independently unknown retained stages/personal facts survive normal Save.
// The canonical credit Page and source payload owners validate reported facts;
// no basis/risk/PAL/179/credit allowance is derived by this adapter.
// Lifecycle controls include k1res_nullable_entity_basis_keeps_current_facts_and_checks_known_values_and_completion,
// k1res_fuel_availability_retains_known_payment_and_nullable_basis_through_both_writers,
// and k1res_canonical_credit_pages_preserve_printed_lines_and_confirmation_through_save_restart.
// See k1_activity_ordinary.rs for its calculation scope and evidence. Remaining
// characters need their actual financial-owner feeds; repayment, prior
// nondeductible and credit-bond boundaries remain separate.
use crate::kernel::ratio::exact;
use crate::validation::ValidationIssue;
use alloc::{string::String, vec::Vec};
use libertas_macros::*;

fn in_source_field(issue: ValidationIssue, field: &'static str) -> ValidationIssue {
    if issue.field.is_some() { issue.in_field(field) } else { issue.at_field(field) }
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1Section179SaleSource {
    /// Actual accepted physical sale of this same K-1.
    #[libertas_number(min = 1)]
    pub sale: i64,
    /// Actual accepted component source of this same K-1.
    #[libertas_number(min = 1)]
    pub component: i64,
}
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1Section179SaleSourcePage {
    /// Actual accepted physical sale of this same K-1.
    #[libertas_number(min = 1)]
    #[libertas_enum_source("^<K1ComponentDetailsPage>.choices.sales")]
    pub sale: i64,
    /// Actual accepted component source of this same K-1.
    #[libertas_number(min = 1)]
    #[libertas_enum_source("^<K1ComponentDetailsPage>.choices.components")]
    pub component: i64,
}
impl K1Section179SaleSourcePage {
    fn record(&self) -> K1Section179SaleSource {
        K1Section179SaleSource { sale: self.sale, component: self.component }
    }
    fn from_record(value: &K1Section179SaleSource) -> Self {
        Self { sale: value.sale, component: value.component }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1InvestmentPropertySource {
    /// Actual accepted physical property of this same K-1.
    #[libertas_number(min = 1)]
    pub property: i64,
    /// Actual accepted component source of this same K-1.
    #[libertas_number(min = 1)]
    pub component: i64,
}
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1InvestmentPropertySourcePage {
    /// Actual accepted physical property of this same K-1.
    #[libertas_number(min = 1)]
    #[libertas_enum_source("^<K1ComponentDetailsPage>.choices.properties")]
    pub property: i64,
    /// Actual accepted component source of this same K-1.
    #[libertas_number(min = 1)]
    #[libertas_enum_source("^<K1ComponentDetailsPage>.choices.components")]
    pub component: i64,
}
impl K1InvestmentPropertySourcePage {
    fn record(&self) -> K1InvestmentPropertySource {
        K1InvestmentPropertySource { property: self.property, component: self.component }
    }
    fn from_record(value: &K1InvestmentPropertySource) -> Self {
        Self { property: value.property, component: value.component }
    }
}

pub(crate) fn section179_sales(income: &crate::FederalIncome) -> &[crate::K1Section179Sale] {
    match income {
        crate::FederalIncome::PartnershipActivities { data } => &data.entity.section179_sales,
        crate::FederalIncome::ScorpActivities { data } => &data.entity.section179_sales,
        _ => &[],
    }
}
fn source_row_issue(
    rows: impl Iterator<Item = (i64, i64)>,
    source_field: &'static str,
    list: &'static str,
    duplicate_key: &'static str,
) -> Option<ValidationIssue> {
    let mut seen = alloc::collections::BTreeSet::new();
    for (index, (source, component)) in rows.enumerate() {
        for (field, id) in [(source_field, source), ("component", component)] {
            if id <= 0 {
                return Some(
                    ValidationIssue::new("K1_ACTIVITY_SOURCE_REFERENCE_NOT_ACCEPTED")
                        .at_field(field)
                        .in_list(list, index),
                );
            }
        }
        if !seen.insert(source) {
            return Some(
                ValidationIssue::new(duplicate_key).at_field(source_field).in_list(list, index),
            );
        }
    }
    None
}
fn sale_source_issue(rows: Option<&[K1Section179SaleSource]>) -> Option<ValidationIssue> {
    source_row_issue(
        rows?.iter().map(|row| (row.sale, row.component)),
        "sale",
        "section179_sale_sources",
        "K1_ACTIVITY_SALE_SOURCE_TWICE",
    )
}
fn sale_identity_issue(
    value: &K1ActivitySources,
    sales: &[crate::K1Section179Sale],
    accepted: Option<&K1ActivitySources>,
    accepted_sales: &[crate::K1Section179Sale],
) -> Option<ValidationIssue> {
    for (index, row) in value.section179_sale_sources.iter().flatten().enumerate() {
        for (field, valid) in [
            (
                "sale",
                row.sale > 0
                    && sales.iter().any(|v| v.id == row.sale)
                    && accepted_sales.iter().any(|v| v.id == row.sale),
            ),
            (
                "component",
                row.component > 0
                    && value.components.iter().any(|v| v.id == row.component)
                    && accepted.is_some_and(|v| v.components.iter().any(|v| v.id == row.component)),
            ),
        ] {
            if !valid {
                return Some(
                    ValidationIssue::new("K1_ACTIVITY_SOURCE_REFERENCE_NOT_ACCEPTED")
                        .at_field(field)
                        .in_list("section179_sale_sources", index),
                );
            }
        }
    }
    None
}

fn investment_properties(income: &crate::FederalIncome) -> &[crate::K1InvestmentProperty] {
    match income {
        crate::FederalIncome::PartnershipActivities { data } => &data.entity.investment_property,
        crate::FederalIncome::ScorpActivities { data } => &data.entity.investment_property,
        _ => &[],
    }
}
fn property_source_issue(rows: Option<&[K1InvestmentPropertySource]>) -> Option<ValidationIssue> {
    source_row_issue(
        rows?.iter().map(|row| (row.property, row.component)),
        "property",
        "investment_property_sources",
        "K1_ACTIVITY_PROPERTY_SOURCE_TWICE",
    )
}
fn property_identity_issue(
    value: &K1ActivitySources,
    properties: &[crate::K1InvestmentProperty],
    accepted: Option<&K1ActivitySources>,
    accepted_properties: &[crate::K1InvestmentProperty],
) -> Option<ValidationIssue> {
    for (index, row) in value.investment_property_sources.iter().flatten().enumerate() {
        for (field, valid) in [
            (
                "property",
                row.property > 0
                    && properties.iter().any(|v| v.id() == row.property)
                    && accepted_properties.iter().any(|v| v.id() == row.property),
            ),
            (
                "component",
                row.component > 0
                    && value.components.iter().any(|v| v.id == row.component)
                    && accepted.is_some_and(|v| v.components.iter().any(|v| v.id == row.component)),
            ),
        ] {
            if !valid {
                return Some(
                    ValidationIssue::new("K1_ACTIVITY_SOURCE_REFERENCE_NOT_ACCEPTED")
                        .at_field(field)
                        .in_list("investment_property_sources", index),
                );
            }
        }
    }
    None
}

impl K1ActivityChoiceContext {
    fn from_source_properties(
        source: Option<&K1ActivitySources>,
        properties: &[crate::K1InvestmentProperty],
        sales: &[crate::K1Section179Sale],
    ) -> Self {
        let mut choices = Self::from_sources(source);
        choices.properties = properties
            .iter()
            .filter(|v| v.id() > 0)
            .map(|property| {
                let (key, amount, year) = match property {
                    crate::K1InvestmentProperty::RehabilitationRental {
                        expenditures,
                        placed_year,
                        ..
                    }
                    | crate::K1InvestmentProperty::RehabilitationOther {
                        expenditures,
                        placed_year,
                        ..
                    } => ("LABEL_K1_PROPERTY_REHABILITATION", *expenditures, *placed_year),
                    crate::K1InvestmentProperty::Energy { basis, .. } => {
                        ("LABEL_K1_PROPERTY_ENERGY", *basis, 2026)
                    }
                    crate::K1InvestmentProperty::CleanElectricity { basis, .. } => {
                        ("LABEL_K1_PROPERTY_CLEAN_ELECTRICITY", *basis, 2026)
                    }
                };
                K1ActivitySourceChoice {
                    id: property.id(),
                    label: crate::Message::new(
                        key,
                        alloc::vec![
                            crate::MessageArg::usd_cents(amount),
                            crate::MessageArg::Number(year as i64),
                            crate::MessageArg::Number(property.id())
                        ],
                    )
                    .encode(),
                }
            })
            .collect();
        choices.sales = sales
            .iter()
            .filter(|sale| sale.id > 0)
            .map(|sale| K1ActivitySourceChoice {
                id: sale.id,
                label: crate::Message::new(
                    "LABEL_K1_ACTIVITY_SOURCE",
                    alloc::vec![crate::MessageArg::Text(sale.description.clone())],
                )
                .encode(),
            })
            .collect();
        choices
    }
}
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1ActivitySources {
    /// Actual whole-sale source associations; no computed allowance or duplicated monetary values.
    /// [NullText]
    /// The sale-to-activity source attribution is unavailable.
    /// [SetText]
    /// The actual sale sources belong to these activities
    pub section179_sale_sources: Option<Vec<K1Section179SaleSource>>,
    /// Actual statement activities.
    pub components: Vec<K1StatementActivity>,
    /// Actual separately identified at-risk groups.
    pub risk_groups: Vec<K1RiskGroup>,
    /// Actual separately identified passive groups.
    pub passive_groups: Vec<K1PassiveGroup>,
    /// Actual section199A businesses or indivisible reported aggregates.
    pub qbi_blocks: Vec<K1QbiBlock>,
    /// Amounts the source establishes outside all listed activities.
    /// [NullText]
    /// The entity-only source attribution is unavailable.
    /// [SetText]
    /// The entity-only source amounts are
    pub entity_only: Option<K1EntityOnlyAmounts>,
    /// Actual alternative-minimum-tax entity-only source; absence is never copied as zero.
    pub alternative_entity_only: K1EntityOnlyAmt,
    /// Actual whole-property source associations; no credit amount or computed allowance.
    /// [NullText]
    /// The property-to-activity source attribution is unavailable.
    /// [SetText]
    /// The actual property sources belong to these activities
    pub investment_property_sources: Option<Vec<K1InvestmentPropertySource>>,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum K1StatementKind {
    /// Box1 trade or business.
    TradeBusiness,
    /// Box2 rental real estate.
    RentalRealEstate,
    /// Box3 other rental.
    OtherRental,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum K1ActivityPersonalSource {
    /// This activity has no partnership personal-service source.
    NotApplicable,
    /// Actual partnership records; independent of passive participation and box14.
    Partnership {
        /// What produces this activity's income.
        /// [NullText]
        /// The income source is unresolved.
        /// [SetText]
        /// This activity's income comes from
        income_source: Option<crate::BusinessIncomeSource>,
        /// Allocable services-only gross income, excluding separately entered guaranteed payments.
        /// [NullText]
        /// The gross personal-service source is unavailable.
        /// [SetText]
        /// The actual gross personal-service income is
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        gross_personal_services: Option<i64>,
    },
    /// The personal-service source records are unavailable.
    Unavailable,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1CurrentSourceParts {
    /// Actual documented positive current income of this source character, before owner limits.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub income: i64,
    /// Actual documented current loss of this source character, as a positive magnitude.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub loss: i64,
}
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1QualifiedCurrentControls {
    /// Ordinary-character source parts with their original code/character provenance.
    /// [NullText]
    /// The ordinary source parts or their character support are unavailable.
    /// [SetText]
    /// The documented ordinary source parts are
    pub ordinary: Option<K1CurrentSourceParts>,
    /// Property-character source parts with their original code/character provenance.
    /// [NullText]
    /// The property source parts or their character support are unavailable.
    /// [SetText]
    /// The documented property source parts are
    pub property: Option<K1CurrentSourceParts>,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1StatementActivity {
    /// Actual accepted source identity; new rows start at zero.
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    #[libertas_number(min = 0)]
    pub id: i64,
    /// Name on the actual source statement.
    #[libertas_size(min = 1)]
    pub name: String,
    /// Actual statement activity character.
    pub kind: K1StatementKind,
    /// Actual personal-service source facts for this activity, separate from passive and SE classification.
    pub personal_services: K1ActivityPersonalSource,
    /// Actual regular-tax self-employment source, independent of AMT amounts.
    pub self_employment: K1StatementSe,
    /// Actual current positive-income and loss controls, before owner limitations.
    pub qualified_current_controls: K1QualifiedCurrentControls,
    /// This activity's regular-tax monetary source.
    /// [NullText]
    /// The source partition is unavailable.
    /// [SetText]
    /// Use these documented source parts
    pub regular: Option<K1StatementAmounts>,
    /// Actual AMT source fact; never inferred from absence.
    pub alternative: K1StatementAmt,
    /// Actual accepted same-parent at-risk source.
    /// [NullText]
    /// That source relationship is unavailable.
    /// [SetText]
    /// The actual source is
    #[libertas_number(min = 1)]
    pub risk_group: Option<i64>,
    /// Actual accepted same-parent passive source.
    /// [NullText]
    /// That source relationship is unavailable.
    /// [SetText]
    /// The actual source is
    #[libertas_number(min = 1)]
    pub passive_group: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
// Complete source payloads stay inline, matching the normal App wire model.
#[allow(clippy::large_enum_variant)]
pub enum K1StatementAmt {
    /// The source establishes identical regular and AMT monetary parts.
    SameAsRegular,
    /// Actual different AMT source records.
    Separate { amounts: K1StatementAmounts },
    /// The AMT source partition is unavailable.
    Unavailable,
}

#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct K1RetainedLossCarry {
    /// Actual loss retained only by basis limits.
    /// [NullText]
    /// This retained source amount is unavailable.
    /// [SetText]
    /// The actual retained source amount is
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub basis: Option<i64>,
    /// Actual loss retained only by at-risk limits; already passed basis.
    /// [NullText]
    /// This retained source amount is unavailable.
    /// [SetText]
    /// The actual retained source amount is
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub risk: Option<i64>,
}

#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct K1RetainedSeHistory {
    /// Actual retained basis-limited SE-character amount.
    /// [NullText]
    /// This retained source amount is unavailable.
    /// [SetText]
    /// The actual retained source amount is
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub basis: Option<i64>,
    /// Actual retained at-risk-limited SE-character amount.
    /// [NullText]
    /// This retained source amount is unavailable.
    /// [SetText]
    /// The actual retained source amount is
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub risk: Option<i64>,
    /// Actual retained passive-limited SE-character amount.
    /// [NullText]
    /// This retained source amount is unavailable.
    /// [SetText]
    /// The actual retained source amount is
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub passive: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1StatementProperty {
    /// Actual current net section1231 source, before owner limits.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = -9223372036854775807)]
    pub net: i64,
    /// Actual current business-property unrecaptured1250 component; not additional income.
    /// It may exceed the net when other actual property losses offset gains.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub unrecaptured: i64,
    /// Actual prior property loss suspended only by basis.
    /// [NullText]
    /// This retained source amount is unavailable.
    /// [SetText]
    /// The actual retained source amount is
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub basis_loss: Option<i64>,
    /// Actual prior property loss suspended only by at-risk limits.
    /// [NullText]
    /// This retained source amount is unavailable.
    /// [SetText]
    /// The actual retained source amount is
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub risk_loss: Option<i64>,
    /// Actual prior property loss suspended only by passive limits.
    /// [NullText]
    /// This retained source amount is unavailable.
    /// [SetText]
    /// The actual retained source amount is
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub passive_loss: Option<i64>,
}

#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct K1StatementPartnerCapitalCarry {
    /// Actual retained short-term source, each limitation stage independent.
    pub short: K1RetainedLossCarry,
    /// Actual retained long-term source, each limitation stage independent.
    pub long: K1RetainedLossCarry,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1StatementAmounts {
    /// Signed source result of the activity's declared box, excluding separate items.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = -9223372036854775807)]
    pub operating: i64,
    /// Actual tax-exempt interest attributed to this source.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub exempt_interest: i64,
    /// Other actual tax-exempt income attributed to this source.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub exempt_other: i64,
    /// Actual noncapital nondeductible expense attributed to this source.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub nondeductible: i64,
    /// Documented portfolio source parts; not repeated in operating income.
    pub portfolio: K1StatementPortfolio,
    /// Actual operating loss carried at basis and at-risk stages.
    pub operating_carry: K1RetainedLossCarry,
    /// Actual operating loss carried only by passive limits.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    /// [NullText]
    /// This retained source amount is unavailable.
    /// [SetText]
    /// The actual retained source amount is
    pub passive_operating: Option<i64>,
    /// Actual business-property source and distinct stage balances.
    /// [NullText]
    /// This source has no business-property record.
    /// [SetText]
    /// This source's business-property record is
    pub property: Option<K1StatementProperty>,
    /// Current section179 source amount.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub section179: i64,
    /// Prior section179 source amount suspended only by basis.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    /// [NullText]
    /// This retained source amount is unavailable.
    /// [SetText]
    /// The actual retained source amount is
    pub section179_basis_carry: Option<i64>,
    /// Prior section179 business-income carry; not deducted from basis again.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    /// [NullText]
    /// This retained source amount is unavailable.
    /// [SetText]
    /// The actual retained source amount is
    pub section179_business_carry: Option<i64>,
    /// Part of that business-income carry for actual sold-property sources.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    /// [NullText]
    /// This retained source amount is unavailable.
    /// [SetText]
    /// The actual retained source amount is
    pub section179_sold_carry: Option<i64>,
    /// Actual cash charity source amount.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub cash_charity: i64,
    /// Actual charity balance suspended only by basis.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    /// [NullText]
    /// This retained source amount is unavailable.
    /// [SetText]
    /// The actual retained source amount is
    pub charity_basis_carry: Option<i64>,
    /// Actual foreign tax source amount.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub foreign_taxes: i64,
    /// Actual foreign tax balance suspended only by basis.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    /// [NullText]
    /// This retained source amount is unavailable.
    /// [SetText]
    /// The actual retained source amount is
    pub foreign_basis_carry: Option<i64>,
    /// Exact existing entity-matched income/deduction source codes.
    pub items: K1StatementItems,
    /// Actual capital stage/source records.
    pub capital_carry: K1StatementCapitalCarry,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1StatementPortfolio {
    /// Taxable portfolio interest.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub interest: i64,
    /// Ordinary dividends including their qualified part.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub dividends: i64,
    /// Qualified dividend portion.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub qualified_dividends: i64,
    /// Portfolio royalty source.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub royalties: i64,
    /// Signed short-term portfolio source.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = -9223372036854775807)]
    pub short: i64,
    /// Signed long-term portfolio source.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = -9223372036854775807)]
    pub long: i64,
    /// Signed collectible component, not additional capital income.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = -9223372036854775807)]
    pub collectibles: i64,
    /// Unrecaptured gain from partnership-interest sales.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub unrecaptured_partnership: i64,
    /// Unrecaptured gain from funds/estates/trusts.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub unrecaptured_fund: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum K1StatementItems {
    Partnership { income: Vec<crate::PartnerIncome>, deductions: Vec<crate::PartnerDeduction> },
    Scorp { income: Vec<crate::ScorpIncome>, deductions: Vec<crate::ScorpDeduction> },
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum K1StatementCapitalCarry {
    Partnership {
        amounts: K1StatementPartnerCapitalCarry,
    },
    Scorp {
        amounts: K1StatementScorpCapitalCarry,
    },
    /// The capital carry/source attribution is unavailable.
    Unavailable,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum K1ComponentQualified {
    Partnership {
        /// Actual current ordinary qualified source portion, not final deduction.
        #[libertas_money("USD", 2)]
        // Encoding safety: signs() exposes a positive native magnitude.
        #[libertas_number(min = -9223372036854775807)]
        current_income: i64,
        /// Actual signed current property qualified source portion.
        #[libertas_money("USD", 2)]
        // Encoding safety: signs() exposes a positive native magnitude.
        #[libertas_number(min = -9223372036854775807)]
        current_property: i64,
        /// Actual existing qualified histories/gross allocation, not another loss ledger.
        records: crate::partnership_qbi::PartnerQbiRecords,
    },
    Scorp {
        /// Actual current qualified income portion, not final deduction.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        current_income: i64,
        ordinary: crate::ScorpLossQbi,
        property: crate::ScorpPropertyQbi,
    },
    /// Source qualification/attribution is unavailable; mandatory income is retained.
    Unavailable,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum K1StatementSe {
    /// This activity has no self-employment source.
    NotApplicable,
    /// Actual partner source subsets and retained character.
    Partnership { source: K1PartnerSeSource },
    /// The actual source attribution is unavailable.
    Unavailable,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1RiskGroup {
    /// Actual accepted source identity; new rows start at zero.
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    #[libertas_number(min = 0)]
    pub id: i64,
    /// Name on the actual source statement.
    #[libertas_size(min = 1)]
    pub name: String,
    /// Actual existing tax-system history/financing source.
    pub source: K1RiskSource,
    /// Actual cash investment source parts, not calculated capacity.
    /// [NullText]
    /// The cash investment source attribution is unavailable.
    /// [SetText]
    /// The documented cash investment source parts are
    pub investment_parts: Option<K1RiskInvestmentParts>,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum K1RiskSource {
    Partnership {
        regular: crate::PartnerRiskRecord,
        /// Actual alternative-minimum-tax financing/history worksheet.
        /// [NullText]
        /// The alternative worksheet is unavailable.
        /// [SetText]
        /// The actual alternative worksheet is
        alternative: Option<crate::PartnerRiskRecord>,
    },
    Scorp {
        regular: crate::ScorpRisk,
        /// Actual alternative-minimum-tax financing/history worksheet.
        /// [NullText]
        /// The alternative worksheet is unavailable.
        /// [SetText]
        /// The actual alternative worksheet is
        alternative: Option<crate::ScorpRisk>,
    },
    /// Required history/financing source is unavailable.
    Unavailable,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1RiskInvestmentParts {
    /// Actual cash contributions assigned by source.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub contributions: i64,
    /// Actual cash withdrawals assigned by source.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub distributions: i64,
    /// Actual loan investment assigned by source.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub loan_advances: i64,
    /// Actual principal returns assigned by source.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub loan_repayments: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1PassiveGroup {
    /// Actual accepted source identity; new rows start at zero.
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    #[libertas_number(min = 0)]
    pub id: i64,
    /// Name on the actual source statement.
    #[libertas_size(min = 1)]
    pub name: String,
    /// Existing established participation source.
    pub participation: K1ActivityParticipation,
    /// The actual existing qualifying complete-disposition facts are established.
    /// Check the sale agreement and retained return for the complete taxable disposition.
    pub disposed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum K1ActivityParticipation {
    Partnership {
        facts: crate::PartnerParticipation,
        trade_kind: crate::ScorpTradeKind,
    },
    Scorp {
        facts: crate::ScorpParticipation,
        trade_kind: crate::ScorpTradeKind,
    },
    Rental {
        facts: crate::K1RentalParticipation,
    },
    /// Participation/classification source is unavailable.
    Unavailable,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1QbiBlock {
    /// Actual accepted source identity; new rows start at zero.
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    #[libertas_number(min = 0)]
    pub id: i64,
    /// Name on the actual source statement.
    #[libertas_size(min = 1)]
    pub name: String,
    /// The entity statement actually reports this as one indivisible aggregate.
    pub reported_aggregate: bool,
    /// Actual existing reported statement/retained source; no final deduction.
    pub statement: K1ReportedQbiStatement,
    /// Distinct trades/businesses actually identified by the source owner.
    /// [NullText]
    /// The source does not establish the constituent businesses.
    /// [SetText]
    /// The source identifies these businesses
    pub business_names: Option<Vec<K1QbiBusinessName>>,
    /// Actual component-attributed current and retained qualified-source records.
    /// [NullText]
    /// The source allocation is unavailable.
    /// [SetText]
    /// The source establishes these allocations
    pub allocations: Option<Vec<K1QbiAllocation>>,
    /// Individual aggregation of this actual QBI source block.
    /// [NullText]
    /// No individual group is selected for this block.
    /// [SetText]
    /// This block is in individual group
    #[libertas_size(min = 1)]
    pub qbi_aggregation_group: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1QbiAllocation {
    /// Actual accepted same-parent component source.
    #[libertas_number(min = 1)]
    pub component: i64,
    /// Existing qualified-source subsets for this block, with their original stages and years.
    pub qualified: K1ComponentQualified,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1QbiAllocationPage {
    /// Actual accepted same-parent component source.
    #[libertas_number(min = 1)]
    #[libertas_enum_source("^<K1ComponentDetailsPage>.choices.components")]
    pub component: i64,
    /// Existing qualified-source subsets for this block, with their original stages and years.
    pub qualified: K1ComponentQualified,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1QbiBlockPage {
    /// Actual accepted source identity; new rows start at zero.
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    #[libertas_number(min = 0)]
    pub id: i64,
    /// Name on the actual source statement.
    #[libertas_size(min = 1)]
    pub name: String,
    /// The entity statement actually reports this as one indivisible aggregate.
    pub reported_aggregate: bool,
    /// Actual existing reported statement/retained source; no final deduction.
    pub statement: K1ReportedQbiStatement,
    /// Distinct trades/businesses actually identified by the source owner.
    /// [NullText]
    /// The source does not establish the constituent businesses.
    /// [SetText]
    /// The source identifies these businesses
    pub business_names: Option<Vec<K1QbiBusinessName>>,
    /// Actual component-attributed current and retained qualified-source records.
    /// [NullText]
    /// The source allocation is unavailable.
    /// [SetText]
    /// The source establishes these allocations
    pub allocations: Option<Vec<K1QbiAllocationPage>>,
    /// Individual aggregation group for this block; empty means none.
    #[libertas_default("")]
    pub qbi_group: String,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum K1ReportedQbiStatement {
    Partnership {
        /// Printed actual statement for this reported business/block.
        /// [NullText]
        /// No current statement is reported for this source.
        /// [SetText]
        /// The source statement is
        statement: Option<crate::PartnerQbiStatement>,
    },
    Scorp {
        /// Printed actual statement for this reported business/block.
        /// [NullText]
        /// No current statement is reported for this source.
        /// [SetText]
        /// The source statement is
        statement: Option<K1ScorpBlockStatement>,
    },
    /// The reported-block source is unavailable.
    Unavailable,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1ScorpBlockStatement {
    /// Actual signed CURRENT ordinary amount on this reported business/block statement.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = -9223372036854775807)]
    pub income: i64,
    /// Actual signed CURRENT property amount on this reported business/block statement.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = -9223372036854775807)]
    pub section1231_qbi: i64,
    /// Original allocated W-2 wages, once for this indivisible reported block.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub wages: i64,
    /// Original allocated unadjusted basis of qualified property, once for this block.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub property: i64,
    /// Original specified-service classification; no current eligibility is computed here.
    /// Check the original Section 199A statement and ask its issuer when classification is missing.
    pub service: bool,
    /// Original qualified REIT dividend amount.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub reit_dividends: i64,
    /// Original qualified PTP source amount.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = -9223372036854775807)]
    pub ptp_income: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1StatementScorpCapitalCarry {
    /// Actual retained basis short source portion.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    /// [NullText]
    /// This retained source amount is unavailable.
    /// [SetText]
    /// The actual retained source amount is
    pub basis_short: Option<i64>,
    /// Actual retained basis long source portion.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    /// [NullText]
    /// This retained source amount is unavailable.
    /// [SetText]
    /// The actual retained source amount is
    pub basis_long: Option<i64>,
    /// Actual retained basis collectibles source portion.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    /// [NullText]
    /// This retained source amount is unavailable.
    /// [SetText]
    /// The actual retained source amount is
    pub basis_collectibles: Option<i64>,
    /// Actual retained risk short source portion.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    /// [NullText]
    /// This retained source amount is unavailable.
    /// [SetText]
    /// The actual retained source amount is
    pub risk_short: Option<i64>,
    /// Actual retained risk long source portion.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    /// [NullText]
    /// This retained source amount is unavailable.
    /// [SetText]
    /// The actual retained source amount is
    pub risk_long: Option<i64>,
    /// Actual retained risk collectibles source portion.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    /// [NullText]
    /// This retained source amount is unavailable.
    /// [SetText]
    /// The actual retained source amount is
    pub risk_collectibles: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1EntityOnlyAmounts {
    /// Actual entity-only exempt interest source.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub exempt_interest: i64,
    /// Actual entity-only exempt other source.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub exempt_other: i64,
    /// Actual entity-only nondeductible source.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub nondeductible: i64,
    /// Actual entity-only cash charity source.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub cash_charity: i64,
    /// Actual entity-only charity basis carry source.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    /// [NullText]
    /// This retained source amount is unavailable.
    /// [SetText]
    /// The actual retained source amount is
    pub charity_basis_carry: Option<i64>,
    /// Actual entity-only foreign taxes source.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub foreign_taxes: i64,
    /// Actual entity-only foreign basis carry source.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    /// [NullText]
    /// This retained source amount is unavailable.
    /// [SetText]
    /// The actual retained source amount is
    pub foreign_basis_carry: Option<i64>,
    pub portfolio: K1StatementPortfolio,
    pub items: K1StatementItems,
    pub capital_carry: K1StatementCapitalCarry,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
// Complete source payloads stay inline, matching the normal App wire model.
#[allow(clippy::large_enum_variant)]
pub enum K1EntityOnlyAmt {
    /// The supporting source establishes identical regular and AMT entity-only parts.
    SameAsRegular,
    /// Actual separate AMT entity-only source parts and balances.
    Separate { amounts: K1EntityOnlyAmounts },
    /// The actual AMT entity-only source is unavailable.
    Unavailable,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1PartnerSeSource {
    /// Actual reported source net SE earnings, before owner adjustments.
    /// [NullText]
    /// This source's reported SE amount is unavailable.
    /// [SetText]
    /// The source reports
    #[libertas_money("USD", 2)]
    #[libertas_number(min = -9223372036854775807)]
    pub reported: Option<i64>,
    pub history: K1RetainedSeHistory,
    /// Actual source gross and farming/nonfarming character.
    /// [NullText]
    /// The actual SE gross source is unavailable.
    /// [SetText]
    /// Use the actual gross source
    pub gross: Option<crate::PartnerSeGross>,
    /// Actual guaranteed service-payment source subset.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub guaranteed_services: i64,
    /// Actual guaranteed capital-payment source subset.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub guaranteed_capital: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1QbiBusinessName {
    /// Trade/business name actually identified by the reported-block source.
    #[libertas_size(min = 1)]
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1StatementActivityPage {
    /// Actual accepted source identity; new rows start at zero.
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    #[libertas_number(min = 0)]
    pub id: i64,
    /// Name on the actual source statement.
    #[libertas_size(min = 1)]
    pub name: String,
    /// Actual statement activity character.
    pub kind: K1StatementKind,
    /// Actual personal-service source facts for this activity, separate from passive and SE classification.
    pub personal_services: K1ActivityPersonalSource,
    /// Actual regular-tax self-employment source, independent of AMT amounts.
    pub self_employment: K1StatementSe,
    /// Actual current positive-income and loss controls, before owner limitations.
    pub qualified_current_controls: K1QualifiedCurrentControls,
    /// This activity's regular-tax monetary source.
    /// [NullText]
    /// The source partition is unavailable.
    /// [SetText]
    /// Use these documented source parts
    pub regular: Option<K1StatementAmounts>,
    /// Actual AMT source fact; never inferred from absence.
    pub alternative: K1StatementAmt,
    /// Actual accepted same-parent at-risk source.
    /// [NullText]
    /// That source relationship is unavailable.
    /// [SetText]
    /// The actual source is
    #[libertas_number(min = 1)]
    #[libertas_enum_source("^<K1ComponentDetailsPage>.choices.risk")]
    pub risk_group: Option<i64>,
    /// Actual accepted same-parent passive source.
    /// [NullText]
    /// That source relationship is unavailable.
    /// [SetText]
    /// The actual source is
    #[libertas_number(min = 1)]
    #[libertas_enum_source("^<K1ComponentDetailsPage>.choices.passive")]
    pub passive_group: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1ActivitySourcesPage {
    /// Actual whole-sale source associations; no computed allowance or duplicated monetary values.
    /// [NullText]
    /// The sale-to-activity source attribution is unavailable.
    /// [SetText]
    /// The actual sale sources belong to these activities
    pub section179_sale_sources: Option<Vec<K1Section179SaleSourcePage>>,
    /// Actual statement activities.
    pub components: Vec<K1StatementActivityPage>,
    /// Actual separately identified at-risk groups.
    pub risk_groups: Vec<K1RiskGroup>,
    /// Actual separately identified passive groups.
    pub passive_groups: Vec<K1PassiveGroup>,
    /// Actual section199A businesses or indivisible reported aggregates.
    pub qbi_blocks: Vec<K1QbiBlockPage>,
    /// Amounts the source establishes outside all listed activities.
    /// [NullText]
    /// The entity-only source attribution is unavailable.
    /// [SetText]
    /// The entity-only source amounts are
    pub entity_only: Option<K1EntityOnlyAmounts>,
    /// Actual alternative-minimum-tax entity-only source; absence is never copied as zero.
    pub alternative_entity_only: K1EntityOnlyAmt,
    /// Actual whole-property source associations; no credit amount or computed allowance.
    /// [NullText]
    /// The property-to-activity source attribution is unavailable.
    /// [SetText]
    /// The actual property sources belong to these activities
    pub investment_property_sources: Option<Vec<K1InvestmentPropertySourcePage>>,
}

#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct K1ActivityChoiceContext {
    /// Actual accepted physical section179 sale sources
    #[libertas_read_only]
    #[libertas_output_only]
    pub sales: Vec<K1ActivitySourceChoice>,
    /// Actual accepted component sources
    #[libertas_read_only]
    #[libertas_output_only]
    pub components: Vec<K1ActivitySourceChoice>,
    /// Actual accepted at-risk groups
    #[libertas_read_only]
    #[libertas_output_only]
    pub risk: Vec<K1ActivitySourceChoice>,
    /// Actual accepted passive groups
    #[libertas_read_only]
    #[libertas_output_only]
    pub passive: Vec<K1ActivitySourceChoice>,
    /// Actual accepted physical investment-property sources
    #[libertas_read_only]
    #[libertas_output_only]
    pub properties: Vec<K1ActivitySourceChoice>,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1ActivitySourceChoice {
    /// Actual accepted global source identity
    #[libertas_key]
    #[libertas_unique(1)]
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_number(min = 1)]
    pub id: i64,
    /// Source
    #[libertas_formatted_text]
    #[libertas_read_only]
    pub label: Vec<u8>,
}

impl K1StatementActivityPage {
    pub(crate) fn record(&self) -> K1StatementActivity {
        K1StatementActivity {
            id: self.id,
            name: self.name.clone(),
            kind: self.kind.clone(),
            personal_services: self.personal_services.clone(),
            self_employment: self.self_employment.clone(),
            qualified_current_controls: self.qualified_current_controls.clone(),
            regular: self.regular.clone(),
            alternative: self.alternative.clone(),
            risk_group: self.risk_group,
            passive_group: self.passive_group,
        }
    }
    fn from_record(source: &K1StatementActivity) -> Self {
        Self {
            id: source.id,
            name: source.name.clone(),
            kind: source.kind.clone(),
            personal_services: source.personal_services.clone(),
            self_employment: source.self_employment.clone(),
            qualified_current_controls: source.qualified_current_controls.clone(),
            regular: source.regular.clone(),
            alternative: source.alternative.clone(),
            risk_group: source.risk_group,
            passive_group: source.passive_group,
        }
    }
}

impl K1QbiBlockPage {
    fn record(&self) -> K1QbiBlock {
        K1QbiBlock {
            id: self.id,
            name: self.name.clone(),
            reported_aggregate: self.reported_aggregate,
            qbi_aggregation_group: crate::common::optional_text(&self.qbi_group),
            statement: self.statement.clone(),
            business_names: self.business_names.clone(),
            allocations: self.allocations.as_ref().map(|items| {
                items
                    .iter()
                    .map(|v| K1QbiAllocation {
                        component: v.component,
                        qualified: v.qualified.clone(),
                    })
                    .collect()
            }),
        }
    }
    fn from_record(source: &K1QbiBlock) -> Self {
        Self {
            id: source.id,
            name: source.name.clone(),
            reported_aggregate: source.reported_aggregate,
            qbi_group: source.qbi_aggregation_group.clone().unwrap_or_default(),
            statement: source.statement.clone(),
            business_names: source.business_names.clone(),
            allocations: source.allocations.as_ref().map(|items| {
                items
                    .iter()
                    .map(|v| K1QbiAllocationPage {
                        component: v.component,
                        qualified: v.qualified.clone(),
                    })
                    .collect()
            }),
        }
    }
}

// Source identity is authorized before assign_record_ids changes new zero IDs.
// A role switch, foreign parent ID or reference to a row created in this edit
// cannot borrow authorization from another role's accepted catalog.
impl K1ActivitySources {
    pub(crate) fn ids(&self) -> impl Iterator<Item = i64> + '_ {
        self.components
            .iter()
            .map(|v| v.id)
            .chain(self.risk_groups.iter().map(|v| v.id))
            .chain(self.passive_groups.iter().map(|v| v.id))
            .chain(self.qbi_blocks.iter().map(|v| v.id))
    }

    pub(crate) fn ids_mut(&mut self) -> impl Iterator<Item = &mut i64> {
        self.components
            .iter_mut()
            .map(|v| &mut v.id)
            .chain(self.risk_groups.iter_mut().map(|v| &mut v.id))
            .chain(self.passive_groups.iter_mut().map(|v| &mut v.id))
            .chain(self.qbi_blocks.iter_mut().map(|v| &mut v.id))
    }

    #[cfg(test)]
    pub(crate) fn identity_rule(&self, accepted: Option<&Self>) -> Option<&'static str> {
        self.identity_issue(accepted).map(|issue| issue.key)
    }
    fn identity_issue(&self, accepted: Option<&Self>) -> Option<ValidationIssue> {
        use alloc::collections::BTreeSet;
        let roles = [
            (
                "components",
                self.components.iter().map(|v| v.id).collect::<Vec<_>>(),
                accepted.map_or_else(Vec::new, |v| v.components.iter().map(|v| v.id).collect()),
            ),
            (
                "risk_groups",
                self.risk_groups.iter().map(|v| v.id).collect::<Vec<_>>(),
                accepted.map_or_else(Vec::new, |v| v.risk_groups.iter().map(|v| v.id).collect()),
            ),
            (
                "passive_groups",
                self.passive_groups.iter().map(|v| v.id).collect::<Vec<_>>(),
                accepted.map_or_else(Vec::new, |v| v.passive_groups.iter().map(|v| v.id).collect()),
            ),
            (
                "qbi_blocks",
                self.qbi_blocks.iter().map(|v| v.id).collect::<Vec<_>>(),
                accepted.map_or_else(Vec::new, |v| v.qbi_blocks.iter().map(|v| v.id).collect()),
            ),
        ];
        for (field, ids, old) in &roles {
            if let Some(index) =
                ids.iter().position(|id| *id != 0 && (*id < 0 || !old.contains(id)))
            {
                return Some(
                    ValidationIssue::new("K1_ACTIVITY_SOURCE_ID_NOT_ACCEPTED")
                        .at_field("id")
                        .in_list(field, index),
                );
            }
        }
        let mut seen = BTreeSet::new();
        for (field, ids, _) in &roles {
            if let Some(index) = ids.iter().position(|id| *id != 0 && !seen.insert(*id)) {
                return Some(
                    ValidationIssue::new("K1_ACTIVITY_SOURCE_ID_REPEATED")
                        .at_field("id")
                        .in_list(field, index),
                );
            }
        }
        let reference = |id: i64, current: &[i64], old: &[i64]| {
            id > 0 && current.contains(&id) && old.contains(&id)
        };
        for (index, component) in self.components.iter().enumerate() {
            for (field, id, role) in [
                ("risk_group", component.risk_group, 1),
                ("passive_group", component.passive_group, 2),
            ] {
                if id.is_some_and(|id| !reference(id, &roles[role].1, &roles[role].2)) {
                    return Some(
                        ValidationIssue::new("K1_ACTIVITY_SOURCE_REFERENCE_NOT_ACCEPTED")
                            .at_field(field)
                            .in_list("components", index),
                    );
                }
            }
        }
        for (block_index, block) in self.qbi_blocks.iter().enumerate() {
            for (index, allocation) in block.allocations.iter().flatten().enumerate() {
                if !reference(allocation.component, &roles[0].1, &roles[0].2) {
                    return Some(
                        ValidationIssue::new("K1_ACTIVITY_SOURCE_REFERENCE_NOT_ACCEPTED")
                            .at_field("component")
                            .in_list("allocations", index)
                            .in_list("qbi_blocks", block_index),
                    );
                }
            }
        }
        for (block_index, block) in self.qbi_blocks.iter().enumerate() {
            if block.qbi_aggregation_group.is_some() && block.id == 0 {
                return Some(
                    ValidationIssue::new("QBI_AGGREGATION_ACCEPT_SOURCE_FIRST")
                        .at_field("qbi_group")
                        .in_list("qbi_blocks", block_index),
                );
            }
            let mut seen = BTreeSet::new();
            if let Some(index) =
                block.allocations.iter().flatten().position(|v| !seen.insert(v.component))
            {
                return Some(
                    ValidationIssue::new("K1_QBI_COMPONENT_REPEATED")
                        .at_field("component")
                        .in_list("allocations", index)
                        .in_list("qbi_blocks", block_index),
                );
            }
        }
        None
    }
}

// Amounts are nonnegative native cents. The type supplies the encoding limit;
// arbitrary entered-amount ceilings are not a separate source policy.
const SOURCE_MONEY: core::ops::RangeInclusive<i64> = 0..=i64::MAX;
// Printed signed controls use the full native representation. Reconciliation
// sums exact whole source cents, including i64::MIN, before comparison.
const SOURCE_SIGNED: core::ops::RangeInclusive<i64> = i64::MIN..=i64::MAX;

impl K1StatementPortfolio {
    fn valid(&self) -> bool {
        [
            self.interest,
            self.dividends,
            self.qualified_dividends,
            self.royalties,
            self.unrecaptured_partnership,
            self.unrecaptured_fund,
        ]
        .into_iter()
        .all(|v| SOURCE_MONEY.contains(&v))
            && [self.short, self.long, self.collectibles]
                .into_iter()
                .all(|v| SOURCE_SIGNED.contains(&v))
            && self.qualified_dividends <= self.dividends
    }
}
fn known_money(value: Option<i64>) -> bool {
    value.is_none_or(|value| SOURCE_MONEY.contains(&value))
}
fn known_subset(part: Option<i64>, total: Option<i64>) -> bool {
    match (part, total) {
        (Some(part), Some(total)) => part <= total,
        _ => true,
    }
}
impl K1RetainedLossCarry {
    fn valid(&self) -> bool {
        known_money(self.basis) && known_money(self.risk)
    }
}
impl K1RetainedSeHistory {
    fn valid(&self) -> bool {
        [self.basis, self.risk, self.passive].into_iter().all(known_money)
    }
}
impl K1StatementProperty {
    fn valid(&self) -> bool {
        SOURCE_SIGNED.contains(&self.net)
            && SOURCE_MONEY.contains(&self.unrecaptured)
            && [self.basis_loss, self.risk_loss, self.passive_loss].into_iter().all(known_money)
    }
}
impl K1StatementPartnerCapitalCarry {
    fn valid(&self) -> bool {
        self.short.valid() && self.long.valid()
    }
}
impl K1StatementCapitalCarry {
    fn valid(&self, partnership: bool) -> bool {
        match self {
            Self::Partnership { amounts } => partnership && amounts.valid(),
            Self::Scorp { amounts } => {
                !partnership
                    && [
                        amounts.basis_short,
                        amounts.basis_long,
                        amounts.basis_collectibles,
                        amounts.risk_short,
                        amounts.risk_long,
                        amounts.risk_collectibles,
                    ]
                    .into_iter()
                    .all(known_money)
                    && known_subset(amounts.basis_collectibles, amounts.basis_long)
                    && known_subset(amounts.risk_collectibles, amounts.risk_long)
            }
            Self::Unavailable => true,
        }
    }
}
impl K1StatementItems {
    fn issue(&self, partnership: bool) -> Option<ValidationIssue> {
        match self {
            Self::Partnership { income, deductions } if partnership => {
                crate::PartnerIncome::issue(income, "income")
                    .or_else(|| crate::PartnerDeduction::issue(deductions, "deductions"))
                    .or_else(|| {
                        (!crate::PartnerIncome::valid_list(income)).then(|| {
                            ValidationIssue::new("K1_ACTIVITY_SOURCE_AMOUNT_INVALID")
                                .at_field("income")
                        })
                    })
                    .or_else(|| {
                        (!crate::PartnerDeduction::valid_list(deductions)).then(|| {
                            ValidationIssue::new("K1_ACTIVITY_SOURCE_AMOUNT_INVALID")
                                .at_field("deductions")
                        })
                    })
            }
            Self::Scorp { income, deductions } if !partnership => {
                crate::ScorpIncome::issue(income, "income")
                    .or_else(|| crate::ScorpDeduction::issue(deductions, "deductions"))
                    .or_else(|| {
                        (!crate::ScorpIncome::valid_list(income)).then(|| {
                            ValidationIssue::new("K1_ACTIVITY_SOURCE_AMOUNT_INVALID")
                                .at_field("income")
                        })
                    })
                    .or_else(|| {
                        (!crate::ScorpDeduction::valid_list(deductions)).then(|| {
                            ValidationIssue::new("K1_ACTIVITY_SOURCE_AMOUNT_INVALID")
                                .at_field("deductions")
                        })
                    })
            }
            _ => Some(ValidationIssue::new("K1_ACTIVITY_SOURCE_ENTITY_MISMATCH")),
        }
    }
}
impl K1StatementAmounts {
    #[cfg(test)]
    pub(crate) fn rule(&self, partnership: bool) -> Option<&'static str> {
        self.issue(partnership).map(|issue| issue.key)
    }
    fn issue(&self, partnership: bool) -> Option<ValidationIssue> {
        let fields = [
            ("operating", SOURCE_SIGNED.contains(&self.operating)),
            ("exempt_interest", SOURCE_MONEY.contains(&self.exempt_interest)),
            ("exempt_other", SOURCE_MONEY.contains(&self.exempt_other)),
            ("nondeductible", SOURCE_MONEY.contains(&self.nondeductible)),
            ("section179", SOURCE_MONEY.contains(&self.section179)),
            ("cash_charity", SOURCE_MONEY.contains(&self.cash_charity)),
            ("foreign_taxes", SOURCE_MONEY.contains(&self.foreign_taxes)),
            ("passive_operating", known_money(self.passive_operating)),
            ("section179_basis_carry", known_money(self.section179_basis_carry)),
            ("section179_business_carry", known_money(self.section179_business_carry)),
            ("section179_sold_carry", known_money(self.section179_sold_carry)),
            ("charity_basis_carry", known_money(self.charity_basis_carry)),
            ("foreign_basis_carry", known_money(self.foreign_basis_carry)),
            ("operating_carry", self.operating_carry.valid()),
            ("portfolio", self.portfolio.valid()),
            ("property", self.property.as_ref().is_none_or(|v| v.valid())),
            ("capital_carry", self.capital_carry.valid(partnership)),
        ];
        if let Some((field, _)) = fields.into_iter().find(|(_, valid)| !valid) {
            return Some(ValidationIssue::new("K1_ACTIVITY_SOURCE_AMOUNT_INVALID").at_field(field));
        }
        if !known_subset(self.section179_sold_carry, self.section179_business_carry) {
            return Some(
                ValidationIssue::new("K1_ACTIVITY_SOLD_CARRY_ABOVE_TOTAL")
                    .at_field("section179_sold_carry"),
            );
        }
        self.items.issue(partnership).map(|issue| in_source_field(issue, "items"))
    }
}
impl K1EntityOnlyAmounts {
    fn issue(&self, partnership: bool) -> Option<ValidationIssue> {
        let fields = [
            ("exempt_interest", SOURCE_MONEY.contains(&self.exempt_interest)),
            ("exempt_other", SOURCE_MONEY.contains(&self.exempt_other)),
            ("nondeductible", SOURCE_MONEY.contains(&self.nondeductible)),
            ("cash_charity", SOURCE_MONEY.contains(&self.cash_charity)),
            ("foreign_taxes", SOURCE_MONEY.contains(&self.foreign_taxes)),
            ("charity_basis_carry", known_money(self.charity_basis_carry)),
            ("foreign_basis_carry", known_money(self.foreign_basis_carry)),
            ("portfolio", self.portfolio.valid()),
            ("capital_carry", self.capital_carry.valid(partnership)),
        ];
        if let Some((field, _)) = fields.into_iter().find(|(_, valid)| !valid) {
            return Some(ValidationIssue::new("K1_ACTIVITY_SOURCE_AMOUNT_INVALID").at_field(field));
        }
        self.items.issue(partnership).map(|issue| in_source_field(issue, "items"))
    }
}

impl K1ComponentQualified {
    fn valid(&self, partnership: bool) -> bool {
        match self {
            Self::Partnership { current_income, current_property, records } => {
                partnership
                    && *current_income != i64::MIN
                    && *current_property != i64::MIN
                    && records.valid()
            }
            Self::Scorp { current_income, ordinary, property } => {
                !partnership
                    && SOURCE_MONEY.contains(current_income)
                    && ordinary.valid_retained()
                    && property.valid_retained()
            }
            Self::Unavailable => true,
        }
    }
    // These are entered regular-tax qualified source subsets. No annual SSTB,
    // basis, at-risk, PAL or EBL allowance is computed in this source module.
    fn current(&self) -> Option<[i64; 3]> {
        match self {
            Self::Partnership { current_income, current_property, records } => {
                Some([*current_income, *current_property, records.qualified_se_gross])
            }
            Self::Scorp { current_income, ordinary, property } => Some([
                *current_income - ordinary.current,
                property.gain - property.losses.current,
                0,
            ]),
            Self::Unavailable => None,
        }
    }
    fn histories(&self) -> Vec<(u8, u8, &[crate::ScorpLossYear])> {
        match self {
            Self::Partnership { records, .. } => {
                records.history.as_ref().map_or_else(Vec::new, |v| {
                    alloc::vec![
                        (0, 0, v.basis.as_slice()),
                        (0, 1, v.risk.as_slice()),
                        (0, 2, v.passive.as_slice())
                    ]
                })
            }
            Self::Scorp { ordinary, property, .. } => alloc::vec![
                (0, 0, ordinary.history.as_slice()),
                (0, 1, ordinary.at_risk.as_slice()),
                (0, 2, ordinary.passive.as_slice()),
                (1, 0, property.losses.history.as_slice()),
                (1, 1, property.losses.at_risk.as_slice()),
                (1, 2, property.losses.passive.as_slice()),
            ],
            Self::Unavailable => Vec::new(),
        }
    }
}

impl K1ActivitySources {
    #[cfg(test)]
    pub(crate) fn source_rule(&self, partnership: bool) -> Option<&'static str> {
        self.source_issue(partnership).map(|issue| issue.key)
    }
    fn source_issue(&self, partnership: bool) -> Option<ValidationIssue> {
        if let Some(issue) = sale_source_issue(self.section179_sale_sources.as_deref()) {
            return Some(issue);
        }
        for (field, names) in [
            ("components", self.components.iter().map(|v| &v.name).collect::<Vec<_>>()),
            ("risk_groups", self.risk_groups.iter().map(|v| &v.name).collect::<Vec<_>>()),
            ("passive_groups", self.passive_groups.iter().map(|v| &v.name).collect::<Vec<_>>()),
            ("qbi_blocks", self.qbi_blocks.iter().map(|v| &v.name).collect::<Vec<_>>()),
        ] {
            if let Some(index) = names.iter().position(|name| !source_name(name)) {
                return Some(
                    ValidationIssue::new("K1_ACTIVITY_SOURCE_SIZE_INVALID")
                        .at_field("name")
                        .in_list(field, index),
                );
            }
        }
        if let Some(issue) = property_source_issue(self.investment_property_sources.as_deref()) {
            return Some(issue);
        }

        for (index, component) in self.components.iter().enumerate() {
            if [
                &component.qualified_current_controls.ordinary,
                &component.qualified_current_controls.property,
            ]
            .into_iter()
            .flatten()
            .any(|v| !SOURCE_MONEY.contains(&v.income) || !SOURCE_MONEY.contains(&v.loss))
            {
                return Some(
                    ValidationIssue::new("K1_ACTIVITY_SOURCE_AMOUNT_INVALID")
                        .at_field("qualified_current_controls")
                        .in_list("components", index),
                );
            }
            match &component.self_employment {
                K1StatementSe::Partnership { source } if partnership => {
                    if !source.history.valid()
                        || source.reported.is_some_and(|v| !SOURCE_SIGNED.contains(&v))
                        || ![source.guaranteed_services, source.guaranteed_capital]
                            .into_iter()
                            .all(|v| SOURCE_MONEY.contains(&v))
                        || source
                            .gross
                            .as_ref()
                            .is_some_and(|v| !SOURCE_MONEY.contains(&v.source().1))
                    {
                        return Some(
                            ValidationIssue::new("K1_ACTIVITY_SOURCE_AMOUNT_INVALID")
                                .at_field("self_employment")
                                .in_list("components", index),
                        );
                    }
                    if component.regular.as_ref().is_some_and(|amounts| {
                        !known_subset(source.history.basis, amounts.operating_carry.basis)
                            || !known_subset(source.history.risk, amounts.operating_carry.risk)
                            || !known_subset(source.history.passive, amounts.passive_operating)
                    }) {
                        return Some(
                            ValidationIssue::new("K1_ACTIVITY_SE_HISTORY_ABOVE_LOSS")
                                .at_field("history")
                                .in_field("source")
                                .in_field("self_employment")
                                .in_list("components", index),
                        );
                    }
                }
                K1StatementSe::NotApplicable => {}
                K1StatementSe::Unavailable => {}
                _ => {
                    return Some(
                        ValidationIssue::new("K1_ACTIVITY_SOURCE_ENTITY_MISMATCH")
                            .at_field("self_employment")
                            .in_list("components", index),
                    );
                }
            }
            match &component.personal_services {
                K1ActivityPersonalSource::Partnership { gross_personal_services, .. }
                    if partnership =>
                {
                    if gross_personal_services.is_some_and(|v| !SOURCE_MONEY.contains(&v)) {
                        return Some(
                            ValidationIssue::new("K1_ACTIVITY_SOURCE_AMOUNT_INVALID")
                                .at_field("gross_personal_services")
                                .in_field("personal_services")
                                .in_list("components", index),
                        );
                    }
                }
                K1ActivityPersonalSource::NotApplicable | K1ActivityPersonalSource::Unavailable => {
                }
                _ => {
                    return Some(
                        ValidationIssue::new("K1_ACTIVITY_SOURCE_ENTITY_MISMATCH")
                            .at_field("personal_services")
                            .in_list("components", index),
                    );
                }
            }
            if let Some(rule) = component.regular.as_ref().and_then(|v| v.issue(partnership)) {
                return Some(in_source_field(rule, "regular").in_list("components", index));
            }
            if let K1StatementAmt::Separate { amounts } = &component.alternative
                && let Some(rule) = amounts.issue(partnership)
            {
                return Some(
                    in_source_field(in_source_field(rule, "amounts"), "alternative")
                        .in_list("components", index),
                );
            }
        }
        if let Some(rule) = self.entity_only.as_ref().and_then(|v| v.issue(partnership)) {
            return Some(in_source_field(rule, "entity_only"));
        }
        if let K1EntityOnlyAmt::Separate { amounts } = &self.alternative_entity_only
            && let Some(key) = amounts.issue(partnership)
        {
            return Some(in_source_field(
                in_source_field(key, "amounts"),
                "alternative_entity_only",
            ));
        }
        for (index, risk) in self.risk_groups.iter().enumerate() {
            if risk.investment_parts.as_ref().is_some_and(|parts| {
                ![
                    parts.contributions,
                    parts.distributions,
                    parts.loan_advances,
                    parts.loan_repayments,
                ]
                .into_iter()
                .all(|v| SOURCE_MONEY.contains(&v))
            }) {
                return Some(
                    ValidationIssue::new("K1_ACTIVITY_SOURCE_AMOUNT_INVALID")
                        .at_field("investment_parts")
                        .in_list("risk_groups", index),
                );
            }
            // Reuse each actual source owner's intrinsic history predicate.
            // Contextual financing and capacity still need its real group feed;
            // no synthetic legacy K1 is created to run them.
            match &risk.source {
                K1RiskSource::Partnership { regular, alternative } if partnership => {
                    if !regular.source_valid()
                        || alternative.as_ref().is_some_and(|v| !v.source_valid())
                    {
                        return Some(
                            ValidationIssue::new("K1_ACTIVITY_SOURCE_AMOUNT_INVALID")
                                .at_field("source")
                                .in_list("risk_groups", index),
                        );
                    }
                }
                K1RiskSource::Scorp { regular, alternative } if !partnership => {
                    if !regular.source_valid()
                        || alternative.as_ref().is_some_and(|v| !v.source_valid())
                    {
                        return Some(
                            ValidationIssue::new("K1_ACTIVITY_SOURCE_AMOUNT_INVALID")
                                .at_field("source")
                                .in_list("risk_groups", index),
                        );
                    }
                }
                K1RiskSource::Unavailable => {}
                _ => {
                    return Some(
                        ValidationIssue::new("K1_ACTIVITY_SOURCE_ENTITY_MISMATCH")
                            .at_field("source")
                            .in_list("risk_groups", index),
                    );
                }
            }
        }
        for (index, passive) in self.passive_groups.iter().enumerate() {
            match &passive.participation {
                K1ActivityParticipation::Partnership { facts, .. } if partnership => {
                    // The existing classification owner returns Unavailable for
                    // malformed participation facts; source absence has its
                    // own Unavailable variant here.
                    if facts.material() == crate::participation::Participation::Unavailable {
                        return Some(
                            ValidationIssue::new("K1_ACTIVITY_SOURCE_AMOUNT_INVALID")
                                .at_field("facts")
                                .in_field("participation")
                                .in_list("passive_groups", index),
                        );
                    }
                }
                K1ActivityParticipation::Scorp { facts, .. } if !partnership => {
                    if !facts.valid() {
                        return Some(
                            ValidationIssue::new("K1_ACTIVITY_SOURCE_AMOUNT_INVALID")
                                .at_field("facts")
                                .in_field("participation")
                                .in_list("passive_groups", index),
                        );
                    }
                }
                K1ActivityParticipation::Rental { .. } | K1ActivityParticipation::Unavailable => {}
                _ => {
                    return Some(
                        ValidationIssue::new("K1_ACTIVITY_SOURCE_ENTITY_MISMATCH")
                            .at_field("participation")
                            .in_list("passive_groups", index),
                    );
                }
            }
        }
        for (index, block) in self.qbi_blocks.iter().enumerate() {
            if let Some(names) = &block.business_names
                && let Some(name_index) = names.iter().position(|v| !source_name(&v.name))
            {
                return Some(
                    ValidationIssue::new("K1_ACTIVITY_SOURCE_SIZE_INVALID")
                        .at_field("name")
                        .in_list("business_names", name_index)
                        .in_list("qbi_blocks", index),
                );
            }
            match &block.statement {
                K1ReportedQbiStatement::Partnership { statement } if partnership => {
                    if statement.as_ref().is_some_and(|v| {
                        !SOURCE_SIGNED.contains(&v.income)
                            || !SOURCE_SIGNED.contains(&v.section1231_qbi)
                            || !SOURCE_SIGNED.contains(&v.ptp_income)
                            || ![v.wages, v.property, v.reit_dividends]
                                .into_iter()
                                .all(|v| SOURCE_MONEY.contains(&v))
                    }) {
                        return Some(
                            ValidationIssue::new("K1_ACTIVITY_SOURCE_AMOUNT_INVALID")
                                .at_field("statement")
                                .in_list("qbi_blocks", index),
                        );
                    }
                }
                K1ReportedQbiStatement::Scorp { statement } if !partnership => {
                    if statement.as_ref().is_some_and(|v| {
                        !SOURCE_SIGNED.contains(&v.ptp_income)
                            || !SOURCE_SIGNED.contains(&v.income)
                            || !SOURCE_SIGNED.contains(&v.section1231_qbi)
                            || ![v.wages, v.property, v.reit_dividends]
                                .into_iter()
                                .all(|v| SOURCE_MONEY.contains(&v))
                    }) {
                        return Some(
                            ValidationIssue::new("K1_ACTIVITY_SOURCE_AMOUNT_INVALID")
                                .at_field("statement")
                                .in_list("qbi_blocks", index),
                        );
                    }
                }
                K1ReportedQbiStatement::Unavailable => {}
                _ => {
                    return Some(
                        ValidationIssue::new("K1_ACTIVITY_SOURCE_ENTITY_MISMATCH")
                            .at_field("statement")
                            .in_list("qbi_blocks", index),
                    );
                }
            }
            if let Some(allocation_index) =
                block.allocations.iter().flatten().position(|v| !v.qualified.valid(partnership))
            {
                return Some(
                    ValidationIssue::new("K1_QBI_SOURCE_INVALID")
                        .at_field("qualified")
                        .in_list("allocations", allocation_index)
                        .in_list("qbi_blocks", index),
                );
            }
        }
        // All qualified scalar and retained-history owners above have validated
        // these sources before any signed magnitude or subset accumulation.
        self.qualified_source_issue()
    }

    fn qualified_source_issue(&self) -> Option<ValidationIssue> {
        use alloc::collections::BTreeMap;
        // One authoritative tax control per component/character/stage/year.
        // Only the block's original/remaining qualified subsets are summed.
        let mut histories: BTreeMap<(i64, u8, u8, u32), [i64; 4]> = BTreeMap::new();
        let mut currents: BTreeMap<i64, [exact::Exact; 5]> = BTreeMap::new();
        for (block_index, block) in self.qbi_blocks.iter().enumerate() {
            if let Some(allocations) = &block.allocations {
                let known =
                    allocations.iter().map(|v| v.qualified.current()).collect::<Option<Vec<_>>>();
                if let Some(known) = known {
                    let actual = [
                        known.iter().map(|v| exact::whole(v[0])).sum::<exact::Exact>(),
                        known.iter().map(|v| exact::whole(v[1])).sum::<exact::Exact>(),
                    ];
                    let reported = match &block.statement {
                        K1ReportedQbiStatement::Partnership { statement } => Some(
                            statement.as_ref().map_or([0; 2], |v| [v.income, v.section1231_qbi]),
                        ),
                        K1ReportedQbiStatement::Scorp { statement } => Some(
                            statement.as_ref().map_or([0; 2], |v| [v.income, v.section1231_qbi]),
                        ),
                        K1ReportedQbiStatement::Unavailable => None,
                    };
                    if reported.is_some_and(|v| v.map(exact::whole) != actual) {
                        return Some(
                            ValidationIssue::new("K1_QBI_PRINTED_CURRENT_RECONCILE")
                                .at_field("statement")
                                .in_list("qbi_blocks", block_index),
                        );
                    }
                }
            }
            for (allocation_index, allocation) in block.allocations.iter().flatten().enumerate() {
                if let Some(current) = allocation.qualified.signs() {
                    // Source controls may be unknown. Keep even a sign partition
                    // exact; its total has no independent native ceiling when
                    // the corresponding actual source control is absent.
                    let total = currents
                        .entry(allocation.component)
                        .or_insert_with(|| core::array::from_fn(|_| exact::whole(0)));
                    for i in 0..5 {
                        total[i] += exact::whole(current[i]);
                    }
                }
                for (character, stage, rows) in allocation.qualified.histories() {
                    for row in rows {
                        let key = (allocation.component, character, stage, row.year);
                        let value =
                            histories.entry(key).or_insert([row.original, row.remaining, 0, 0]);
                        if value[0] != row.original || value[1] != row.remaining {
                            return Some(
                                ValidationIssue::new("K1_QBI_SHARED_TAX_HISTORY_CONFLICT")
                                    .at_field("qualified")
                                    .in_list("allocations", allocation_index)
                                    .in_list("qbi_blocks", block_index),
                            );
                        }
                        // Canonical retained-history validation ran above. Both
                        // subsets are nonnegative; overflow exceeds the actual
                        // native original/remaining control, with no cancellation.
                        let qualified = value[2].checked_add(row.qualified);
                        let remaining = value[3].checked_add(row.remaining_qualified);
                        if qualified.is_none() || remaining.is_none() {
                            return Some(
                                ValidationIssue::new(
                                    "K1_QBI_SHARED_QUALIFIED_HISTORY_ABOVE_SOURCE",
                                )
                                .at_field("qualified")
                                .in_list("allocations", allocation_index)
                                .in_list("qbi_blocks", block_index),
                            );
                        }
                        value[2] = qualified.unwrap();
                        value[3] = remaining.unwrap();
                        if value[2] > value[0]
                            || value[3] > value[1]
                            || value[3] > value[2]
                            || value[2] - value[3] > value[0] - value[1]
                        {
                            return Some(
                                ValidationIssue::new(
                                    "K1_QBI_SHARED_QUALIFIED_HISTORY_ABOVE_SOURCE",
                                )
                                .at_field("qualified")
                                .in_list("allocations", allocation_index)
                                .in_list("qbi_blocks", block_index),
                            );
                        }
                    }
                }
            }
        }
        for (component_index, component) in self.components.iter().enumerate() {
            if let (Some(amounts), Some(current)) =
                (component.regular.as_ref(), currents.get(&component.id))
            {
                for (field, controls, indices) in [
                    ("ordinary", &component.qualified_current_controls.ordinary, [0, 1]),
                    ("property", &component.qualified_current_controls.property, [2, 3]),
                ] {
                    if controls.as_ref().is_some_and(|v| {
                        current[indices[0]] > exact::whole(v.income)
                            || current[indices[1]] > exact::whole(v.loss)
                    }) {
                        return Some(
                            ValidationIssue::new("K1_QBI_CURRENT_ABOVE_COMPONENT_SOURCE")
                                .at_field(field)
                                .in_field("qualified_current_controls")
                                .in_list("components", component_index),
                        );
                    }
                }
                // These raw sign controls retain their original source provenance.
                // The canonical character owner reconciles them to the tax source;
                // raw net box values are not substitutes for that delivered view.
                if let K1StatementSe::Partnership { source } = &component.self_employment
                    && source
                        .gross
                        .as_ref()
                        .is_some_and(|v| current[4] > exact::whole(v.source().1))
                {
                    return Some(
                        ValidationIssue::new("K1_QBI_GROSS_ABOVE_COMPONENT_SOURCE")
                            .at_field("gross")
                            .in_field("source")
                            .in_field("self_employment")
                            .in_list("components", component_index),
                    );
                }
                for character in 0..=1 {
                    let balances = if character == 0 {
                        [
                            amounts.operating_carry.basis,
                            amounts.operating_carry.risk,
                            amounts.passive_operating,
                        ]
                    } else {
                        amounts
                            .property
                            .as_ref()
                            .map_or([Some(0); 3], |v| [v.basis_loss, v.risk_loss, v.passive_loss])
                    };
                    for stage in 0..=2 {
                        let selected: Vec<_> = histories
                            .iter()
                            .filter(|(key, _)| {
                                key.0 == component.id && key.1 == character && key.2 == stage
                            })
                            .collect();
                        if !selected.is_empty()
                            && balances[usize::from(stage)].is_some_and(|balance| {
                                selected
                                    .iter()
                                    .map(|(_, v)| exact::whole(v[1]))
                                    .sum::<exact::Exact>()
                                    != exact::whole(balance)
                            })
                        {
                            let issue =
                                ValidationIssue::new("K1_QBI_TAX_HISTORY_RECONCILE_COMPONENT");
                            let issue = if character == 0 {
                                if stage == 2 {
                                    issue.at_field("passive_operating")
                                } else {
                                    issue
                                        .at_field(if stage == 0 { "basis" } else { "risk" })
                                        .in_field("operating_carry")
                                }
                            } else {
                                issue
                                    .at_field(match stage {
                                        0 => "basis_loss",
                                        1 => "risk_loss",
                                        _ => "passive_loss",
                                    })
                                    .in_field("property")
                            };
                            return Some(
                                issue.in_field("regular").in_list("components", component_index),
                            );
                        }
                    }
                }
            }
        }
        None
    }
}
fn source_name(value: &str) -> bool {
    !value.is_empty()
}

impl K1ActivityChoiceContext {
    pub(crate) fn from_sources(source: Option<&K1ActivitySources>) -> Self {
        let Some(source) = source else {
            return Self::default();
        };
        let choice = |id: i64, name: &String| K1ActivitySourceChoice {
            id,
            label: crate::Message::new(
                "LABEL_K1_ACTIVITY_SOURCE",
                alloc::vec![crate::MessageArg::Text(name.clone())],
            )
            .encode(),
        };
        Self {
            components: source
                .components
                .iter()
                .filter(|v| v.id > 0)
                .map(|v| choice(v.id, &v.name))
                .collect(),
            risk: source
                .risk_groups
                .iter()
                .filter(|v| v.id > 0)
                .map(|v| choice(v.id, &v.name))
                .collect(),
            passive: source
                .passive_groups
                .iter()
                .filter(|v| v.id > 0)
                .map(|v| choice(v.id, &v.name))
                .collect(),
            properties: Vec::new(),
            sales: Vec::new(),
        }
    }
}

// Genuine multi-activity parents: no legacy aggregate risk/passive record.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PartnerEntityBasis {
    /// Opening outside basis for this whole partnership interest.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    /// [NullText]
    /// The opening basis source is unavailable.
    /// [SetText]
    /// The established opening basis is
    pub opening: Option<i64>,
    /// Actual cash capital contributed, excluding guaranteed payments and loan shares.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub contributions: i64,
    /// Beginning allocated partnership liabilities for outside basis.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub opening_liabilities: i64,
    /// Ending allocated partnership liabilities for outside basis.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub ending_liabilities: i64,
    /// Partnership liabilities personally assumed during the year.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub assumed: i64,
    /// Personal liabilities assumed by the partnership.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub transferred: i64,
    /// Actual cash distributions, including premiums charged against them.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub distributions: i64,
    /// Prior basis-only nondeductible expense control; not a deductible loss.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    /// [NullText]
    /// The prior nondeductible amount source is unavailable.
    /// [SetText]
    /// The established prior nondeductible amount is
    pub prior_nondeductible: Option<i64>,
    /// The whole-interest cash/liability source records and annual ordering are established.
    pub complete: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ScorpEntityBasis {
    /// Opening stock basis for the whole shareholding.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    /// [NullText]
    /// The opening basis source is unavailable.
    /// [SetText]
    /// The established opening basis is
    pub opening: Option<i64>,
    /// Actual additional stock tax basis, excluding shareholder loans.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub additions: i64,
    /// Actual nondividend distributions.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub distributions: i64,
    /// Prior basis-only nondeductible expense control under the continuing election.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    /// [NullText]
    /// The prior nondeductible amount source is unavailable.
    /// [SetText]
    /// The established prior nondeductible amount is
    pub prior_nondeductible: Option<i64>,
    /// An already effective losses-before-nondeductible-expenses election is established.
    pub losses_first: bool,
    /// The whole-shareholding basis source records and annual ordering are established.
    pub complete: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ScorpEntityLoan {
    /// The actual direct shareholder loan, including principal and repayment facts.
    pub regular: crate::ScorpLoan,
    /// Opening AMT basis for this same physical loan.
    /// [NullText]
    /// The separate AMT basis is unavailable.
    /// [SetText]
    /// The actual AMT opening basis is
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub alternative_basis: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PartnerEntitySource {
    /// Actual AMT section179 income-limit carryover history from this K-1.
    /// This is independent of regular carryovers and of any unavailable component attribution.
    /// [NullText]
    /// This entity's AMT carryover history is unavailable.
    /// [SetText]
    /// The actual AMT carryover history is
    pub amt_section179_income_carry: Option<crate::Section179IncomeCarryHistory>,
    /// This entity's actual current tax period, from its own K-1 records.
    /// [NullText]
    /// This entity's tax period is unavailable.
    /// [SetText]
    /// This entity's actual tax period is
    pub reporting_period: Option<crate::K1EntityTaxPeriod>,
    /// Actual physical section179 sale sources, each retaining its regular and AMT availability.
    /// Save new sources before selecting their activity associations.
    pub section179_sales: Vec<crate::K1Section179Sale>,
    /// Whole-interest regular-tax basis source.
    pub regular: PartnerEntityBasis,
    /// Whole-interest separate AMT basis source.
    /// [NullText]
    /// The separate AMT global basis source is unavailable.
    /// [SetText]
    /// The actual AMT global basis source is
    pub alternative: Option<PartnerEntityBasis>,
    /// This is a publicly traded partnership under section7704(b).
    pub publicly_traded: bool,
    /// This is the final K-1.
    pub final_k1: bool,
    /// Established holding period of the partnership interest.
    /// [NullText]
    /// No holding period is established.
    /// [SetText]
    /// The interest holding period is
    pub interest_term: Option<crate::GainTerm>,
    /// What happened to the whole interest during the year.
    pub disposition: crate::K1Disposition,
    /// The actual separately identified whole-interest sale gain parts.
    pub sale: crate::PartnerSaleParts,
    /// Actual Schedule K-3 source.
    /// [NullText]
    /// There is no Schedule K-3 source.
    /// [SetText]
    /// The Schedule K-3 source is
    pub schedule_k3: Option<crate::ScheduleK3>,
    /// Actual entity-reported credit source records.
    pub credits: Vec<crate::K1Credit>,
    /// Actual investment-credit property source records.
    pub investment_property: Vec<crate::K1InvestmentProperty>,
    /// Actual investment-property confirmation.
    /// [NullText]
    /// The investment-property source is unconfirmed.
    /// [SetText]
    /// The source confirmation is
    pub investment_confirmed: Option<bool>,
    /// Actual low-income housing acquisition confirmation.
    /// [NullText]
    /// The housing source is unconfirmed.
    /// [SetText]
    /// The housing confirmation is
    pub housing_confirmed: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ScorpEntitySource {
    /// Actual AMT section179 income-limit carryover history from this K-1.
    /// This is independent of regular carryovers and of any unavailable component attribution.
    /// [NullText]
    /// This entity's AMT carryover history is unavailable.
    /// [SetText]
    /// The actual AMT carryover history is
    pub amt_section179_income_carry: Option<crate::Section179IncomeCarryHistory>,
    /// This entity's actual current tax period, from its own K-1 records.
    /// [NullText]
    /// This entity's tax period is unavailable.
    /// [SetText]
    /// This entity's actual tax period is
    pub reporting_period: Option<crate::K1EntityTaxPeriod>,
    /// Actual physical section179 sale sources, each retaining its regular and AMT availability.
    /// Save new sources before selecting their activity associations.
    pub section179_sales: Vec<crate::K1Section179Sale>,
    /// Whole-shareholding regular-tax stock basis source.
    pub regular: ScorpEntityBasis,
    /// Whole-shareholding separate AMT stock basis source.
    /// [NullText]
    /// The separate AMT stock basis source is unavailable.
    /// [SetText]
    /// The actual AMT stock basis source is
    pub alternative: Option<ScorpEntityBasis>,
    /// Actual shareholder loans, each retaining its same-source AMT basis.
    pub loans: Vec<ScorpEntityLoan>,
    /// This is the final K-1.
    pub final_k1: bool,
    /// Established holding period of the shares.
    /// [NullText]
    /// No share holding period is established.
    /// [SetText]
    /// The share holding period is
    pub stock_term: Option<crate::GainTerm>,
    /// What happened to the whole shareholding during the year.
    pub disposition: crate::K1Disposition,
    /// Actual Schedule K-3 source.
    /// [NullText]
    /// There is no Schedule K-3 source.
    /// [SetText]
    /// The Schedule K-3 source is
    pub schedule_k3: Option<crate::ScheduleK3>,
    /// Actual entity-reported credit source records.
    pub credits: Vec<crate::K1Credit>,
    /// Actual investment-credit property source records.
    pub investment_property: Vec<crate::K1InvestmentProperty>,
    /// Actual investment-property confirmation.
    /// [NullText]
    /// The investment-property source is unconfirmed.
    /// [SetText]
    /// The source confirmation is
    pub investment_confirmed: Option<bool>,
    /// Actual low-income housing acquisition confirmation.
    /// [NullText]
    /// The housing source is unconfirmed.
    /// [SetText]
    /// The housing confirmation is
    pub housing_confirmed: Option<bool>,
}

// Ephemeral Page conversion reuses the canonical code-specific credit input.
// The accepted source list and confirmation fact remain the single authority.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PartnerEntitySourcePage {
    /// Actual AMT section179 income-limit carryover history from this K-1.
    /// This is independent of regular carryovers and of any unavailable component attribution.
    /// [NullText]
    /// This entity's AMT carryover history is unavailable.
    /// [SetText]
    /// The actual AMT carryover history is
    pub amt_section179_income_carry: Option<crate::Section179IncomeCarryHistory>,
    /// Whole-interest regular-tax basis source.
    pub regular: PartnerEntityBasis,
    /// Whole-interest separate AMT basis source.
    /// [NullText]
    /// The separate AMT global basis source is unavailable.
    /// [SetText]
    /// The actual AMT global basis source is
    pub alternative: Option<PartnerEntityBasis>,
    /// This is a publicly traded partnership under section7704(b).
    pub publicly_traded: bool,
    /// This is the final K-1.
    pub final_k1: bool,
    /// Established holding period of the partnership interest.
    /// [NullText]
    /// No holding period is established.
    /// [SetText]
    /// The interest holding period is
    pub interest_term: Option<crate::GainTerm>,
    /// What happened to the whole interest during the year.
    pub disposition: crate::K1Disposition,
    /// The actual separately identified whole-interest sale gain parts.
    pub sale: crate::PartnerSaleParts,
    /// Actual Schedule K-3 source.
    /// [NullText]
    /// There is no Schedule K-3 source.
    /// [SetText]
    /// The Schedule K-3 source is
    pub schedule_k3: Option<crate::ScheduleK3>,
    /// Actual entity-reported credit source records.
    pub credits: Vec<crate::K1CreditPage>,
    /// Actual investment-credit property source records.
    pub investment_property: Vec<crate::K1InvestmentProperty>,
    /// Actual investment-property confirmation.
    /// [NullText]
    /// The investment-property source is unconfirmed.
    /// [SetText]
    /// The source confirmation is
    pub investment_confirmed: Option<crate::K1InvestmentConfirmation>,
    /// Actual low-income housing acquisition confirmation.
    /// [NullText]
    /// The housing source is unconfirmed.
    /// [SetText]
    /// The housing confirmation is
    pub housing_confirmed: Option<bool>,
}
impl PartnerEntitySourcePage {
    fn record(
        &self,
        reporting_period: Option<crate::K1EntityTaxPeriod>,
        section179_sales: Vec<crate::K1Section179Sale>,
    ) -> PartnerEntitySource {
        PartnerEntitySource {
            reporting_period,
            section179_sales,
            amt_section179_income_carry: self.amt_section179_income_carry.clone(),
            regular: self.regular.clone(),
            alternative: self.alternative.clone(),
            publicly_traded: self.publicly_traded,
            final_k1: self.final_k1,
            interest_term: self.interest_term,
            disposition: self.disposition.clone(),
            sale: self.sale.clone(),
            schedule_k3: self.schedule_k3.clone(),
            credits: self.credits.iter().cloned().map(Into::into).collect(),
            investment_property: self.investment_property.clone(),
            investment_confirmed: self
                .investment_confirmed
                .map(|value| matches!(value, crate::K1InvestmentConfirmation::Yes)),
            housing_confirmed: self.housing_confirmed,
        }
    }
    fn from_record(source: &PartnerEntitySource) -> Self {
        Self {
            amt_section179_income_carry: source.amt_section179_income_carry.clone(),
            regular: source.regular.clone(),
            alternative: source.alternative.clone(),
            publicly_traded: source.publicly_traded,
            final_k1: source.final_k1,
            interest_term: source.interest_term,
            disposition: source.disposition.clone(),
            sale: source.sale.clone(),
            schedule_k3: source.schedule_k3.clone(),
            credits: source.credits.iter().cloned().map(Into::into).collect(),
            investment_property: source.investment_property.clone(),
            investment_confirmed: source.investment_confirmed.map(Into::into),
            housing_confirmed: source.housing_confirmed,
        }
    }
}

// Ephemeral Page conversion reuses the canonical code-specific credit input.
// The accepted source list and confirmation fact remain the single authority.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ScorpEntitySourcePage {
    /// Actual AMT section179 income-limit carryover history from this K-1.
    /// This is independent of regular carryovers and of any unavailable component attribution.
    /// [NullText]
    /// This entity's AMT carryover history is unavailable.
    /// [SetText]
    /// The actual AMT carryover history is
    pub amt_section179_income_carry: Option<crate::Section179IncomeCarryHistory>,
    /// Whole-shareholding regular-tax stock basis source.
    pub regular: ScorpEntityBasis,
    /// Whole-shareholding separate AMT stock basis source.
    /// [NullText]
    /// The separate AMT stock basis source is unavailable.
    /// [SetText]
    /// The actual AMT stock basis source is
    pub alternative: Option<ScorpEntityBasis>,
    /// Actual shareholder loans, each retaining its same-source AMT basis.
    pub loans: Vec<ScorpEntityLoan>,
    /// This is the final K-1.
    pub final_k1: bool,
    /// Established holding period of the shares.
    /// [NullText]
    /// No share holding period is established.
    /// [SetText]
    /// The share holding period is
    pub stock_term: Option<crate::GainTerm>,
    /// What happened to the whole shareholding during the year.
    pub disposition: crate::K1Disposition,
    /// Actual Schedule K-3 source.
    /// [NullText]
    /// There is no Schedule K-3 source.
    /// [SetText]
    /// The Schedule K-3 source is
    pub schedule_k3: Option<crate::ScheduleK3>,
    /// Actual entity-reported credit source records.
    pub credits: Vec<crate::K1CreditPage>,
    /// Actual investment-credit property source records.
    pub investment_property: Vec<crate::K1InvestmentProperty>,
    /// Actual investment-property confirmation.
    /// [NullText]
    /// The investment-property source is unconfirmed.
    /// [SetText]
    /// The source confirmation is
    pub investment_confirmed: Option<crate::K1InvestmentConfirmation>,
    /// Actual low-income housing acquisition confirmation.
    /// [NullText]
    /// The housing source is unconfirmed.
    /// [SetText]
    /// The housing confirmation is
    pub housing_confirmed: Option<bool>,
}
impl ScorpEntitySourcePage {
    fn record(
        &self,
        reporting_period: Option<crate::K1EntityTaxPeriod>,
        section179_sales: Vec<crate::K1Section179Sale>,
    ) -> ScorpEntitySource {
        ScorpEntitySource {
            reporting_period,
            section179_sales,
            amt_section179_income_carry: self.amt_section179_income_carry.clone(),
            regular: self.regular.clone(),
            alternative: self.alternative.clone(),
            loans: self.loans.clone(),
            final_k1: self.final_k1,
            stock_term: self.stock_term,
            disposition: self.disposition.clone(),
            schedule_k3: self.schedule_k3.clone(),
            credits: self.credits.iter().cloned().map(Into::into).collect(),
            investment_property: self.investment_property.clone(),
            investment_confirmed: self
                .investment_confirmed
                .map(|value| matches!(value, crate::K1InvestmentConfirmation::Yes)),
            housing_confirmed: self.housing_confirmed,
        }
    }
    fn from_record(source: &ScorpEntitySource) -> Self {
        Self {
            amt_section179_income_carry: source.amt_section179_income_carry.clone(),
            regular: source.regular.clone(),
            alternative: source.alternative.clone(),
            loans: source.loans.clone(),
            final_k1: source.final_k1,
            stock_term: source.stock_term,
            disposition: source.disposition.clone(),
            schedule_k3: source.schedule_k3.clone(),
            credits: source.credits.iter().cloned().map(Into::into).collect(),
            investment_property: source.investment_property.clone(),
            investment_confirmed: source.investment_confirmed.map(Into::into),
            housing_confirmed: source.housing_confirmed,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1CurrentProperty {
    /// Original current section1231 source net, excluding separately entered section179 sale records.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = -9223372036854775807)]
    pub net: i64,
    /// Original business-asset unrecaptured section1250 rate component, not additional income.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub unrecaptured: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1CurrentStatementTotals {
    /// Original current box1 amount before owner limits.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = -9223372036854775807)]
    #[libertas_llm_doc_field(blank = 0)]
    pub ordinary: i64,
    /// Original current box2 amount before owner limits.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = -9223372036854775807)]
    #[libertas_llm_doc_field(blank = 0)]
    pub rental_real_estate: i64,
    /// Original current box3 amount before owner limits.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = -9223372036854775807)]
    #[libertas_llm_doc_field(blank = 0)]
    pub other_rental: i64,
    /// Original current business/rental property source control, excluding carryovers.
    /// [NullText]
    /// No current business-property source is reported.
    /// [SetText]
    /// The original current business-property source is
    pub property: Option<K1CurrentProperty>,
    /// Original current portfolio source control totals.
    pub portfolio: K1StatementPortfolio,
    /// Original tax-exempt interest control.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_llm_doc_field(blank = 0)]
    pub exempt_interest: i64,
    /// Original other tax-exempt income control.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_llm_doc_field(blank = 0)]
    pub exempt_other: i64,
    /// Original noncapital nondeductible expense control.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_llm_doc_field(blank = 0)]
    pub nondeductible: i64,
    /// Original current section179 amount, excluding business-income carries.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_llm_doc_field(blank = 0)]
    pub section179: i64,
    /// Original current cash charity amount.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_llm_doc_field(blank = 0)]
    pub cash_charity: i64,
    /// Original current foreign-tax amount.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_llm_doc_field(blank = 0)]
    pub foreign_taxes: i64,
    /// Original existing-code income and deduction control totals.
    pub items: K1StatementItems,
}

/// Availability of the actual current fuel-credit source.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
// The canonical source payload stays inline in the normal App wire model.
#[allow(clippy::large_enum_variant)]
pub enum K1CurrentFuelSource {
    /// No current fuel-credit source is reported.
    NotReported,
    /// The actual current fuel-credit source is reported.
    Reported { source: crate::PartnerFuelTaxCredit },
}
impl K1CurrentFuelSource {
    fn valid(&self) -> bool {
        match self {
            Self::NotReported => true,
            Self::Reported { source } => source.valid(),
        }
    }
}
/// Availability of the actual prior fuel-credit recovery source.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum K1PriorFuelSource {
    /// No prior fuel-credit recovery source is reported.
    NotReported,
    /// The actual prior fuel-credit recovery source is reported.
    Reported { source: crate::PartnerPriorFuelCredit },
}
impl K1PriorFuelSource {
    fn valid(&self) -> bool {
        match self {
            Self::NotReported => true,
            Self::Reported { source } => source.valid(),
        }
    }
}

#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct PartnerPersonSource {
    /// Original guaranteed service-payment control.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    /// [NullText]
    /// This personal source amount is unavailable.
    /// [SetText]
    /// The actual personal source amount is
    pub guaranteed_services: Option<i64>,
    /// Original guaranteed capital-payment control.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    /// [NullText]
    /// This personal source amount is unavailable.
    /// [SetText]
    /// The actual personal source amount is
    pub guaranteed_capital: Option<i64>,
    /// Original box13 codeR payment.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    /// [NullText]
    /// This personal source amount is unavailable.
    /// [SetText]
    /// The actual personal source amount is
    pub pensions: Option<i64>,
    /// Actual health-premium subset already included in service payments.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    /// [NullText]
    /// This personal source amount is unavailable.
    /// [SetText]
    /// The actual personal source amount is
    pub health_guaranteed: Option<i64>,
    /// Actual health-premium subset charged against cash distributions.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    /// [NullText]
    /// This personal source amount is unavailable.
    /// [SetText]
    /// The actual personal source amount is
    #[libertas_read_only]
    pub health_distributions: Option<i64>,
    /// Original deferred-compensation additional tax source.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    /// [NullText]
    /// This personal source amount is unavailable.
    /// [SetText]
    /// The actual personal source amount is
    pub deferred_compensation_tax: Option<i64>,
    /// Original conservation reserve program source.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    /// [NullText]
    /// This personal source amount is unavailable.
    /// [SetText]
    /// The actual personal source amount is
    pub crp_payments: Option<i64>,
    /// Actual limited-partner treatment under section1402(a)(13) is established.
    /// [NullText]
    /// The actual limited-partner treatment is unavailable.
    /// [SetText]
    /// The actual limited-partner treatment is
    pub limited_for_se: Option<bool>,
    /// Original box14 codeA control before partner-level adjustments.
    /// [NullText]
    /// The reported net self-employment source is unavailable.
    /// [SetText]
    /// The actual reported net self-employment source is
    #[libertas_money("USD", 2)]
    #[libertas_number(min = -9223372036854775807)]
    pub reported_se: Option<i64>,
    /// Original retained SE-character controls, each stage separate.
    pub se_history: K1RetainedSeHistory,
    /// Original gross self-employment income control.
    /// [NullText]
    /// The original gross self-employment source is unavailable.
    /// [SetText]
    /// The original gross self-employment source is
    pub se_gross: Option<Vec<crate::PartnerSeGross>>,
    /// Actual current fuel-credit source.
    /// [NullText]
    /// The current fuel-credit source is unavailable.
    /// [SetText]
    /// The actual current fuel-credit source is
    pub fuel_tax_credit: Option<K1CurrentFuelSource>,
    /// Actual prior fuel-credit recovery source.
    /// [NullText]
    /// The prior fuel-credit recovery source is unavailable.
    /// [SetText]
    /// The actual prior fuel-credit recovery source is
    pub prior_fuel_credit: Option<K1PriorFuelSource>,
}

#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct ScorpPersonSource {
    /// Actual federal backup withholding, included once as a payment.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    /// [NullText]
    /// This personal source amount is unavailable.
    /// [SetText]
    /// The actual personal source amount is
    pub withholding: Option<i64>,
    /// Actual specified private-activity bond interest included in tax-exempt interest.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    /// [NullText]
    /// This personal source amount is unavailable.
    /// [SetText]
    /// The actual personal source amount is
    pub private_activity: Option<i64>,
}

// Ephemeral rows on the existing benefits question. Accepted payment facts
// still live once on the actual parent; readonly list membership is server-owned.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1ActivityPartnerBenefits {
    /// Actual partnership source
    #[libertas_hidden]
    #[libertas_read_only]
    pub id: i64,
    /// Partnership
    #[libertas_formatted_text]
    #[libertas_read_only]
    #[libertas_output_only]
    pub name: Vec<u8>,
    /// Actual box13 codeM payment, already included in service payments.
    /// [NullText]
    /// This source is unavailable.
    /// [SetText]
    /// The actual source is
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_read_only]
    pub medical_insurance: Option<i64>,
    /// Actual box13 codeR source payment.
    /// [NullText]
    /// This source is unavailable.
    /// [SetText]
    /// The actual source is
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_read_only]
    pub pensions: Option<i64>,
    /// Actual cash distribution source
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub cash_distributions: i64,
    /// Actual health plan and premium source; no plan is a separate known choice.
    /// [NullText]
    /// This source is unavailable.
    /// [SetText]
    /// The actual source is
    pub health: Option<crate::SeHealth>,
    /// Actual premiums charged against the partnership cash distributions.
    /// [NullText]
    /// This source is unavailable.
    /// [SetText]
    /// The actual source is
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub from_distributions: Option<i64>,
    /// Actual retirement choice; no plan is a separate known choice.
    /// [NullText]
    /// This source is unavailable.
    /// [SetText]
    /// The actual source is
    pub retirement: Option<crate::PartnerPlan>,
}
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1ActivityScorpBenefits {
    /// Actual S corporation source
    #[libertas_hidden]
    #[libertas_read_only]
    pub activity: i64,
    /// S corporation
    #[libertas_formatted_text]
    #[libertas_read_only]
    #[libertas_output_only]
    pub name: Vec<u8>,
    /// Actual corporation-established health plan and premium source.
    /// [NullText]
    /// This source is unavailable.
    /// [SetText]
    /// The actual source is
    pub health: Option<crate::SeHealth>,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
// Complete source payloads stay inline, matching the normal App wire model.
#[allow(clippy::large_enum_variant)]
pub enum K1ComponentStatement {
    /// The actual component statement or retained source partition is unavailable.
    Unavailable,
    /// Actual documented distinct-activity source records.
    Entered { source: K1ActivitySources },
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PartnerActivityK1 {
    /// Genuine whole-entity-interest source facts.
    pub entity: PartnerEntitySource,
    /// Original regular-tax current statement controls.
    pub current: K1CurrentStatementTotals,
    /// Original separate AMT current statement controls.
    /// [NullText]
    /// The separate AMT current source is unavailable.
    /// [SetText]
    /// The actual separate AMT current source is
    pub alternative_current: Option<K1CurrentStatementTotals>,
    /// The actual distinct activity source statement.
    pub activities: K1ComponentStatement,
    /// Actual personal payment and benefit source facts.
    /// [NullText]
    /// The personal source facts are unavailable.
    /// [SetText]
    /// The actual personal source facts are
    pub person: Option<PartnerPersonSource>,
    /// Other K-1 boxes requiring their actual source worksheets.
    pub unentered: Vec<crate::PartnerK1Item>,
    /// Additional situations requiring their actual source worksheets.
    pub special: Vec<crate::PartnerSpecial>,
    /// Actual health-plan facts owned by Self-employed benefits, not this K-1 page.
    /// [NullText]
    /// The health-plan source has not been established.
    /// [SetText]
    /// The established health-plan source is
    pub health_insurance: Option<crate::BusinessHealthInsurance>,
    /// Actual retirement-plan facts owned by Self-employed benefits, not this K-1 page.
    /// [NullText]
    /// The retirement-plan source has not been established.
    /// [SetText]
    /// The established retirement-plan source is
    pub retirement: Option<crate::PartnerRetirement>,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ScorpActivityK1 {
    /// Genuine whole-entity-interest source facts.
    pub entity: ScorpEntitySource,
    /// Original regular-tax current statement controls.
    pub current: K1CurrentStatementTotals,
    /// Original separate AMT current statement controls.
    /// [NullText]
    /// The separate AMT current source is unavailable.
    /// [SetText]
    /// The actual separate AMT current source is
    pub alternative_current: Option<K1CurrentStatementTotals>,
    /// The actual distinct activity source statement.
    pub activities: K1ComponentStatement,
    /// Actual personal payment and benefit source facts.
    /// [NullText]
    /// The personal source facts are unavailable.
    /// [SetText]
    /// The actual personal source facts are
    pub person: Option<ScorpPersonSource>,
    /// Other K-1 boxes requiring their actual source worksheets.
    pub unentered: Vec<crate::ScorpK1Item>,
    /// Additional situations requiring their actual source worksheets.
    pub special: Vec<crate::ScorpSpecial>,
    /// Actual health-plan facts owned by Self-employed benefits, not this K-1 page.
    /// [NullText]
    /// The health-plan source has not been established.
    /// [SetText]
    /// The established health-plan source is
    pub health_insurance: Option<crate::BusinessHealthInsurance>,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
// Complete source payloads stay inline, matching the normal App wire model.
#[allow(clippy::large_enum_variant)]
pub enum K1ComponentStatementPage {
    /// The actual component statement or retained source partition is unavailable.
    Unavailable,
    /// Actual documented distinct-activity source records.
    Entered { source: K1ActivitySourcesPage },
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1ComponentDetailsPage {
    /// Actual accepted same-parent source choices; fresh rows start with the supplied empty catalog.
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_copy_from("$.empty_k1_activity_choices")]
    pub choices: K1ActivityChoiceContext,
    /// Actual distinct activity statement and retained sources.
    pub statement: K1ComponentStatementPage,
}

#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read the entity name and the marked current source controls from this Schedule K-1 (Form 1065) and its actual attached statement. Verify the document year against the supplied facts and report a mismatch or illegible or conflicting values as an error. Keep every printed sign; parentheses indicate a loss. Ordinary, rental real estate and other rental are boxes 1, 2 and 3. Tax-exempt income and nondeductible expense must be explicitly identified by the source. Read the current section 179, cash-charity and foreign-tax amounts only when the source establishes the matching current aggregate; do not combine unlike codes, invent an allocation or infer an unavailable amount. A clearly blank marked amount is zero. Read section179_sales from the actual attached fully taxable property sale statement: description, acquired and sold dates and regular price, cost plus selling expense, depreciation excluding section179, and passed-through section179. Return an empty list only when that statement reports no such sale. Never import alternative AMT source facts or accepted sale IDs, and never invent a sale-to-activity association. Read reporting_period only from this entity's actual current beginning and ending dates; return null when unavailable and never borrow another source's dates or the individual's calendar year. Read only marked fields. Never infer or replace opening basis, loans, at-risk or passive groups, participation, component or QBI block identities, historical balances, actual component allocations, AMT differences, personal benefit facts or elections. Those are separate records the filer supplies. Leave every unmarked value unchanged.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    accept_multiple_inputs = true
)]
pub struct K1PartnershipActivitiesPage {
    /// Actual physical section179 sale sources, each retaining its regular and AMT availability.
    /// Save new sources before selecting their activity associations.
    #[libertas_llm_doc_field]
    pub section179_sales: Vec<crate::K1Section179Sale>,
    /// This entity's actual current tax period, from its own K-1 records.
    /// [NullText]
    /// This entity's tax period is unavailable.
    /// [SetText]
    /// This entity's actual tax period is
    #[libertas_llm_doc_field]
    pub reporting_period: Option<crate::K1EntityTaxPeriod>,
    /// Entity name on the actual K-1.
    #[libertas_llm_doc_field]
    #[libertas_size(min = 1)]
    pub name: String,
    /// Genuine whole-interest source facts; no aggregate at-risk worksheet.
    pub entity: PartnerEntitySourcePage,
    /// Original current regular-tax statement controls.
    #[libertas_llm_doc_field]
    pub current: K1CurrentStatementTotals,
    /// Original separate AMT current statement controls.
    /// [NullText]
    /// The separate AMT current source is unavailable.
    /// [SetText]
    /// The actual separate AMT current source is
    pub alternative_current: Option<K1CurrentStatementTotals>,
    /// Actual distinct activities, groups and reported QBI source blocks.
    pub activities: K1ComponentDetailsPage,
    /// Actual personal payment and benefit source facts.
    /// [NullText]
    /// The personal source facts are unavailable.
    /// [SetText]
    /// The actual personal source facts are
    pub person: Option<PartnerPersonSource>,
    /// Other actual K-1 boxes requiring additional source worksheets.
    pub unentered: Vec<crate::PartnerK1Item>,
    /// Additional actual situations requiring their source worksheets.
    pub special: Vec<crate::PartnerSpecial>,
}

impl K1PartnershipActivitiesPage {
    pub(crate) fn record(&self) -> PartnerActivityK1 {
        PartnerActivityK1 {
            entity: self.entity.record(self.reporting_period, self.section179_sales.clone()),
            current: self.current.clone(),
            alternative_current: self.alternative_current.clone(),
            activities: self.activities.record(),
            person: self.person.clone(),
            unentered: self.unentered.clone(),
            special: self.special.clone(),
            health_insurance: None,
            retirement: None,
        }
    }
    pub(crate) fn from_record(name: &str, source: &PartnerActivityK1) -> Self {
        Self {
            name: name.into(),
            reporting_period: source.entity.reporting_period,
            section179_sales: source.entity.section179_sales.clone(),
            entity: PartnerEntitySourcePage::from_record(&source.entity),
            current: source.current.clone(),
            alternative_current: source.alternative_current.clone(),
            activities: K1ComponentDetailsPage::from_record(
                &source.activities,
                &source.entity.investment_property,
                &source.entity.section179_sales,
            ),
            person: source.person.clone(),
            unentered: source.unentered.clone(),
            special: source.special.clone(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read the entity name and the marked current source controls from this Schedule K-1 (Form 1120-S) and its actual attached statement. Verify the document year against the supplied facts and report a mismatch or illegible or conflicting values as an error. Keep every printed sign; parentheses indicate a loss. Ordinary, rental real estate and other rental are boxes 1, 2 and 3. Tax-exempt income and nondeductible expense must be explicitly identified by the source. Read the current section 179, cash-charity and foreign-tax amounts only when the source establishes the matching current aggregate; do not combine unlike codes, invent an allocation or infer an unavailable amount. A clearly blank marked amount is zero. Read section179_sales from the actual attached fully taxable property sale statement: description, acquired and sold dates and regular price, cost plus selling expense, depreciation excluding section179, and passed-through section179. Return an empty list only when that statement reports no such sale. Never import alternative AMT source facts or accepted sale IDs, and never invent a sale-to-activity association. Read reporting_period only from this entity's actual current beginning and ending dates; return null when unavailable and never borrow another source's dates or the individual's calendar year. Read only marked fields. Never infer or replace opening basis, loans, at-risk or passive groups, participation, component or QBI block identities, historical balances, actual component allocations, AMT differences, personal benefit facts or elections. Those are separate records the filer supplies. Leave every unmarked value unchanged.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    accept_multiple_inputs = true
)]
pub struct K1ScorpActivitiesPage {
    /// Actual physical section179 sale sources, each retaining its regular and AMT availability.
    /// Save new sources before selecting their activity associations.
    #[libertas_llm_doc_field]
    pub section179_sales: Vec<crate::K1Section179Sale>,
    /// This entity's actual current tax period, from its own K-1 records.
    /// [NullText]
    /// This entity's tax period is unavailable.
    /// [SetText]
    /// This entity's actual tax period is
    #[libertas_llm_doc_field]
    pub reporting_period: Option<crate::K1EntityTaxPeriod>,
    /// Entity name on the actual K-1.
    #[libertas_llm_doc_field]
    #[libertas_size(min = 1)]
    pub name: String,
    /// Genuine whole-interest source facts; no aggregate at-risk worksheet.
    pub entity: ScorpEntitySourcePage,
    /// Original current regular-tax statement controls.
    #[libertas_llm_doc_field]
    pub current: K1CurrentStatementTotals,
    /// Original separate AMT current statement controls.
    /// [NullText]
    /// The separate AMT current source is unavailable.
    /// [SetText]
    /// The actual separate AMT current source is
    pub alternative_current: Option<K1CurrentStatementTotals>,
    /// Actual distinct activities, groups and reported QBI source blocks.
    pub activities: K1ComponentDetailsPage,
    /// Actual personal payment and benefit source facts.
    /// [NullText]
    /// The personal source facts are unavailable.
    /// [SetText]
    /// The actual personal source facts are
    pub person: Option<ScorpPersonSource>,
    /// Other actual K-1 boxes requiring additional source worksheets.
    pub unentered: Vec<crate::ScorpK1Item>,
    /// Additional actual situations requiring their source worksheets.
    pub special: Vec<crate::ScorpSpecial>,
}

impl K1ScorpActivitiesPage {
    pub(crate) fn record(&self) -> ScorpActivityK1 {
        ScorpActivityK1 {
            entity: self.entity.record(self.reporting_period, self.section179_sales.clone()),
            current: self.current.clone(),
            alternative_current: self.alternative_current.clone(),
            activities: self.activities.record(),
            person: self.person.clone(),
            unentered: self.unentered.clone(),
            special: self.special.clone(),
            health_insurance: None,
        }
    }
    pub(crate) fn from_record(name: &str, source: &ScorpActivityK1) -> Self {
        Self {
            name: name.into(),
            reporting_period: source.entity.reporting_period,
            section179_sales: source.entity.section179_sales.clone(),
            entity: ScorpEntitySourcePage::from_record(&source.entity),
            current: source.current.clone(),
            alternative_current: source.alternative_current.clone(),
            activities: K1ComponentDetailsPage::from_record(
                &source.activities,
                &source.entity.investment_property,
                &source.entity.section179_sales,
            ),
            person: source.person.clone(),
            unentered: source.unentered.clone(),
            special: source.special.clone(),
        }
    }
}

impl K1ActivitySourcesPage {
    fn record(&self) -> K1ActivitySources {
        K1ActivitySources {
            components: self.components.iter().map(K1StatementActivityPage::record).collect(),
            risk_groups: self.risk_groups.clone(),
            passive_groups: self.passive_groups.clone(),
            qbi_blocks: self.qbi_blocks.iter().map(K1QbiBlockPage::record).collect(),
            section179_sale_sources: self
                .section179_sale_sources
                .as_ref()
                .map(|v| v.iter().map(K1Section179SaleSourcePage::record).collect()),
            investment_property_sources: self
                .investment_property_sources
                .as_ref()
                .map(|v| v.iter().map(K1InvestmentPropertySourcePage::record).collect()),
            entity_only: self.entity_only.clone(),
            alternative_entity_only: self.alternative_entity_only.clone(),
        }
    }
    fn from_record(source: &K1ActivitySources) -> Self {
        Self {
            components: source
                .components
                .iter()
                .map(K1StatementActivityPage::from_record)
                .collect(),
            risk_groups: source.risk_groups.clone(),
            passive_groups: source.passive_groups.clone(),
            qbi_blocks: source.qbi_blocks.iter().map(K1QbiBlockPage::from_record).collect(),
            section179_sale_sources: source
                .section179_sale_sources
                .as_ref()
                .map(|v| v.iter().map(K1Section179SaleSourcePage::from_record).collect()),
            investment_property_sources: source
                .investment_property_sources
                .as_ref()
                .map(|v| v.iter().map(K1InvestmentPropertySourcePage::from_record).collect()),
            entity_only: source.entity_only.clone(),
            alternative_entity_only: source.alternative_entity_only.clone(),
        }
    }
}
impl Default for K1ComponentDetailsPage {
    fn default() -> Self {
        Self {
            choices: K1ActivityChoiceContext::default(),
            statement: K1ComponentStatementPage::Unavailable,
        }
    }
}
impl K1ComponentDetailsPage {
    fn record(&self) -> K1ComponentStatement {
        match &self.statement {
            K1ComponentStatementPage::Unavailable => K1ComponentStatement::Unavailable,
            K1ComponentStatementPage::Entered { source } => {
                K1ComponentStatement::Entered { source: source.record() }
            }
        }
    }
    fn from_record(
        source: &K1ComponentStatement,
        properties: &[crate::K1InvestmentProperty],
        sales: &[crate::K1Section179Sale],
    ) -> Self {
        Self {
            choices: K1ActivityChoiceContext::from_source_properties(
                source.source(),
                properties,
                sales,
            ),
            statement: match source {
                K1ComponentStatement::Unavailable => K1ComponentStatementPage::Unavailable,
                K1ComponentStatement::Entered { source } => K1ComponentStatementPage::Entered {
                    source: K1ActivitySourcesPage::from_record(source),
                },
            },
        }
    }
}

impl K1ComponentStatement {
    pub(crate) fn source(&self) -> Option<&K1ActivitySources> {
        match self {
            Self::Unavailable => None,
            Self::Entered { source } => Some(source),
        }
    }
    fn source_mut(&mut self) -> Option<&mut K1ActivitySources> {
        match self {
            Self::Unavailable => None,
            Self::Entered { source } => Some(source),
        }
    }
}
pub(crate) fn source(income: &crate::FederalIncome) -> Option<&K1ActivitySources> {
    match income {
        crate::FederalIncome::PartnershipActivities { data } => data.activities.source(),
        crate::FederalIncome::ScorpActivities { data } => data.activities.source(),
        _ => None,
    }
}
fn same_source_kind(a: &crate::FederalIncome, b: &crate::FederalIncome) -> bool {
    matches!(
        (a, b),
        (
            crate::FederalIncome::PartnershipActivities { .. },
            crate::FederalIncome::PartnershipActivities { .. }
        ) | (
            crate::FederalIncome::ScorpActivities { .. },
            crate::FederalIncome::ScorpActivities { .. }
        )
    )
}
pub(crate) fn source_ids(income: &crate::FederalIncome) -> Vec<i64> {
    source(income).map_or_else(Vec::new, |v| v.ids().collect())
}
// Metadata follows ids()/ids_mut() exactly; the existing allocator still owns
// uniqueness and assignment once. Identity is hidden, so locate its actual row.
pub(crate) fn source_identity_rows(income: &crate::FederalIncome) -> Vec<ValidationIssue> {
    let Some(source) = source(income) else {
        return Vec::new();
    };
    [
        ("components", source.components.len()),
        ("risk_groups", source.risk_groups.len()),
        ("passive_groups", source.passive_groups.len()),
        ("qbi_blocks", source.qbi_blocks.len()),
    ]
    .into_iter()
    .flat_map(|(field, len)| {
        (0..len).map(move |index| {
            ValidationIssue::new("K1_ACTIVITY_SOURCE_ID_REPEATED")
                .at_list_item(field, index)
                .in_field("source")
                .in_field("statement")
                .in_field("activities")
        })
    })
    .collect()
}
pub(crate) fn source_ids_mut(income: &mut crate::FederalIncome) -> Vec<&mut i64> {
    let source = match income {
        crate::FederalIncome::PartnershipActivities { data } => data.activities.source_mut(),
        crate::FederalIncome::ScorpActivities { data } => data.activities.source_mut(),
        _ => None,
    };
    source.map_or_else(Vec::new, |v| v.ids_mut().collect())
}
pub(crate) fn identity_issue(
    entry: &crate::FederalIncomeEntry,
    accepted: &[crate::FederalIncomeEntry],
) -> Option<ValidationIssue> {
    let value = source(&entry.income)?;
    let old_entry = accepted
        .iter()
        .find(|v| entry.id > 0 && v.id == entry.id && same_source_kind(&entry.income, &v.income));
    let old = old_entry.and_then(|v| source(&v.income));
    value
        .identity_issue(old)
        .or_else(|| {
            property_identity_issue(
                value,
                investment_properties(&entry.income),
                old,
                old_entry.map_or(&[], |v| investment_properties(&v.income)),
            )
        })
        .or_else(|| {
            sale_identity_issue(
                value,
                section179_sales(&entry.income),
                old,
                old_entry.map_or(&[], |v| section179_sales(&v.income)),
            )
        })
        .map(|issue| {
            in_source_field(
                in_source_field(in_source_field(issue, "source"), "statement"),
                "activities",
            )
        })
}
pub(crate) fn page_issue(
    item: &crate::IncomeDocumentItem,
    accepted: &[crate::FederalIncomeEntry],
) -> Option<ValidationIssue> {
    let old = accepted.iter().find(|v| item.id > 0 && v.id == item.id);
    let name = match &item.document {
        crate::IncomeDocument::K1PartnershipActivities(value) => {
            if value
                .person
                .as_ref()
                .and_then(|v| v.health_distributions)
                .is_some_and(|v| Some(v) != old.and_then(|v| managed_payment(&v.income)))
            {
                return Some(
                    ValidationIssue::new("K1_ACTIVITY_BENEFIT_PAYMENT_SOURCE_CHANGED")
                        .at_field("health_distributions")
                        .in_field("person"),
                );
            }
            &value.name
        }
        crate::IncomeDocument::K1ScorpActivities(value) => &value.name,
        _ => return None,
    };
    if !source_name(name) {
        return Some(ValidationIssue::new("K1_ACTIVITY_SOURCE_SIZE_INVALID").at_field("name"));
    }
    // Accepted parent/role authorization runs once on the converted
    // candidate graph before the existing global allocator assigns IDs.
    None
}
pub(crate) fn income_issue(income: &crate::FederalIncome) -> Option<ValidationIssue> {
    let structural = match income {
        crate::FederalIncome::PartnershipActivities { data } => data.issue(),
        crate::FederalIncome::ScorpActivities { data } => data.issue(),
        _ => return None,
    };
    if structural.is_some() {
        return structural;
    }
    for alternative in [false, true] {
        let result = match income {
            crate::FederalIncome::PartnershipActivities { data } => {
                data.before_section179(alternative)
            }
            crate::FederalIncome::ScorpActivities { data } => data.before_section179(alternative),
            _ => unreachable!(),
        };
        if let Err((issue, _)) = result
            && issue.key == ordinary::RANGE_KEY
        {
            return Some(if issue.field.is_some() { issue } else { issue.at_field("activities") });
        }
    }
    None
}

// Bound owner outputs, never raw boxes, are the receiver's cent inputs.
pub(crate) fn section179_amounts_in_range(entries: &[crate::FederalIncomeEntry]) -> bool {
    [false, true].into_iter().all(|alternative| {
        let mut total = 0i64;
        for entry in entries {
            let output = match &entry.income {
                crate::FederalIncome::PartnershipActivities { data } => {
                    data.before_section179(alternative)
                }
                crate::FederalIncome::ScorpActivities { data } => {
                    data.before_section179(alternative)
                }
                _ => continue,
            };
            if let Ok(output) = output {
                for component in output.components {
                    let Some(next) = total.checked_add(component.section179) else {
                        return false;
                    };
                    total = next;
                }
            }
        }
        true
    })
}

// Established benefit facts are the normal benefits section's authority.
// A K1 Page records only its printed payment/subset facts and cannot erase or
// replace those accepted plans when reopened, resaved or converted from Single.
fn managed_payment(income: &crate::FederalIncome) -> Option<i64> {
    match income {
        crate::FederalIncome::Partnership { data } => Some(data.health_distributions),
        crate::FederalIncome::PartnershipActivities { data } => {
            data.person.as_ref().and_then(|v| v.health_distributions)
        }
        _ => None,
    }
}
pub(crate) fn preserve_person_sources(
    income: &mut crate::FederalIncome,
    accepted: Option<&crate::FederalIncome>,
) {
    match income {
        crate::FederalIncome::PartnershipActivities { data } => {
            if let Some(payment) = accepted.and_then(managed_payment) {
                data.person.get_or_insert_with(PartnerPersonSource::default).health_distributions =
                    Some(payment);
            }
            match accepted {
                Some(crate::FederalIncome::PartnershipActivities { data: old }) => {
                    data.health_insurance = old.health_insurance.clone();
                    data.retirement = old.retirement.clone();
                }
                Some(crate::FederalIncome::Partnership { data: old }) => {
                    data.health_insurance = Some(old.health_insurance.clone());
                    data.retirement = Some(old.retirement.clone());
                }
                _ => {}
            }
        }
        crate::FederalIncome::ScorpActivities { data } => match accepted {
            Some(crate::FederalIncome::ScorpActivities { data: old }) => {
                data.health_insurance = old.health_insurance.clone()
            }
            Some(crate::FederalIncome::Scorp { data: old }) => {
                data.health_insurance = Some(old.health_insurance.clone())
            }
            _ => {}
        },
        _ => {}
    }
}
impl K1CurrentStatementTotals {
    fn issue(&self, partnership: bool) -> Option<ValidationIssue> {
        let fields = [
            ("ordinary", SOURCE_SIGNED.contains(&self.ordinary)),
            ("rental_real_estate", SOURCE_SIGNED.contains(&self.rental_real_estate)),
            ("other_rental", SOURCE_SIGNED.contains(&self.other_rental)),
            ("exempt_interest", SOURCE_MONEY.contains(&self.exempt_interest)),
            ("exempt_other", SOURCE_MONEY.contains(&self.exempt_other)),
            ("nondeductible", SOURCE_MONEY.contains(&self.nondeductible)),
            ("section179", SOURCE_MONEY.contains(&self.section179)),
            ("cash_charity", SOURCE_MONEY.contains(&self.cash_charity)),
            ("foreign_taxes", SOURCE_MONEY.contains(&self.foreign_taxes)),
            ("portfolio", self.portfolio.valid()),
            (
                "property",
                self.property.as_ref().is_none_or(|v| {
                    SOURCE_SIGNED.contains(&v.net) && SOURCE_MONEY.contains(&v.unrecaptured)
                }),
            ),
        ];
        if let Some((field, _)) = fields.into_iter().find(|(_, valid)| !valid) {
            return Some(ValidationIssue::new("K1_ACTIVITY_SOURCE_AMOUNT_INVALID").at_field(field));
        }
        self.items.issue(partnership).map(|issue| in_source_field(issue, "items"))
    }
}
impl PartnerEntityBasis {
    fn issue(&self) -> Option<ValidationIssue> {
        for (field, value) in [
            ("opening", self.opening),
            ("contributions", Some(self.contributions)),
            ("opening_liabilities", Some(self.opening_liabilities)),
            ("ending_liabilities", Some(self.ending_liabilities)),
            ("assumed", Some(self.assumed)),
            ("transferred", Some(self.transferred)),
            ("distributions", Some(self.distributions)),
            ("prior_nondeductible", self.prior_nondeductible),
        ] {
            if !known_money(value) {
                return Some(
                    ValidationIssue::new("K1_ACTIVITY_SOURCE_AMOUNT_INVALID").at_field(field),
                );
            }
        }
        if self.complete && (self.opening.is_none() || self.prior_nondeductible.is_none()) {
            return Some(
                ValidationIssue::new("K1_ACTIVITY_COMPLETE_BASIS_SOURCE_MISSING")
                    .at_field("complete"),
            );
        }
        None
    }
}
impl ScorpEntityBasis {
    fn issue(&self) -> Option<ValidationIssue> {
        for (field, value) in [
            ("opening", self.opening),
            ("additions", Some(self.additions)),
            ("distributions", Some(self.distributions)),
            ("prior_nondeductible", self.prior_nondeductible),
        ] {
            if !known_money(value) {
                return Some(
                    ValidationIssue::new("K1_ACTIVITY_SOURCE_AMOUNT_INVALID").at_field(field),
                );
            }
        }
        if !self.losses_first && self.prior_nondeductible.is_some_and(|value| value != 0) {
            return Some(
                ValidationIssue::new("K1_ACTIVITY_SOURCE_AMOUNT_INVALID")
                    .at_field("prior_nondeductible"),
            );
        }
        if self.complete && (self.opening.is_none() || self.prior_nondeductible.is_none()) {
            return Some(
                ValidationIssue::new("K1_ACTIVITY_COMPLETE_BASIS_SOURCE_MISSING")
                    .at_field("complete"),
            );
        }
        None
    }
}
impl PartnerActivityK1 {
    pub(crate) fn benefits_issue(&self) -> Option<ValidationIssue> {
        if self.retirement.as_ref().is_some_and(|v| !v.valid()) {
            return Some(
                ValidationIssue::new("K1_ACTIVITY_SOURCE_AMOUNT_INVALID").at_field("retirement"),
            );
        }
        let person = self.person.as_ref()?;
        if !person.se_history.valid() {
            return Some(
                ValidationIssue::new("K1_ACTIVITY_SOURCE_AMOUNT_INVALID")
                    .at_field("se_history")
                    .in_field("person"),
            );
        }
        for (field, value) in [
            ("guaranteed_services", person.guaranteed_services),
            ("guaranteed_capital", person.guaranteed_capital),
            ("pensions", person.pensions),
            ("health_guaranteed", person.health_guaranteed),
            ("health_distributions", person.health_distributions),
            ("deferred_compensation_tax", person.deferred_compensation_tax),
            ("crp_payments", person.crp_payments),
        ] {
            if !known_money(value) {
                return Some(
                    ValidationIssue::new("K1_ACTIVITY_SOURCE_AMOUNT_INVALID")
                        .at_field(field)
                        .in_field("person"),
                );
            }
        }
        for (field, valid) in [
            ("reported_se", person.reported_se.is_none_or(|v| SOURCE_SIGNED.contains(&v))),
            (
                "se_gross",
                person.se_gross.as_ref().is_none_or(|v| {
                    v.iter().all(|v| SOURCE_MONEY.contains(&v.source().1))
                        && v.iter().enumerate().all(|(index, source)| {
                            !v[..index]
                                .iter()
                                .any(|earlier| earlier.source().0 == source.source().0)
                        })
                }),
            ),
            ("fuel_tax_credit", person.fuel_tax_credit.as_ref().is_none_or(|v| v.valid())),
            ("prior_fuel_credit", person.prior_fuel_credit.as_ref().is_none_or(|v| v.valid())),
        ] {
            if !valid {
                return Some(
                    ValidationIssue::new("K1_ACTIVITY_SOURCE_AMOUNT_INVALID")
                        .at_field(field)
                        .in_field("person"),
                );
            }
        }
        if !known_subset(person.health_guaranteed, person.guaranteed_services) {
            return Some(
                ValidationIssue::new("K1_ACTIVITY_HEALTH_SUBSET_ABOVE_SOURCE")
                    .at_field("health_guaranteed")
                    .in_field("person"),
            );
        }
        if !known_subset(person.health_distributions, Some(self.entity.regular.distributions))
            || self
                .entity
                .alternative
                .as_ref()
                .is_some_and(|v| !known_subset(person.health_distributions, Some(v.distributions)))
        {
            return Some(
                ValidationIssue::new("K1_ACTIVITY_HEALTH_SUBSET_ABOVE_SOURCE")
                    .at_field("health_distributions")
                    .in_field("person"),
            );
        }
        None
    }

    #[cfg(test)]
    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }
    fn issue(&self) -> Option<ValidationIssue> {
        if let Some(issue) = self.entity.regular.issue() {
            return Some(issue.in_field("regular").in_field("entity"));
        }
        if let Some(issue) = self.entity.alternative.as_ref().and_then(|value| value.issue()) {
            return Some(issue.in_field("alternative").in_field("entity"));
        }
        if let Some(issue) =
            self.entity.amt_section179_income_carry.as_ref().and_then(|history| history.issue())
        {
            return Some(issue.in_field("amt_section179_income_carry").in_field("entity"));
        }
        if self.entity.reporting_period.as_ref().is_some_and(|period| !period.valid()) {
            return Some(
                ValidationIssue::new("K1_ENTITY_PERIOD_DATES").at_field("reporting_period"),
            );
        }
        if let Some(issue) =
            crate::K1Section179Sale::source_list_issue(&self.entity.section179_sales)
        {
            return Some(issue);
        }
        let intrinsic = |field: &'static str, valid: bool| {
            (!valid)
                .then(|| ValidationIssue::new("K1_ACTIVITY_SOURCE_AMOUNT_INVALID").at_field(field))
        };
        if let Some(issue) = [
            intrinsic("disposition", self.entity.disposition.valid()),
            intrinsic("sale", self.entity.sale.valid()),
            self.entity
                .schedule_k3
                .as_ref()
                .and_then(|value| {
                    value.issue().or_else(|| {
                        (!value.valid()).then(|| {
                            ValidationIssue::new("K1_ACTIVITY_SOURCE_AMOUNT_INVALID").at_record()
                        })
                    })
                })
                .map(|issue| issue.in_field("schedule_k3")),
            crate::K1Credit::issue_list(&self.entity.credits),
            intrinsic(
                "investment_property",
                crate::K1InvestmentProperty::valid_list(&self.entity.investment_property),
            ),
        ]
        .into_iter()
        .flatten()
        .next()
        {
            return Some(issue.in_field("entity"));
        }
        if let Some(issue) = crate::PartnerSpecial::issue_list(&self.special) {
            return Some(issue);
        }
        if self.health_insurance.as_ref().is_some_and(|v| !v.valid()) {
            // Managed plans have no editable field on the K1 source question.
            return Some(ValidationIssue::new("K1_ACTIVITY_SOURCE_AMOUNT_INVALID"));
        }
        if self.unentered.iter().enumerate().any(|(i, v)| self.unentered[..i].contains(v)) {
            return Some(
                ValidationIssue::new("K1_ACTIVITY_SOURCE_CODE_REPEATED").at_field("unentered"),
            );
        }
        if let Some(issue) = self.benefits_issue() {
            return Some(issue);
        }
        self.current
            .issue(true)
            .map(|issue| in_source_field(issue, "current"))
            .or_else(|| {
                self.alternative_current
                    .as_ref()
                    .and_then(|v| v.issue(true))
                    .map(|issue| in_source_field(issue, "alternative_current"))
            })
            .or_else(|| {
                self.activities.source().and_then(|v| v.source_issue(true)).map(|issue| {
                    in_source_field(
                        in_source_field(in_source_field(issue, "source"), "statement"),
                        "activities",
                    )
                })
            })
            .or_else(|| {
                self.activities
                    .source()
                    .and_then(|v| self.current.reconcile_issue(v, false))
                    .map(|issue| in_source_field(issue, "current"))
            })
            .or_else(|| {
                self.activities
                    .source()
                    .and_then(|v| {
                        self.alternative_current.as_ref().and_then(|c| c.reconcile_issue(v, true))
                    })
                    .map(|issue| in_source_field(issue, "alternative_current"))
            })
    }
}
impl ScorpActivityK1 {
    fn issue(&self) -> Option<ValidationIssue> {
        if let Some(issue) = self.entity.regular.issue() {
            return Some(issue.in_field("regular").in_field("entity"));
        }
        if let Some(issue) = self.entity.alternative.as_ref().and_then(|value| value.issue()) {
            return Some(issue.in_field("alternative").in_field("entity"));
        }
        if let Some(issue) =
            self.entity.amt_section179_income_carry.as_ref().and_then(|history| history.issue())
        {
            return Some(issue.in_field("amt_section179_income_carry").in_field("entity"));
        }
        if self.entity.reporting_period.as_ref().is_some_and(|period| !period.valid()) {
            return Some(
                ValidationIssue::new("K1_ENTITY_PERIOD_DATES").at_field("reporting_period"),
            );
        }
        if let Some(issue) =
            crate::K1Section179Sale::source_list_issue(&self.entity.section179_sales)
        {
            return Some(issue);
        }
        let intrinsic = |field: &'static str, valid: bool| {
            (!valid)
                .then(|| ValidationIssue::new("K1_ACTIVITY_SOURCE_AMOUNT_INVALID").at_field(field))
        };
        if let Some(issue) = [
            intrinsic("disposition", self.entity.disposition.valid()),
            self.entity
                .schedule_k3
                .as_ref()
                .and_then(|value| {
                    value.issue().or_else(|| {
                        (!value.valid()).then(|| {
                            ValidationIssue::new("K1_ACTIVITY_SOURCE_AMOUNT_INVALID").at_record()
                        })
                    })
                })
                .map(|issue| issue.in_field("schedule_k3")),
            crate::K1Credit::issue_list(&self.entity.credits),
            intrinsic(
                "investment_property",
                crate::K1InvestmentProperty::valid_list(&self.entity.investment_property),
            ),
        ]
        .into_iter()
        .flatten()
        .next()
        {
            return Some(issue.in_field("entity"));
        }
        if let Some(issue) = crate::ScorpSpecial::issue_list(&self.special) {
            return Some(issue);
        }
        if self.health_insurance.as_ref().is_some_and(|v| !v.valid()) {
            // Managed plans have no editable field on the K1 source question.
            return Some(ValidationIssue::new("K1_ACTIVITY_SOURCE_AMOUNT_INVALID"));
        }
        if self.unentered.iter().enumerate().any(|(i, v)| self.unentered[..i].contains(v)) {
            return Some(
                ValidationIssue::new("K1_ACTIVITY_SOURCE_CODE_REPEATED").at_field("unentered"),
            );
        }
        if !crate::ScorpLoan::sources_valid(self.entity.loans.iter().map(|v| &v.regular)) {
            return Some(
                ValidationIssue::new("K1_ACTIVITY_SOURCE_AMOUNT_INVALID")
                    .at_field("loans")
                    .in_field("entity"),
            );
        }
        if let Some(index) = self.entity.loans.iter().position(|v| {
            v.alternative_basis.is_some_and(|basis| !(0..=v.regular.principal).contains(&basis))
        }) {
            return Some(
                ValidationIssue::new("K1_ACTIVITY_SOURCE_AMOUNT_INVALID")
                    .at_field("alternative_basis")
                    .in_list("loans", index)
                    .in_field("entity"),
            );
        }
        if let Some(person) = &self.person {
            for (field, value) in
                [("withholding", person.withholding), ("private_activity", person.private_activity)]
            {
                if !known_money(value) {
                    return Some(
                        ValidationIssue::new("K1_ACTIVITY_SOURCE_AMOUNT_INVALID")
                            .at_field(field)
                            .in_field("person"),
                    );
                }
            }
            if !known_subset(person.private_activity, Some(self.current.exempt_interest)) {
                return Some(
                    ValidationIssue::new("K1_ACTIVITY_PRIVATE_INTEREST_ABOVE_EXEMPT")
                        .at_field("private_activity")
                        .in_field("person"),
                );
            }
        }
        self.current
            .issue(false)
            .map(|issue| in_source_field(issue, "current"))
            .or_else(|| {
                self.alternative_current
                    .as_ref()
                    .and_then(|v| v.issue(false))
                    .map(|issue| in_source_field(issue, "alternative_current"))
            })
            .or_else(|| {
                self.activities.source().and_then(|v| v.source_issue(false)).map(|issue| {
                    in_source_field(
                        in_source_field(in_source_field(issue, "source"), "statement"),
                        "activities",
                    )
                })
            })
            .or_else(|| {
                self.activities
                    .source()
                    .and_then(|v| self.current.reconcile_issue(v, false))
                    .map(|issue| in_source_field(issue, "current"))
            })
            .or_else(|| {
                self.activities
                    .source()
                    .and_then(|v| {
                        self.alternative_current.as_ref().and_then(|c| c.reconcile_issue(v, true))
                    })
                    .map(|issue| in_source_field(issue, "alternative_current"))
            })
    }
}

impl K1StatementPortfolio {
    fn values(&self) -> [i64; 9] {
        [
            self.interest,
            self.dividends,
            self.qualified_dividends,
            self.royalties,
            self.short,
            self.long,
            self.collectibles,
            self.unrecaptured_partnership,
            self.unrecaptured_fund,
        ]
    }
}
impl K1StatementAmounts {
    fn current_values(&self) -> [i64; 6] {
        [
            self.exempt_interest,
            self.exempt_other,
            self.nondeductible,
            self.section179,
            self.cash_charity,
            self.foreign_taxes,
        ]
    }
}
impl K1EntityOnlyAmounts {
    fn current_values(&self) -> [i64; 6] {
        [
            self.exempt_interest,
            self.exempt_other,
            self.nondeductible,
            0,
            self.cash_charity,
            self.foreign_taxes,
        ]
    }
}
impl K1StatementItems {
    fn reconciles(&self, parts: &[&Self]) -> bool {
        // Printed zero/blank net source codes need not be repeated as invalid
        // zero-valued code rows. Actual opposite signed child sources remain.
        match self {
            Self::Partnership { income, deductions } => {
                let mut source_income = Vec::new();
                let mut source_deductions = Vec::new();
                for part in parts {
                    let Self::Partnership { income, deductions } = part else {
                        return false;
                    };
                    source_income.extend(income.iter());
                    source_deductions.extend(deductions.iter());
                }
                income.iter().chain(source_income.iter().copied()).all(|v| {
                    let expected = income.iter().find(|r| r.code == v.code).map_or(0, |r| r.amount);
                    source_income
                        .iter()
                        .filter(|r| r.code == v.code)
                        .map(|r| exact::whole(r.amount))
                        .sum::<exact::Exact>()
                        == exact::whole(expected)
                }) && deductions.iter().chain(source_deductions.iter().copied()).all(|v| {
                    let expected =
                        deductions.iter().find(|r| r.code == v.code).map_or(0, |r| r.amount);
                    source_deductions
                        .iter()
                        .filter(|r| r.code == v.code)
                        .map(|r| exact::whole(r.amount))
                        .sum::<exact::Exact>()
                        == exact::whole(expected)
                })
            }
            Self::Scorp { income, deductions } => {
                let mut source_income = Vec::new();
                let mut source_deductions = Vec::new();
                for part in parts {
                    let Self::Scorp { income, deductions } = part else {
                        return false;
                    };
                    source_income.extend(income.iter());
                    source_deductions.extend(deductions.iter());
                }
                income.iter().chain(source_income.iter().copied()).all(|v| {
                    let expected = income.iter().find(|r| r.code == v.code).map_or(0, |r| r.amount);
                    source_income
                        .iter()
                        .filter(|r| r.code == v.code)
                        .map(|r| exact::whole(r.amount))
                        .sum::<exact::Exact>()
                        == exact::whole(expected)
                }) && deductions.iter().chain(source_deductions.iter().copied()).all(|v| {
                    let expected =
                        deductions.iter().find(|r| r.code == v.code).map_or(0, |r| r.amount);
                    source_deductions
                        .iter()
                        .filter(|r| r.code == v.code)
                        .map(|r| exact::whole(r.amount))
                        .sum::<exact::Exact>()
                        == exact::whole(expected)
                })
            }
        }
    }
}
impl K1CurrentStatementTotals {
    fn reconcile_issue(
        &self,
        source: &K1ActivitySources,
        alternative: bool,
    ) -> Option<ValidationIssue> {
        let mut operating: [_; 3] = core::array::from_fn(|_| exact::whole(0));
        let mut property: [_; 2] = core::array::from_fn(|_| exact::whole(0));
        let mut portfolio: [_; 9] = core::array::from_fn(|_| exact::whole(0));
        let mut current: [_; 6] = core::array::from_fn(|_| exact::whole(0));
        let mut items = Vec::new();
        for component in &source.components {
            let amounts = if alternative {
                match &component.alternative {
                    K1StatementAmt::SameAsRegular => component.regular.as_ref(),
                    K1StatementAmt::Separate { amounts } => Some(amounts),
                    K1StatementAmt::Unavailable => None,
                }
            } else {
                component.regular.as_ref()
            };
            let amounts = amounts?;
            let kind = match component.kind {
                K1StatementKind::TradeBusiness => 0,
                K1StatementKind::RentalRealEstate => 1,
                K1StatementKind::OtherRental => 2,
            };
            operating[kind] += exact::whole(amounts.operating);
            if let Some(value) = &amounts.property {
                property[0] += exact::whole(value.net);
                property[1] += exact::whole(value.unrecaptured);
            }
            for (total, value) in portfolio.iter_mut().zip(amounts.portfolio.values()) {
                *total += exact::whole(value);
            }
            for (total, value) in current.iter_mut().zip(amounts.current_values()) {
                *total += exact::whole(value);
            }
            items.push(&amounts.items);
        }
        let expected_property = self.property.as_ref().map_or([0; 2], |v| [v.net, v.unrecaptured]);
        for (field, actual, expected) in [
            ("ordinary", &operating[0], self.ordinary),
            ("rental_real_estate", &operating[1], self.rental_real_estate),
            ("other_rental", &operating[2], self.other_rental),
        ] {
            if *actual != exact::whole(expected) {
                return Some(
                    ValidationIssue::new("K1_ACTIVITY_CURRENT_SOURCE_RECONCILE").at_field(field),
                );
            }
        }
        if property != expected_property.map(exact::whole) {
            return Some(
                ValidationIssue::new("K1_ACTIVITY_CURRENT_SOURCE_RECONCILE").at_field("property"),
            );
        }
        let entity_only = if alternative {
            match &source.alternative_entity_only {
                K1EntityOnlyAmt::SameAsRegular => source.entity_only.as_ref(),
                K1EntityOnlyAmt::Separate { amounts } => Some(amounts),
                K1EntityOnlyAmt::Unavailable => None,
            }
        } else {
            source.entity_only.as_ref()
        };
        let entity_only = entity_only?;
        for (total, value) in portfolio.iter_mut().zip(entity_only.portfolio.values()) {
            *total += exact::whole(value);
        }
        for (total, value) in current.iter_mut().zip(entity_only.current_values()) {
            *total += exact::whole(value);
        }
        items.push(&entity_only.items);
        for (index, expected) in self.portfolio.values().into_iter().enumerate() {
            if portfolio[index] != exact::whole(expected) {
                let field = [
                    "interest",
                    "dividends",
                    "qualified_dividends",
                    "royalties",
                    "short",
                    "long",
                    "collectibles",
                    "unrecaptured_partnership",
                    "unrecaptured_fund",
                ][index];
                return Some(
                    ValidationIssue::new("K1_ACTIVITY_CURRENT_SOURCE_RECONCILE")
                        .at_field(field)
                        .in_field("portfolio"),
                );
            }
        }
        for (index, (field, expected)) in [
            ("exempt_interest", self.exempt_interest),
            ("exempt_other", self.exempt_other),
            ("nondeductible", self.nondeductible),
            ("section179", self.section179),
            ("cash_charity", self.cash_charity),
            ("foreign_taxes", self.foreign_taxes),
        ]
        .into_iter()
        .enumerate()
        {
            if current[index] != exact::whole(expected) {
                return Some(
                    ValidationIssue::new("K1_ACTIVITY_CURRENT_SOURCE_RECONCILE").at_field(field),
                );
            }
        }
        if !self.items.reconciles(&items) {
            return Some(
                ValidationIssue::new("K1_ACTIVITY_CURRENT_SOURCE_RECONCILE").at_field("items"),
            );
        }
        None
    }
}

impl K1ComponentQualified {
    fn signs(&self) -> Option<[i64; 5]> {
        match self {
            Self::Partnership { current_income, current_property, records } => Some([
                (*current_income).max(0),
                -(*current_income).min(0),
                (*current_property).max(0),
                -(*current_property).min(0),
                records.qualified_se_gross,
            ]),
            Self::Scorp { current_income, ordinary, property } => {
                Some([*current_income, ordinary.current, property.gain, property.losses.current, 0])
            }
            Self::Unavailable => None,
        }
    }
}

#[path = "k1_activity_ordinary.rs"]
pub(crate) mod ordinary;
