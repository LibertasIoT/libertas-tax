// Ordinary-business component source adapter for partnerships and S corporations.
// One entity basis calculation precedes independent at-risk groups and PAL.
// Current unqualified §179 from actual nonpassive components joins the sole
// taxpayer Form4562 allocator after whole-interest basis. Its bound allowance
// reduces that component's own group only; binding-risk cycles, retained §179,
// passive §179 and qualified/property cohorts remain explicitly unavailable.
// IRS i4562 line12 and i1065sk1 box12 (2025 final structure, 2026 constants).
// Coverage: Schedule K-1 (1065 and 1120-S), multiple ordinary-business activities.
// Whole-interest basis once per tax system, independent risk groups and the
// common return passive pool; known zero SE/personal payments and no current
// QBI statement with explicit zero-qualified retained original-year records.
// Partnership components with documented zero current source parts and no
// passive carry also release original-year qualified basis/risk histories
// through the existing FIFO owner when their groups are nonpassive.
// Fully allowed positive farm or nonfarm SE with explicit no-statement qualification
// joins the existing owner-wide SE/optional-method and QBI gross allocation.
// Nonfarm SE also supports current component qualification: actual block gross
// shares reduce current QBI before the existing eligibility owner.
// Explicit component portfolio ST/LT sources and their actual basis/risk
// histories share entity basis and their bound risk group with operating losses.
// Their separate class outputs never enter the ordinary PAL or QBI inputs.
// Regular/AMT facts and their availability remain independent. Other characters,
// Component-bound current partnership and S-corp statements with explicit positive gross
// operating controls and no competing retained losses use current eligibility.
// Positive partnership components can also release their reconciled basis/risk
// histories; those releases retain original-year qualification separately.
// Known passive participation supports the same retained cohort when each
// affected group's own net absorbs every released loss; it earns no active minimum.
// Pure current ordinary losses also use that path when native basis and risk
// outputs fully allow each component; qualified releases reuse the FIFO owner.
// PTPs, dispositions, other qualified amounts, benefit claims and unavailable facts
// retain the existing named unavailable-output boundary. Native Avro Save,
// Estimate, correction, reopen and restart checks are not live client acceptance.
// Bound nonrental nonpassive investment properties reuse Form3468 and3800;
// their canonical reduction enters the entity basis once, before loss limits.
// Scorp stock/debt basis and restoration reuse Form7203; actual loan advances
// use their assigned risk groups. Repayments use the same source parts when
// Form7203 establishes no gain; repayment and distribution gains remain unavailable.
// Explicit entity-only taxable portfolio interest increases partner outside
// basis or Scorp stock basis once;
// its outside-all-activities source stays outside group at-risk income and PAL.
// It composes with other entity-only income/expenses, current component QBI
// and actual debt records. Independent AMT interest enters the existing
// federal investment-income, AMTI and nonbusiness NOL owners (IRS i6251/i4952).
// Explicit entity-only tax-exempt income increases outside/stock basis
// once, without becoming activity income, at-risk income or taxable income.
// Regular exempt interest enters the existing federal line 2a owner once;
// independent AMT source amounts affect only that system's basis calculation.
// Component-attributed other exempt income enters whole-interest basis once
// and only its explicitly named risk group's investment. It is not activity
// income, passive income, SE or QBI. Entity-only exemptions stay outside risk.
// Scorp excluded cancellation-of-debt income is not eligible tax-exempt income.
// Positive entity-only short/long portfolio gains increase basis once and flow
// to Schedule D through the existing capital owners, outside activity risk/PAL.
// Entity-only capital losses, special-rate amounts and unavailable capital carries
// retain the named source boundary; their activity association is not inferred.
// Explicit entity-only ordinary dividends increase whole-entity basis once;
// their qualified subset only supplies return-level preferential character.
// Separate REG/AMT sources remain independent and outside group risk and PAL.
// Entity-only current nondeductible expenses reduce basis through the native
// partner/stock owners, independently by tax system; they never enter a group.
// Evidence: src/tests/passthrough/k1res/{ordinary_multi,scorp_ordinary_multi,
// component_exempt_other,entity_capital_gains,entity_nondeductible,
// entity_interest_composition}.rs.
// IRS source owners reviewed 2026-10-17:
// https://www.irs.gov/instructions/i1065sk1 (basis worksheet)
// https://www.irs.gov/instructions/i7203
// https://www.irs.gov/instructions/i6198
// https://www.irs.gov/instructions/i8582
use super::*;
use crate::k1_loss::{ClassInput, K1LossClass, LossAmounts};
use crate::kernel::carry::{Amount, Cents};
use crate::kernel::loss_limits::risk::{self, RecaptureHistory, RiskEntry, RiskInput};
use crate::kernel::ratio::{allocate_remaining, dollars, positive};
use crate::kernel::source_fact::SourceFact;

pub(crate) type SourceError = (crate::validation::ValidationIssue, &'static str);
pub(crate) const UNAVAILABLE: SourceError = (
    crate::validation::ValidationIssue {
        key: "ISSUE_K1_ACTIVITY_OWNER_OUTPUT_UNAVAILABLE",
        field: None,
    },
    crate::fixture_english!(
        "This estimate does not yet calculate the combined basis, at-risk and passive limits for these activity records."
    ),
);

pub(crate) const RANGE_KEY: &str =
    "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26";
fn source_error((key, message): (&'static str, &'static str)) -> SourceError {
    (crate::validation::ValidationIssue::new(key), message)
}
fn range_error() -> SourceError {
    (
        crate::validation::ValidationIssue::new(RANGE_KEY),
        crate::fixture_english!(
            "Review the document amounts; they must be within the supported input range."
        ),
    )
}
fn basis_range_error(alternative: bool) -> SourceError {
    let (issue, message) = range_error();
    (
        issue
            .at_field("opening")
            .in_field(if alternative { "alternative" } else { "regular" })
            .in_field("entity"),
        message,
    )
}

const SOURCE_CLASSES: [K1LossClass; 3] =
    [K1LossClass::Ordinary, K1LossClass::ShortTerm, K1LossClass::LongTerm];

#[derive(Debug, Default)]
pub(crate) struct Capital {
    pub income: Amount<Cents>,
    pub basis_allowed: Amount<Cents>,
    pub basis_suspended: Amount<Cents>,
    pub risk_allowed: Amount<Cents>,
    pub risk_suspended: Amount<Cents>,
}

#[derive(Debug)]
pub(crate) struct Component {
    pub id: i64,
    pub risk_group: i64,
    pub passive_group: i64,
    pub income: Amount<Cents>,
    pub basis_allowed: Amount<Cents>,
    pub basis_suspended: Amount<Cents>,
    pub risk_allowed: Amount<Cents>,
    pub risk_suspended: Amount<Cents>,
    pub prior_passive: Amount<Cents>,
    pub section179: i64,
    pub section179_basis_suspended: i64,
    pub section179_allowed: i64,
    pub section179_carry: i64,
    pub capital: [Capital; 2],
}

#[derive(Debug)]
pub(crate) struct PassiveGroup {
    pub id: i64,
    material: bool,
    pub current: i64,
    pub prior: i64,
    pub passive: i64,
    pub nonpassive: i64,
    pub prior_released: i64,
}

impl PassiveGroup {
    pub(crate) fn material(&self) -> bool {
        self.material
    }
}

#[derive(Debug)]
pub(crate) struct Output {
    pub basis: EntityBasis,
    pub entity_exempt_interest: i64,
    pub entity_interest: i64,
    section179_risk: Vec<(i64, i64)>,
    pub entity_capital: [i64; 2],
    pub entity_dividends: i64,
    pub entity_qualified_dividends: i64,
    pub components: Vec<Component>,
    pub groups: Vec<PassiveGroup>,
    pub qualified: Vec<QualifiedRelease>,
    pub current_qualified: Vec<CurrentQualified>,
    pub self_employment: Option<CurrentSe>,
}

impl Output {
    pub(crate) fn capital_income(&self) -> impl Iterator<Item = [i64; 2]> + '_ {
        core::iter::once(self.entity_capital).chain(
            self.components
                .iter()
                .map(|component| core::array::from_fn(|term| component.capital[term].income.get())),
        )
    }

    // Each source retains its signed character until the annual category owner.
    pub(crate) fn capital_sources(&self) -> impl Iterator<Item = [i64; 2]> + '_ {
        core::iter::once(self.entity_capital).chain(self.components.iter().map(|component| {
            core::array::from_fn(|term| {
                component.capital[term].income.get() - component.capital[term].risk_allowed.get()
            })
        }))
    }
}

// The actual block/component association survives until its global half-SE
// share is deducted, before the existing current-year QBI eligibility owner.
#[derive(Debug)]
pub(crate) struct CurrentQualified {
    pub block: i64,
    pub component: i64,
    pub qualified_se_gross: i64,
    pub activity: crate::business_rules::QbiActivity,
}

#[derive(Debug)]
pub(crate) struct CurrentSeGross {
    pub component: i64,
    pub gross: i64,
}

// Selected tax-system earnings projection. Gross is independently reported;
// neither taxable box 1 nor net SE earnings establishes it.
#[derive(Debug)]
pub(crate) struct CurrentSe {
    pub net: crate::partnership::PartnerSeResult,
    pub gross: i64,
    // PartnerSeGross::source category: farm = 0, nonfarm = 1.
    pub category: usize,
    pub components: Vec<CurrentSeGross>,
}
struct CurrentSeSource {
    reported: i64,
    passive: i64,
    gross: i64,
    category: usize,
    components: Vec<CurrentSeGross>,
}

#[derive(Debug)]
pub(crate) struct QualifiedRelease {
    pub component: i64,
    pub name: String,
    pub released: Amount<Cents>,
    pub carries: [crate::scorp_basis::LossRelease; 2],
}

#[derive(Debug)]
pub(crate) struct EntityBasis {
    pub ending: Amount<Cents>,
    pub nondeductible_used: Amount<Cents>,
    pub nondeductible_carry: Amount<Cents>,
    pub scorp: Option<crate::scorp_basis::BasisResult>,
    allowed: LossAmounts,
    section179: Amount<Cents>,
}

#[derive(Clone, Copy)]
enum Owner<'a> {
    Partnership(&'a PartnerActivityK1),
    Scorp(&'a ScorpActivityK1),
}

fn sum(values: impl IntoIterator<Item = i64>) -> Result<i64, SourceError> {
    values.into_iter().try_fold(0i64, i64::checked_add).ok_or_else(range_error)
}
fn amount(value: i64) -> Result<Amount<Cents>, SourceError> {
    Amount::new(value).map_err(|_| UNAVAILABLE)
}
fn known_zero(value: Option<i64>) -> bool {
    value == Some(0)
}
fn items_empty(source: &K1StatementItems) -> bool {
    match source {
        K1StatementItems::Partnership { income, deductions } => {
            income.is_empty() && deductions.is_empty()
        }
        K1StatementItems::Scorp { income, deductions } => {
            income.is_empty() && deductions.is_empty()
        }
    }
}
fn capital_empty(source: &K1StatementCapitalCarry) -> bool {
    match source {
        K1StatementCapitalCarry::Partnership { amounts } => [&amounts.short, &amounts.long]
            .into_iter()
            .all(|v| known_zero(v.basis) && known_zero(v.risk)),
        K1StatementCapitalCarry::Scorp { amounts } => [
            amounts.basis_short,
            amounts.basis_long,
            amounts.basis_collectibles,
            amounts.risk_short,
            amounts.risk_long,
            amounts.risk_collectibles,
        ]
        .into_iter()
        .all(known_zero),
        K1StatementCapitalCarry::Unavailable => false,
    }
}
fn component_supported(source: &K1StatementAmounts, section179: bool) -> bool {
    [source.exempt_interest, source.nondeductible, source.cash_charity, source.foreign_taxes]
        .into_iter()
        .all(|v| v == 0)
        && (section179 || source.section179 == 0)
        && source
            .portfolio
            .values()
            .into_iter()
            .enumerate()
            .all(|(index, value)| matches!(index, 4 | 5) || value == 0)
        && source.property.is_none()
        && [
            source.section179_basis_carry,
            source.section179_business_carry,
            source.section179_sold_carry,
            source.charity_basis_carry,
            source.foreign_basis_carry,
        ]
        .into_iter()
        .all(known_zero)
        && items_empty(&source.items)
}
// These are actual stage-specific source facts, never inferred from a parent
// balance or another tax system. Collectibles retain their existing boundary.
fn capital_carries(source: &K1StatementCapitalCarry) -> Result<([i64; 2], [i64; 2]), SourceError> {
    match source {
        K1StatementCapitalCarry::Partnership { amounts } => Ok((
            [amounts.short.basis.ok_or(UNAVAILABLE)?, amounts.long.basis.ok_or(UNAVAILABLE)?],
            [amounts.short.risk.ok_or(UNAVAILABLE)?, amounts.long.risk.ok_or(UNAVAILABLE)?],
        )),
        K1StatementCapitalCarry::Scorp { amounts }
            if known_zero(amounts.basis_collectibles) && known_zero(amounts.risk_collectibles) =>
        {
            Ok((
                [amounts.basis_short.ok_or(UNAVAILABLE)?, amounts.basis_long.ok_or(UNAVAILABLE)?],
                [amounts.risk_short.ok_or(UNAVAILABLE)?, amounts.risk_long.ok_or(UNAVAILABLE)?],
            ))
        }
        _ => Err(UNAVAILABLE),
    }
}
fn entity_basis_only(source: &K1EntityOnlyAmounts) -> bool {
    [source.cash_charity, source.foreign_taxes].into_iter().all(|v| v == 0)
        && source.portfolio.values().into_iter().enumerate().all(|(index, value)| {
            if matches!(index, 4 | 5) { value >= 0 } else { value == 0 || matches!(index, 0..=2) }
        })
        && [source.charity_basis_carry, source.foreign_basis_carry].into_iter().all(known_zero)
        && items_empty(&source.items)
        && capital_empty(&source.capital_carry)
}
fn personal_payments_empty(source: &PartnerPersonSource) -> bool {
    [
        source.guaranteed_services,
        source.guaranteed_capital,
        source.pensions,
        source.health_guaranteed,
        source.health_distributions,
        source.deferred_compensation_tax,
        source.crp_payments,
    ]
    .into_iter()
    .all(known_zero)
        && matches!(source.fuel_tax_credit, Some(K1CurrentFuelSource::NotReported))
        && matches!(source.prior_fuel_credit, Some(K1PriorFuelSource::NotReported))
}

fn personal_empty(source: &PartnerPersonSource) -> bool {
    personal_payments_empty(source)
        && source.reported_se == Some(0)
        && se_history_empty(&source.se_history)
        && source.limited_for_se == Some(true)
        && source.se_gross.as_ref().is_some_and(Vec::is_empty)
}
fn se_history_empty(source: &K1RetainedSeHistory) -> bool {
    [source.basis, source.risk, source.passive].into_iter().all(known_zero)
}

impl PartnerActivityK1 {
    // Availability, not a second Save validator: the actual source controls
    // must establish this fully allowed uniform-category cohort and its existing current QBI projection.
    fn current_se_source(&self) -> Result<CurrentSeSource, SourceError> {
        let person = self.person.as_ref().ok_or(UNAVAILABLE)?;
        let source = self.activities.source().ok_or(UNAVAILABLE)?;
        if !personal_payments_empty(person)
            || person.limited_for_se != Some(false)
            || !se_history_empty(&person.se_history)
            || !(zero_qualified_sources(source)
                || current_qualified_sources(source, None).is_some())
        {
            return Err(UNAVAILABLE);
        }
        let mut reported = Vec::new();
        let mut passive = Vec::new();
        let mut gross = Vec::new();
        let mut components = Vec::new();
        let mut category = None;
        for component in &source.components {
            let values = component.regular.as_ref().ok_or(UNAVAILABLE)?;
            if values.operating < 0
                || [
                    values.operating_carry.basis,
                    values.operating_carry.risk,
                    values.passive_operating,
                ]
                .into_iter()
                .any(|v| v != Some(0))
            {
                return Err(UNAVAILABLE);
            }
            let group = source
                .passive_groups
                .iter()
                .find(|g| Some(g.id) == component.passive_group)
                .ok_or(UNAVAILABLE)?;
            let K1ActivityParticipation::Partnership {
                facts,
                trade_kind: crate::ScorpTradeKind::Ordinary,
            } = &group.participation
            else {
                return Err(UNAVAILABLE);
            };
            let material = facts.material();
            if material == crate::participation::Participation::Unavailable {
                return Err(UNAVAILABLE);
            }
            match &component.self_employment {
                K1StatementSe::NotApplicable => {}
                K1StatementSe::Partnership { source } => {
                    let net = source.reported.ok_or(UNAVAILABLE)?;
                    let (actual_category, actual_gross) =
                        source.gross.as_ref().ok_or(UNAVAILABLE)?.source();
                    if category.is_some_and(|category| category != actual_category) {
                        return Err(UNAVAILABLE);
                    }
                    category = Some(actual_category);
                    if net < 0
                        || !se_history_empty(&source.history)
                        || source.guaranteed_services != 0
                        || source.guaranteed_capital != 0
                    {
                        return Err(UNAVAILABLE);
                    }
                    reported.push(net);
                    gross.push(actual_gross);
                    components
                        .push(CurrentSeGross { component: component.id, gross: actual_gross });
                    if material == crate::participation::Participation::No {
                        passive.push(net);
                    }
                }
                K1StatementSe::Unavailable => return Err(UNAVAILABLE),
            }
        }
        let reported = sum(reported)?;
        let gross = sum(gross)?;
        let category = category.ok_or(UNAVAILABLE)?;
        if (category == 0 && !zero_qualified_sources(source))
            || reported <= 0
            || person.reported_se != Some(reported)
            || !matches!(person.se_gross.as_deref(), Some([control]) if control.source() == (category, gross))
        {
            return Err(UNAVAILABLE);
        }
        dollars(reported).checked_mul(100).ok_or_else(|| {
            let (issue, message) = range_error();
            (issue.at_field("reported_se").in_field("person"), message)
        })?;
        Ok(CurrentSeSource { reported, passive: sum(passive)?, gross, category, components })
    }

    // Explicit zero SE facts do not depend on the later section 179 allowance.
    // Positive or unavailable SE still requires the final ordinary projection.
    pub(crate) fn ordinary_se_source(&self) -> Result<Output, SourceError> {
        if self.person.as_ref().is_some_and(personal_empty) {
            self.before_section179(false)
        } else {
            self.ordinary_source(false)
        }
    }

    pub(crate) fn ordinary_self_employment(&self) -> Result<Option<CurrentSe>, SourceError> {
        self.ordinary_se_source().map(|output| output.self_employment)
    }
}

// The source validator owns row validity, shared-row consistency and balance
// reconciliation. This adapter only establishes availability and a genuinely
// zero qualified remainder; empty allocations never prove a positive carry's
// original-year qualified status.
fn zero_qualified_sources(source: &K1ActivitySources) -> bool {
    let mut covered = alloc::collections::BTreeSet::new();
    for block in &source.qbi_blocks {
        if !matches!(
            block.statement,
            K1ReportedQbiStatement::Partnership { statement: None }
                | K1ReportedQbiStatement::Scorp { statement: None }
        ) || block.qbi_aggregation_group.is_some()
        {
            return false;
        }
        let Some(allocations) = &block.allocations else {
            return false;
        };
        for allocation in allocations {
            match &allocation.qualified {
                K1ComponentQualified::Partnership {
                    current_income: 0,
                    current_property: 0,
                    records,
                } if records.qualified_se_gross == 0 => {}
                K1ComponentQualified::Scorp { current_income: 0, ordinary, property }
                    if ordinary.current == 0
                        && property.gain == 0
                        && property.losses.current == 0 => {}
                _ => return false,
            }
            for (character, stage, rows) in allocation.qualified.histories() {
                if rows.iter().any(|row| row.qualified != 0 || row.remaining_qualified != 0) {
                    return false;
                }
                if character == 0 && !rows.is_empty() {
                    covered.insert((allocation.component, stage));
                }
            }
        }
    }
    source.components.iter().all(|component| {
        component.regular.as_ref().is_some_and(|values| {
            [values.operating_carry.basis, values.operating_carry.risk, values.passive_operating]
                .into_iter()
                .enumerate()
                .all(|(stage, balance)| {
                    balance
                        .is_some_and(|v| v == 0 || covered.contains(&(component.id, stage as u8)))
                })
        })
    })
}

// Explicit zero current income/loss source parts establish the new cohort's
// zero current stage input. Retained qualification can nevertheless pass the
// actual component's basis and risk worksheets. This cohort needs no allocation
// of a shared passive release or of a QBI block.
fn retained_qualified_sources(source: &K1ActivitySources) -> bool {
    let mut seen = alloc::collections::BTreeSet::new();
    for block in &source.qbi_blocks {
        if !matches!(block.statement, K1ReportedQbiStatement::Partnership { statement: None })
            || block.qbi_aggregation_group.is_some()
        {
            return false;
        }
        let Some(allocations) = &block.allocations else { return false };
        for allocation in allocations {
            if !seen.insert(allocation.component) {
                return false;
            }
            let K1ComponentQualified::Partnership {
                current_income: 0,
                current_property: 0,
                records,
            } = &allocation.qualified
            else {
                return false;
            };
            if records.qualified_se_gross != 0 {
                return false;
            }
            let Some(component) = source.components.iter().find(|v| v.id == allocation.component)
            else {
                return false;
            };
            let Some(values) = &component.regular else { return false };
            let Some(history) = &records.history else { return false };
            if values.operating != 0
                || !component.qualified_current_controls.ordinary.as_ref()
                    .is_some_and(|v| v.income == 0 && v.loss == 0)
                || values.passive_operating != Some(0) || !history.passive.is_empty()
                || !source.passive_groups.iter().any(|group| {
                    Some(group.id) == component.passive_group
                        && matches!(&group.participation,
                            K1ActivityParticipation::Partnership { facts, trade_kind: crate::ScorpTradeKind::Ordinary }
                            if facts.material() == crate::participation::Participation::Yes)
                })
            {
                return false;
            }
        }
    }
    source.components.iter().all(|component| {
        component.regular.as_ref().is_some_and(|values| {
            values.passive_operating == Some(0)
                && [values.operating_carry.basis, values.operating_carry.risk]
                    .into_iter()
                    .all(|v| v == Some(0) || seen.contains(&component.id))
        })
    })
}

// A source-bound current statement needs no allocation policy when its one
// component has pure gross operating income or loss. Losses require computed
// native allowances before emitting qualified QBI. Partnership retained
// basis/risk qualification follows the existing component allowances separately.
// The statement supplies qualified income; it never replaces taxable income.
fn current_qualified_sources(
    source: &K1ActivitySources,
    computed: Option<&[Component]>,
) -> Option<Vec<CurrentQualified>> {
    let mut seen = alloc::collections::BTreeSet::new();
    let mut statements = Vec::new();
    for block in &source.qbi_blocks {
        if block.reported_aggregate
            || block.business_names.as_ref().is_some_and(|names| names.len() > 1)
            || block.qbi_aggregation_group.is_some()
                && (block.id <= 0
                    || block.business_names.as_ref().is_none_or(|names| names.len() != 1))
        {
            return None;
        }
        let [allocation] = block.allocations.as_deref()? else { return None };
        if !seen.insert(allocation.component) {
            return None;
        }
        let qualified_se_gross = match &allocation.qualified {
            K1ComponentQualified::Partnership { records, .. } => records.qualified_se_gross,
            _ => 0,
        };
        let (current_income, statement) = match (&block.statement, &allocation.qualified) {
            (
                K1ReportedQbiStatement::Partnership { statement },
                K1ComponentQualified::Partnership { current_income, current_property: 0, records },
            ) if records.history.as_ref().is_none_or(|history| history.passive.is_empty()) => {
                let statement = match statement {
                    Some(value)
                        if value.income == *current_income
                            && value.section1231_qbi == 0
                            && value.reit_dividends == 0
                            && value.ptp_income == 0 =>
                    {
                        Some((value.income, value.wages, value.property, value.service))
                    }
                    None if *current_income == 0 && qualified_se_gross == 0 => None,
                    _ => return None,
                };
                (*current_income, statement)
            }
            (
                K1ReportedQbiStatement::Scorp { statement },
                K1ComponentQualified::Scorp { current_income, ordinary, property },
            ) if (*current_income == 0 || ordinary.current == 0)
                && property.gain == 0
                && property.qualified_1250 == Some(0)
                && property.losses.current == 0
                && allocation
                    .qualified
                    .histories()
                    .into_iter()
                    .all(|(_, _, rows)| rows.is_empty())
                && block.business_names.as_ref().is_some_and(|names| names.len() == 1) =>
            {
                let statement = match statement {
                    Some(value)
                        if value.income == current_income.checked_sub(ordinary.current)?
                            && value.section1231_qbi == 0
                            && value.reit_dividends == 0
                            && value.ptp_income == 0 =>
                    {
                        Some((value.income, value.wages, value.property, value.service))
                    }
                    None if *current_income == 0 && ordinary.current == 0 => None,
                    _ => return None,
                };
                (current_income.checked_sub(ordinary.current)?, statement)
            }
            _ => return None,
        };
        let component = source.components.iter().find(|v| v.id == allocation.component)?;
        let values = component.regular.as_ref()?;
        let controls = component.qualified_current_controls.ordinary.as_ref()?;
        if qualified_se_gross > 0
            && !matches!(&component.self_employment,
                K1StatementSe::Partnership { source }
                if matches!(&source.gross, Some(crate::PartnerSeGross::Nonfarm { .. })))
        {
            return None;
        }
        let carries = [values.operating_carry.basis, values.operating_carry.risk];
        let carries_available = match &allocation.qualified {
            K1ComponentQualified::Partnership { records, .. } => {
                carries.iter().all(Option::is_some)
                    && (carries == [Some(0); 2] || records.history.is_some())
            }
            _ => carries == [Some(0); 2],
        };
        let positive_source = values.operating > 0
            && controls.income == values.operating
            && controls.loss == 0
            && (0..=controls.income).contains(&current_income);
        let loss_source = values.operating < 0
            && controls.income == 0
            && controls.loss == values.operating.checked_neg()?
            && (values.operating..=0).contains(&current_income)
            && carries == [Some(0); 2]
            && allocation.qualified.histories().into_iter().all(|(_, _, rows)| rows.is_empty());
        if !(positive_source || loss_source)
            || !carries_available
            || values.passive_operating != Some(0)
        {
            return None;
        }
        let group =
            source.passive_groups.iter().find(|group| Some(group.id) == component.passive_group)?;
        let material = match &group.participation {
            K1ActivityParticipation::Partnership {
                facts,
                trade_kind: crate::ScorpTradeKind::Ordinary,
            } => {
                if matches!(facts, crate::PartnerParticipation::General { facts } if facts.left_out())
                {
                    return None;
                }
                facts.material()
            }
            K1ActivityParticipation::Scorp {
                facts,
                trade_kind: crate::ScorpTradeKind::Ordinary,
            } => {
                if facts.left_out() {
                    return None;
                }
                facts.material()
            }
            _ => return None,
        };
        let active = match material {
            crate::participation::Participation::Yes => true,
            crate::participation::Participation::No => false,
            crate::participation::Participation::Unavailable => return None,
        };
        if loss_source && !active {
            return None;
        }
        let released_loss = if loss_source {
            if let Some(computed) = computed {
                let output = computed.iter().find(|value| value.id == component.id)?;
                let current = Amount::<Cents>::new(controls.loss).ok()?;
                if output.basis_suspended.get() != 0
                    || output.risk_suspended.get() != 0
                    || output.basis_allowed != current
                    || output.risk_allowed != current
                {
                    return None;
                }
                // Native component outputs must reconcile exactly; qualification
                // follows the same basis and at-risk FIFO stages as retained losses.
                let basis = crate::partnership_qbi::stage(
                    &[],
                    current,
                    Amount::<Cents>::new(current_income.checked_neg()?).ok()?,
                    output.basis_allowed,
                )
                .ok()?;
                let risk = crate::partnership_qbi::stage(
                    &[],
                    basis.amount,
                    basis.qbi,
                    output.risk_allowed,
                )
                .ok()?;
                Some(risk.qbi)
            } else {
                None
            }
        } else {
            None
        };
        if let Some((income, wages, property, service)) = statement {
            statements.push(CurrentQualified {
                block: block.id,
                component: component.id,
                qualified_se_gross,
                activity: crate::business_rules::QbiActivity {
                    source_wages_cents: Some(wages),
                    source_property_cents: Some(property),
                    source_income_cents: Some(income),
                    owner: None,
                    income: released_loss
                        .map_or_else(|| dollars(income), |loss| -dollars(loss.get())),
                    wages: dollars(wages),
                    property: dollars(property),
                    active,
                    service: Some(service),
                    group: block.qbi_aggregation_group.clone(),
                },
            });
        }
    }
    (seen.len() == source.components.len()).then_some(statements)
}

fn retained_releases(
    source: &K1ActivitySources,
    components: &[Component],
) -> Result<Vec<QualifiedRelease>, SourceError> {
    let mut releases = Vec::new();
    for allocation in source.qbi_blocks.iter().flat_map(|v| v.allocations.iter().flatten()) {
        let K1ComponentQualified::Partnership { records, .. } = &allocation.qualified else {
            return Err(UNAVAILABLE);
        };
        let component =
            components.iter().find(|v| v.id == allocation.component).ok_or(UNAVAILABLE)?;
        let native = source.components.iter().find(|v| v.id == component.id).ok_or(UNAVAILABLE)?;
        let values = native.regular.as_ref().ok_or(UNAVAILABLE)?;
        // A current-only allocation needs no historical worksheet or release
        // note when its actual retained balances are explicitly zero.
        if values.operating > 0
            && values.operating_carry.basis == Some(0)
            && values.operating_carry.risk == Some(0)
            && values.passive_operating == Some(0)
            && records.history.as_ref().is_none_or(|history| {
                history.basis.is_empty() && history.risk.is_empty() && history.passive.is_empty()
            })
        {
            continue;
        }
        let history = records.history.as_ref().ok_or(UNAVAILABLE)?;
        let basis = crate::partnership_qbi::stage(
            &history.basis,
            Amount::default(),
            Amount::default(),
            component.basis_allowed,
        )
        .map_err(source_error)?;
        let risk = crate::partnership_qbi::stage(
            &history.risk,
            basis.amount,
            basis.qbi,
            component.risk_allowed,
        )
        .map_err(source_error)?;
        // These are native owner worksheet outputs, not reconstructed annual
        // display totals. A cent/history rounding mismatch remains unavailable.
        if sum(basis.remaining.iter().map(|v| v.remaining.get()))?
            != component.basis_suspended.get()
            || sum(risk.remaining.iter().map(|v| v.remaining.get()))?
                != component.risk_suspended.get()
        {
            return Err(UNAVAILABLE);
        }
        releases.push(QualifiedRelease {
            component: component.id,
            name: native.name.clone(),
            released: risk.qbi,
            carries: [basis, risk],
        });
    }
    Ok(releases)
}

impl PartnerActivityK1 {
    pub(crate) fn before_section179(&self, alternative: bool) -> Result<Output, SourceError> {
        Owner::Partnership(self).ordinary_source(alternative, true)
    }
    pub(crate) fn ordinary_source(&self, alternative: bool) -> Result<Output, SourceError> {
        Owner::Partnership(self).ordinary_source(alternative, false)
    }
}
impl ScorpActivityK1 {
    pub(crate) fn before_section179(&self, alternative: bool) -> Result<Output, SourceError> {
        Owner::Scorp(self).ordinary_source(alternative, true)
    }
    pub(crate) fn ordinary_source(&self, alternative: bool) -> Result<Output, SourceError> {
        Owner::Scorp(self).ordinary_source(alternative, false)
    }
}
impl Owner<'_> {
    // Availability is checked before calculations. An unsupported mandatory
    // source never becomes zero or a partly calculated complete estimate.
    fn ordinary_source(&self, alternative: bool, section179: bool) -> Result<Output, SourceError> {
        let current_se = match self {
            Self::Partnership(owner) if !owner.person.as_ref().is_some_and(personal_empty) => {
                Some(owner.current_se_source()?)
            }
            _ => None,
        };
        let source = match self {
            Self::Partnership(owner) => {
                if owner.issue().is_some()
                    || owner.entity.publicly_traded
                    || owner.entity.final_k1
                    || !matches!(owner.entity.disposition, Some(crate::K1Disposition::Kept))
                    || owner.entity.sale != crate::PartnerSaleParts::default()
                    || owner
                        .entity
                        .amt_section179_income_carry
                        .as_ref()
                        .is_some_and(|v| v.gross != 0)
                    || !owner.entity.section179_sales.is_empty()
                    || owner.entity.schedule_k3.is_some()
                    || !owner.entity.credits.is_empty()
                    || !owner.unentered.is_empty()
                    || !owner.special.is_empty()
                    || !matches!(owner.health_insurance, Some(crate::BusinessHealthInsurance::None))
                    || !matches!(owner.retirement, Some(crate::PartnerRetirement::None))
                    || !(owner.person.as_ref().is_some_and(personal_empty) || current_se.is_some())
                {
                    return Err(UNAVAILABLE);
                }
                owner.activities.source().ok_or(UNAVAILABLE)?
            }
            Self::Scorp(owner) => {
                if owner.issue().is_some()
                    || owner.entity.final_k1
                    || !matches!(owner.entity.disposition, Some(crate::K1Disposition::Kept))
                    || owner
                        .entity
                        .amt_section179_income_carry
                        .as_ref()
                        .is_some_and(|v| v.gross != 0)
                    || !owner.entity.section179_sales.is_empty()
                    || owner.entity.schedule_k3.is_some()
                    || !owner.entity.credits.is_empty()
                    || !owner.unentered.is_empty()
                    || !owner.special.is_empty()
                    || !matches!(owner.health_insurance, Some(crate::BusinessHealthInsurance::None))
                    || !owner.person.as_ref().is_some_and(|v| {
                        known_zero(v.withholding) && known_zero(v.private_activity)
                    })
                {
                    return Err(UNAVAILABLE);
                }
                owner.activities.source().ok_or(UNAVAILABLE)?
            }
        };
        let zero_qualified = zero_qualified_sources(source);
        let current_qualified =
            if zero_qualified { None } else { current_qualified_sources(source, None) };
        if source.identity_issue(Some(source)).is_some()
            || source.components.is_empty()
            || !(zero_qualified
                || matches!(self, Self::Partnership(_)) && retained_qualified_sources(source)
                || current_qualified.is_some())
        {
            return Err(UNAVAILABLE);
        }
        let (entity_only, current_available) = match self {
            Self::Partnership(owner) => (
                entity_only_source(source, alternative),
                !alternative || owner.alternative_current.is_some(),
            ),
            Self::Scorp(owner) => (
                entity_only_source(source, alternative),
                !alternative || owner.alternative_current.is_some(),
            ),
        };
        let entity_only = entity_only.ok_or(UNAVAILABLE)?;
        let entity_interest = entity_only.portfolio.interest;
        if !current_available || !entity_basis_only(entity_only) {
            return Err(UNAVAILABLE);
        }
        // Outside-all-activities interest increases whole-interest basis, but
        // never an activity's at-risk income, passive income or qualified income.
        // Each tax system uses its own documented source (IRS i1065sk1,
        // i1120ssk, i6198 and i8995).
        let entity_dividends = entity_only.portfolio.dividends;
        let entity_qualified_dividends = entity_only.portfolio.qualified_dividends;
        let properties = self.properties();
        let confirmed = match self {
            Self::Partnership(owner) => owner.entity.investment_confirmed,
            Self::Scorp(owner) => owner.entity.investment_confirmed,
        };
        if properties.iter().any(|p| match p.own_form() {
            Some(own) => !own.reduced,
            None => confirmed != Some(true),
        }) {
            return Err(UNAVAILABLE);
        }
        let investment = investment_sources(source, properties)?;
        let credit_reduction = sum(investment.iter().map(|(_, decrease)| *decrease))?;
        let mut components = Vec::new();
        let mut losses = Vec::new();
        let mut inputs = Vec::new();
        let mut income_cents = Vec::new();
        let mut exempt_cents = Vec::new();
        let mut passive_cents = Vec::new();
        let mut section179_cents = Vec::new();
        for (component_index, component) in source.components.iter().enumerate() {
            let locate = |(issue, message): SourceError| {
                (
                    issue
                        .at_field(if alternative { "alternative" } else { "regular" })
                        .in_list("components", component_index)
                        .in_field("source")
                        .in_field("statement")
                        .in_field("activities"),
                    message,
                )
            };
            if component.id <= 0
                || component.kind != K1StatementKind::TradeBusiness
                || (current_se.is_none()
                    && !matches!(component.self_employment, K1StatementSe::NotApplicable))
                || !matches!(component.personal_services, K1ActivityPersonalSource::NotApplicable)
            {
                return Err(UNAVAILABLE);
            }
            let regular = component.regular.as_ref().ok_or(UNAVAILABLE)?;
            let values = if alternative {
                match &component.alternative {
                    K1StatementAmt::SameAsRegular => regular,
                    K1StatementAmt::Separate { amounts } => amounts,
                    K1StatementAmt::Unavailable => return Err(UNAVAILABLE),
                }
            } else {
                regular
            };
            // The same-record choice supplies the baseline before printed
            // depletion adjustments. Independent AMT records already include
            // them. Project before the common basis, risk and passive owners.
            let depletion =
                if alternative && matches!(component.alternative, K1StatementAmt::SameAsRegular) {
                    match self {
                        Self::Partnership(owner) => owner.depletion_adjustment(component.id),
                        Self::Scorp(owner) => owner.depletion_adjustment(component.id),
                    }
                    .ok_or(UNAVAILABLE)?
                } else {
                    0
                };
            let operating = values.operating.checked_add(depletion).ok_or_else(range_error)?;
            if !component_supported(values, section179)
                || (current_se.is_some()
                    && (operating < 0
                        || [
                            values.operating_carry.basis,
                            values.operating_carry.risk,
                            values.passive_operating,
                        ]
                        .into_iter()
                        .any(|v| v != Some(0))))
            {
                return Err(UNAVAILABLE);
            }
            if values.section179 != 0 && (!zero_qualified_sources(source) || !properties.is_empty())
            {
                return Err(UNAVAILABLE);
            }
            section179_cents.push(values.section179);
            let basis_carry = values.operating_carry.basis.ok_or(UNAVAILABLE)?;
            let risk_carry = values.operating_carry.risk.ok_or(UNAVAILABLE)?;
            let prior = values.passive_operating.ok_or(UNAVAILABLE)?;
            let (capital_basis, capital_risk) = capital_carries(&values.capital_carry)?;
            let signed = [operating, values.portfolio.short, values.portfolio.long];
            let basis = [basis_carry, capital_basis[0], capital_basis[1]];
            let risk = [risk_carry, capital_risk[0], capital_risk[1]];
            let mut input = ClassInput::default();
            for (index, class) in SOURCE_CLASSES.into_iter().enumerate() {
                input.current[class] = amount(if signed[index] < 0 {
                    signed[index].checked_neg().ok_or_else(range_error).map_err(locate)?
                } else {
                    0
                })?;
                input.basis_carry[class] = amount(basis[index])?;
                input.risk_carry[class] = amount(risk[index])?;
                sum([input.current[class].get(), basis[index]]).map_err(locate)?;
            }
            losses.push(input.basis());
            inputs.push(input);
            income_cents.push(signed.map(positive));
            exempt_cents.push(values.exempt_other);
            passive_cents.push(prior);
            components.push(Component {
                id: component.id,
                risk_group: component.risk_group.ok_or(UNAVAILABLE)?,
                passive_group: component.passive_group.ok_or(UNAVAILABLE)?,
                income: Amount::default(),
                basis_allowed: Amount::default(),
                basis_suspended: Amount::default(),
                risk_allowed: Amount::default(),
                risk_suspended: Amount::default(),
                prior_passive: Amount::default(),
                section179: 0,
                section179_basis_suspended: 0,
                section179_allowed: 0,
                section179_carry: 0,
                capital: core::array::from_fn(|_| Capital::default()),
            });
        }
        // The native ClassInput owners keep distinct current/prior annual
        // columns. Aggregate their actual source cents independently first.
        let mut input = ClassInput::default();
        for class in SOURCE_CLASSES {
            input.current[class] = amount(sum(inputs.iter().map(|v| v.current[class].get()))?)?;
            input.basis_carry[class] =
                amount(sum(inputs.iter().map(|v| v.basis_carry[class].get()))?)?;
            sum([input.current[class].get(), input.basis_carry[class].get()])?;
        }
        input.current[K1LossClass::Section179] = amount(sum(section179_cents.iter().copied())?)?;
        let loss_classes = input.basis();
        sum(loss_classes.as_slice().iter().map(Amount::get))?;
        let basis = self.basis(
            alternative,
            &loss_classes,
            // All actual exemptions enter whole-interest basis once. Only the
            // component-attributed parts subsequently enter their risk group.
            amount(
                sum(income_cents
                    .iter()
                    .flatten()
                    .copied()
                    .chain(exempt_cents.iter().copied())
                    .chain([
                        entity_only.exempt_interest,
                        entity_only.exempt_other,
                        entity_interest,
                        entity_only.portfolio.short,
                        entity_only.portfolio.long,
                        entity_dividends,
                    ]))
                .map_err(|(issue, message)| {
                    (issue.at_field("source").in_field("statement").in_field("activities"), message)
                })?,
            )?,
            amount(credit_reduction)?,
            amount(entity_only.nondeductible)?,
        )?;
        for (index, class) in SOURCE_CLASSES.into_iter().enumerate() {
            let weights: Vec<_> = losses.iter().map(|v| v[class].get()).collect();
            // The existing allocator's weight receiver is native i64.
            sum(weights.iter().copied())?;
            for ((component, loss), allowed) in components
                .iter_mut()
                .zip(&weights)
                .zip(allocate_remaining(basis.allowed[class].get(), &weights))
            {
                if index == 0 {
                    component.basis_allowed = amount(allowed)?;
                    component.basis_suspended = amount(loss - allowed)?;
                } else {
                    let capital = &mut component.capital[index - 1];
                    capital.basis_allowed = amount(allowed)?;
                    capital.basis_suspended = amount(loss - allowed)?;
                }
            }
        }
        for ((component, source), allowed) in components
            .iter_mut()
            .zip(&section179_cents)
            .zip(allocate_remaining(basis.section179.get(), &section179_cents))
        {
            component.section179 = allowed;
            component.section179_basis_suspended = source - allowed;
        }
        let mut section179_risk = Vec::new();
        let mut contribution_parts = Vec::new();
        let mut distribution_parts = Vec::new();
        let mut loan_advances = Vec::new();
        let mut loan_repayments = Vec::new();
        for (group_index, group) in source.risk_groups.iter().enumerate() {
            let parts = group.investment_parts.as_ref().ok_or(UNAVAILABLE)?;
            let indices: Vec<_> = components
                .iter()
                .enumerate()
                .filter(|(_, c)| c.risk_group == group.id)
                .map(|(i, _)| i)
                .collect();
            if indices.is_empty() {
                return Err(UNAVAILABLE);
            }
            let (investment, _prior_loss, deducted, recaptured) = self
                .investment(
                    alternative,
                    group,
                    sum(indices.iter().map(|&i| exempt_cents[i])).map_err(|(issue, message)| {
                        (
                            issue
                                .at_field("components")
                                .in_field("source")
                                .in_field("statement")
                                .in_field("activities"),
                            message,
                        )
                    })?,
                    sum(indices.iter().flat_map(|&i| {
                        SOURCE_CLASSES.map(|class| inputs[i].risk_carry[class].get())
                    }))?,
                    indices.iter().any(|&i| inputs[i].carries() || passive_cents[i] != 0),
                )
                .map_err(|(issue, message)| {
                    if issue.key != RANGE_KEY || issue.field.is_some() {
                        return (issue, message);
                    }
                    // This capacity combines several actual component sources. Locate
                    // the selected risk record, never blame its first component.
                    (
                        issue
                            .at_field(if alternative { "alternative" } else { "regular" })
                            .in_field("source")
                            .in_list("risk_groups", group_index)
                            .in_field("source")
                            .in_field("statement")
                            .in_field("activities"),
                        message,
                    )
                })?;
            contribution_parts.push(parts.contributions);
            distribution_parts.push(parts.distributions);
            loan_advances.push(parts.loan_advances);
            loan_repayments.push(parts.loan_repayments);
            let income = sum(indices.iter().flat_map(|&i| income_cents[i]))?;
            for &index in &indices {
                components[index].income = amount(income_cents[index][0])?;
                for (capital, value) in
                    components[index].capital.iter_mut().zip(&income_cents[index][1..])
                {
                    capital.income = amount(*value)?;
                }
            }
            // One settlement shares this actual group's income and capacity
            // across every component and admitted character. Class order is
            // the existing catalog order, then durable component source order.
            let mut entries = Vec::new();
            for (class_index, class) in SOURCE_CLASSES.into_iter().enumerate() {
                for &index in &indices {
                    entries.push(RiskEntry {
                        key: (components[index].id, class),
                        basis_allowed: if class_index == 0 {
                            components[index].basis_allowed
                        } else {
                            components[index].capital[class_index - 1].basis_allowed
                        },
                        prior_risk: inputs[index].risk_carry[class],
                    });
                }
            }
            let SourceFact::Reported(settlement) =
                risk::settle_at_risk(SourceFact::Reported(RiskInput {
                    entries: &entries,
                    bypass: &[],
                    income: amount(income)?,
                    capacity: positive(investment),
                }))
                .map_err(|_| UNAVAILABLE)?
            else {
                return Err(UNAVAILABLE);
            };
            let account = settlement
                .close_account(
                    investment,
                    SourceFact::Reported(RecaptureHistory {
                        prior_deducted: amount(deducted)?,
                        prior_recaptured: amount(recaptured)?,
                        threshold_1978: 0,
                    }),
                )
                .map_err(|_| UNAVAILABLE)?;
            if !matches!(account.recapture, SourceFact::Reported(ref r) if r.income.get() == 0) {
                return Err(UNAVAILABLE);
            }
            section179_risk.push((group.id, positive(account.ending)));
            for class in &settlement.classes {
                let &(id, kind) = class.key;
                let component = components.iter_mut().find(|c| c.id == id).ok_or(UNAVAILABLE)?;
                if kind == K1LossClass::Ordinary {
                    component.risk_allowed = class.allowed;
                    component.risk_suspended = class.suspended;
                } else {
                    let capital =
                        &mut component.capital[usize::from(kind == K1LossClass::LongTerm)];
                    capital.risk_allowed = class.allowed;
                    capital.risk_suspended = class.suspended;
                }
            }
        }
        self.reconcile_investment(
            alternative,
            sum(contribution_parts)?,
            sum(distribution_parts)?,
            sum(loan_advances)?,
            sum(loan_repayments)?,
        )?;
        if components.iter().any(|c| !source.risk_groups.iter().any(|g| g.id == c.risk_group)) {
            return Err(UNAVAILABLE);
        }
        let mut groups = Vec::new();
        for group in &source.passive_groups {
            if group.disposed {
                return Err(UNAVAILABLE);
            }
            let material = match (&self, &group.participation) {
                (
                    Self::Partnership(_),
                    K1ActivityParticipation::Partnership {
                        facts,
                        trade_kind: crate::ScorpTradeKind::Ordinary,
                    },
                ) => {
                    if matches!(facts, crate::PartnerParticipation::General { facts } if facts.left_out())
                    {
                        return Err(UNAVAILABLE);
                    }
                    facts.material()
                }
                (
                    Self::Scorp(_),
                    K1ActivityParticipation::Scorp {
                        facts,
                        trade_kind: crate::ScorpTradeKind::Ordinary,
                    },
                ) => {
                    if facts.left_out() {
                        return Err(UNAVAILABLE);
                    }
                    facts.material()
                }
                _ => return Err(UNAVAILABLE),
            };
            let material = match material {
                crate::participation::Participation::Yes => true,
                crate::participation::Participation::No => false,
                crate::participation::Participation::Unavailable => return Err(UNAVAILABLE),
            };
            if !material
                && investment.iter().any(|(component, _)| {
                    source
                        .components
                        .iter()
                        .any(|c| c.id == *component && c.passive_group == Some(group.id))
                })
            {
                return Err(UNAVAILABLE);
            }
            let indices: Vec<_> = components
                .iter()
                .enumerate()
                .filter(|(_, c)| c.passive_group == group.id)
                .map(|(i, _)| i)
                .collect();
            if indices.is_empty() {
                return Err(UNAVAILABLE);
            }
            if !material && indices.iter().any(|&i| section179_cents[i] != 0) {
                return Err(UNAVAILABLE);
            }
            let current = sum(indices
                .iter()
                .map(|&i| components[i].income.get() - components[i].risk_allowed.get()))?;
            let weights: Vec<_> = indices.iter().map(|&i| passive_cents[i]).collect();
            let prior = sum(weights.iter().copied())?;
            for (&index, share) in indices.iter().zip(allocate_remaining(prior, &weights)) {
                components[index].prior_passive = amount(share)?;
            }
            current.checked_sub(prior).ok_or_else(range_error)?;
            let pooled = crate::partnership::PooledNets::of(current, prior, material, false);
            groups.push(PassiveGroup {
                id: group.id,
                material,
                current,
                prior: pooled.prior_pool,
                passive: pooled.passive,
                nonpassive: pooled.nonpassive,
                prior_released: pooled.prior_released,
            });
        }
        if components.iter().any(|c| !groups.iter().any(|g| g.id == c.passive_group)) {
            return Err(UNAVAILABLE);
        }
        let current_retained = matches!(self, Self::Partnership(_))
            && current_qualified.is_some()
            && source.components.iter().any(|component| {
                component.regular.as_ref().is_some_and(|values| {
                    values.operating_carry.basis.is_some_and(|v| v > 0)
                        || values.operating_carry.risk.is_some_and(|v| v > 0)
                })
            });
        // A nonnegative passive group has absorbed all its losses locally,
        // including when several components share it. A different group's
        // profit cannot establish a qualified share of a negative group's PAL.
        // Use the already computed owner net before exposing either QBI output.
        if !alternative
            && !zero_qualified_sources(source)
            && current_retained
            && source.passive_groups.iter().any(|source_group| {
                matches!(&source_group.participation,
                K1ActivityParticipation::Partnership { facts, .. }
                if facts.material() == crate::participation::Participation::No)
                    && groups
                        .iter()
                        .find(|group| group.id == source_group.id)
                        .is_none_or(|group| group.passive < 0)
            })
        {
            return Err(UNAVAILABLE);
        }
        let qualified = if !alternative
            && !zero_qualified_sources(source)
            && (retained_qualified_sources(source) || current_retained)
        {
            retained_releases(source, &components)?
        } else {
            Vec::new()
        };
        let current_qualified = if !alternative && current_qualified.is_some() {
            current_qualified_sources(source, Some(&components)).ok_or(UNAVAILABLE)?
        } else {
            Vec::new()
        };
        let self_employment = if let Some(mut current) = current_se {
            if alternative {
                let mut reported = Vec::new();
                let mut passive = Vec::new();
                for (source_component, component) in source.components.iter().zip(&components) {
                    let K1StatementSe::Partnership { source: se } =
                        &source_component.self_employment
                    else {
                        continue;
                    };
                    let regular = source_component.regular.as_ref().ok_or(UNAVAILABLE)?;
                    let net = se.reported.ok_or(UNAVAILABLE)?;
                    let delta = component
                        .income
                        .get()
                        .checked_sub(regular.operating)
                        .ok_or_else(range_error)?;
                    // The statement identifies SE character independently of
                    // taxable income. A mixed current source needs its own AMT
                    // character facts; a taxable-income change cannot supply them.
                    if delta != 0 && net != regular.operating {
                        return Err(UNAVAILABLE);
                    }
                    let adjusted = net.checked_add(delta).ok_or_else(range_error)?;
                    if adjusted < 0 {
                        return Err(UNAVAILABLE);
                    }
                    reported.push(adjusted);
                    let group = groups
                        .iter()
                        .find(|group| group.id == component.passive_group)
                        .ok_or(UNAVAILABLE)?;
                    if !group.material {
                        passive.push(adjusted);
                    }
                }
                current.reported = sum(reported)?;
                current.passive = sum(passive)?;
            }
            // These are the actual stage outputs, not a reconstructed ledger.
            if components.iter().any(|c| {
                c.basis_allowed.get() != 0
                    || c.basis_suspended.get() != 0
                    || c.risk_allowed.get() != 0
                    || c.risk_suspended.get() != 0
                    || c.prior_passive.get() != 0
            }) || groups.iter().any(|g| g.prior != 0 || g.prior_released != 0)
            {
                return Err(UNAVAILABLE);
            }
            Some(CurrentSe {
                gross: current.gross,
                category: current.category,
                components: current.components,
                net: crate::partnership::PartnerSeResult::from_resolved(
                    crate::partnership::ResolvedPartnerSe {
                        reported: current.reported,
                        guaranteed_services: 0,
                        guaranteed_capital: 0,
                        limited_for_se: false,
                        section179: 0,
                        allowed_se: 0,
                        unmatched_loss: 0,
                        passive_earnings: current.passive,
                        basis_suspended: 0,
                        risk_suspended: 0,
                        passive_suspended: 0,
                    },
                ),
            })
        } else {
            None
        };
        Ok(Output {
            basis,
            entity_exempt_interest: entity_only.exempt_interest,
            entity_interest,
            section179_risk,
            entity_capital: [entity_only.portfolio.short, entity_only.portfolio.long],
            entity_dividends,
            entity_qualified_dividends,
            components,
            groups,
            qualified,
            current_qualified,
            self_employment,
        })
    }
}

fn entity_only_source(
    source: &K1ActivitySources,
    alternative: bool,
) -> Option<&K1EntityOnlyAmounts> {
    if alternative {
        match &source.alternative_entity_only {
            K1EntityOnlyAmt::SameAsRegular => source.entity_only.as_ref(),
            K1EntityOnlyAmt::Separate { amounts } => Some(amounts),
            K1EntityOnlyAmt::Unavailable => None,
        }
    } else {
        source.entity_only.as_ref()
    }
}

impl<'a> Owner<'a> {
    fn properties(self) -> &'a [crate::K1InvestmentProperty] {
        match self {
            Self::Partnership(owner) => &owner.entity.investment_property,
            Self::Scorp(owner) => &owner.entity.investment_property,
        }
    }
    fn basis(
        self,
        alternative: bool,
        losses: &crate::k1_loss::LossAmounts,
        income: Amount<Cents>,
        credit_reduction: Amount<Cents>,
        nondeductible: Amount<Cents>,
    ) -> Result<EntityBasis, SourceError> {
        match self {
            Self::Partnership(owner) => {
                let entity = if alternative {
                    owner.entity.alternative.as_ref().ok_or(UNAVAILABLE)?
                } else {
                    &owner.entity.regular
                };
                if !entity.complete
                    || entity.prior_nondeductible != Some(0)
                    || entity.assumed != 0
                    || entity.transferred != 0
                {
                    return Err(UNAVAILABLE);
                }
                let basis = crate::partnership_basis::PartnerBasis {
                    opening: Amount::<Cents>::new(entity.opening.ok_or(UNAVAILABLE)?)
                        .expect("bounded basis result"),
                    contributions: Amount::<Cents>::new(entity.contributions)
                        .expect("bounded basis result"),
                    opening_liabilities: Amount::<Cents>::new(entity.opening_liabilities)
                        .expect("bounded basis result"),
                    ending_liabilities: Amount::<Cents>::new(entity.ending_liabilities)
                        .expect("bounded basis result"),
                    partnership_liabilities_assumed: Amount::<Cents>::new(0)
                        .expect("bounded basis result"),
                    personal_liabilities_transferred: Amount::<Cents>::new(0)
                        .expect("bounded basis result"),
                    income,
                    cash_distributions: Amount::<Cents>::new(entity.distributions)
                        .expect("bounded basis result"),
                    distributed_property_basis: Amount::<Cents>::new(0)
                        .expect("bounded basis result"),
                    nondeductible,
                    credit_reduction,
                    depletion: Amount::<Cents>::new(0).expect("bounded basis result"),
                    losses,
                    business_interest: Amount::<Cents>::new(0).expect("bounded basis result"),
                    excess_business_interest: Amount::<Cents>::new(0)
                        .expect("bounded basis result"),
                    deemed_money: Amount::<Cents>::new(0).expect("bounded basis result"),
                }
                .calculate()
                .ok_or_else(|| basis_range_error(alternative))?;
                if basis.distribution_gain.get() != 0 {
                    return Err(UNAVAILABLE);
                }
                Ok(EntityBasis {
                    ending: basis.ending,
                    nondeductible_used: basis.nondeductible_used,
                    nondeductible_carry: basis.nondeductible_carry,
                    allowed: basis.allowed,
                    section179: basis.allowed[K1LossClass::Section179],
                    scorp: None,
                })
            }
            Self::Scorp(owner) => {
                let entity = if alternative {
                    owner.entity.alternative.as_ref().ok_or(UNAVAILABLE)?
                } else {
                    &owner.entity.regular
                };
                if !entity.complete || entity.prior_nondeductible != Some(0) {
                    return Err(UNAVAILABLE);
                }
                let loans = owner
                    .entity
                    .loans
                    .iter()
                    .map(|loan| {
                        let value = &loan.regular;
                        Ok(crate::scorp_basis::ShareholderLoan {
                            kind: match value.kind {
                                crate::ScorpLoanKind::FormalNote => {
                                    crate::scorp_basis::LoanKind::FormalNote
                                }
                                crate::ScorpLoanKind::OpenAccount => {
                                    crate::scorp_basis::LoanKind::OpenAccount
                                }
                                crate::ScorpLoanKind::FormerOpenAccount => {
                                    crate::scorp_basis::LoanKind::FormerOpenAccount
                                }
                            },
                            principal: Amount::<Cents>::new(value.principal)
                                .expect("bounded basis result"),
                            basis: Amount::<Cents>::new(if alternative {
                                loan.alternative_basis.ok_or(UNAVAILABLE)?
                            } else {
                                value.basis
                            })
                            .expect("bounded basis result"),
                            advances: Amount::<Cents>::new(value.advances)
                                .expect("bounded basis result"),
                            repayments: Amount::<Cents>::new(value.repayments)
                                .expect("bounded basis result"),
                        })
                    })
                    .collect::<Result<Vec<_>, SourceError>>()?;
                let basis = crate::scorp_basis::StockBasis {
                    opening: Amount::<Cents>::new(entity.opening.ok_or(UNAVAILABLE)?)
                        .expect("bounded basis result"),
                    additions: Amount::<Cents>::new(entity.additions)
                        .expect("bounded basis result"),
                    income,
                    distributions: Amount::<Cents>::new(entity.distributions)
                        .expect("bounded basis result"),
                    nondeductible,
                    credit_reduction,
                    prior_nondeductible: Amount::<Cents>::new(0).expect("bounded basis result"),
                    losses_first: entity.losses_first,
                    losses,
                }
                .with_loans(&loans)
                .map_err(|error| {
                    if error == crate::scorp_basis::BasisError::Overflow {
                        basis_range_error(alternative)
                    } else {
                        UNAVAILABLE
                    }
                })?;
                // No source associates a repayment gain with a risk/passive
                // group. Only each actual loan owner's zero gain needs no split.
                if basis.distribution_gain.get() != 0
                    || basis
                        .loans
                        .iter()
                        .any(|loan| loan.ordinary_gain.get() != 0 || loan.capital_gain.get() != 0)
                {
                    return Err(UNAVAILABLE);
                }
                Ok(EntityBasis {
                    ending: basis.stock,
                    nondeductible_used: basis.nondeductible_used,
                    nondeductible_carry: basis.nondeductible_carry,
                    allowed: basis.allowed,
                    section179: basis.allowed[K1LossClass::Section179],
                    scorp: Some(basis),
                })
            }
        }
    }

    fn investment(
        self,
        alternative: bool,
        group: &K1RiskGroup,
        exempt: i64,
        prior: i64,
        has_prior: bool,
    ) -> Result<(i64, i64, i64, i64), SourceError> {
        let parts = group.investment_parts.as_ref().ok_or(UNAVAILABLE)?;
        match (self, &group.source) {
            (Self::Partnership(owner), K1RiskSource::Partnership { regular, alternative: amt }) => {
                if parts.loan_advances != 0 || parts.loan_repayments != 0 {
                    return Err(UNAVAILABLE);
                }
                let entity = if alternative {
                    owner.entity.alternative.as_ref().ok_or(UNAVAILABLE)?
                } else {
                    &owner.entity.regular
                };
                let risk = if alternative { amt.as_ref().ok_or(UNAVAILABLE)? } else { regular };
                if risk.prior_loss != prior || risk.prior_nondeductible != 0 {
                    return Err(UNAVAILABLE);
                }
                // Group debt is a subset of the whole partnership liability source.
                if risk.opening_debt > entity.opening_liabilities
                    || risk.ending_debt > entity.ending_liabilities
                {
                    return Err(UNAVAILABLE);
                }
                let investment = risk
                    .investment(
                        crate::partnership::PartnerRiskFinancing {
                            contributions: parts.contributions,
                            distributions: parts.distributions,
                            opening_liabilities: entity.opening_liabilities,
                            ending_liabilities: entity.ending_liabilities,
                            assumed: 0,
                            transferred: 0,
                            has_prior: has_prior || risk.opening_debt != 0,
                        },
                        exempt,
                        0,
                    )
                    .map_err(source_error)?
                    .ok_or(UNAVAILABLE)?;
                Ok((investment, risk.prior_loss, risk.deducted, risk.recaptured))
            }
            (Self::Scorp(_), K1RiskSource::Scorp { regular, alternative: amt }) => {
                let risk = if alternative { amt.as_ref().ok_or(UNAVAILABLE)? } else { regular };
                if risk.prior_loss != prior || risk.prior_nondeductible != 0 {
                    return Err(UNAVAILABLE);
                }
                let investment = risk
                    .component_investment(
                        parts,
                        exempt,
                        has_prior
                            || match self {
                                Self::Scorp(owner) => {
                                    let entity = if alternative {
                                        owner.entity.alternative.as_ref().ok_or(UNAVAILABLE)?
                                    } else {
                                        &owner.entity.regular
                                    };
                                    entity.opening != Some(0)
                                        || owner
                                            .entity
                                            .loans
                                            .iter()
                                            .any(|v| v.regular.principal != 0)
                                }
                                _ => false,
                            },
                    )
                    .map_err(source_error)?;
                Ok((investment, risk.prior_loss, risk.deducted, risk.recaptured))
            }
            _ => Err(UNAVAILABLE),
        }
    }
    fn reconcile_investment(
        self,
        alternative: bool,
        contributions: i64,
        distributions: i64,
        advances: i64,
        repayments: i64,
    ) -> Result<(), SourceError> {
        let matches = match self {
            Self::Partnership(owner) => {
                let entity = if alternative {
                    owner.entity.alternative.as_ref().ok_or(UNAVAILABLE)?
                } else {
                    &owner.entity.regular
                };
                let source = owner.activities.source().ok_or(UNAVAILABLE)?;
                let mut opening = Vec::new();
                let mut ending = Vec::new();
                for group in &source.risk_groups {
                    let K1RiskSource::Partnership { regular, alternative: amt } = &group.source
                    else {
                        return Err(UNAVAILABLE);
                    };
                    let risk = if alternative { amt.as_ref().ok_or(UNAVAILABLE)? } else { regular };
                    opening.push(risk.opening_debt);
                    ending.push(risk.ending_debt);
                }
                contributions == entity.contributions
                    && distributions == entity.distributions
                    && sum(opening)? <= entity.opening_liabilities
                    && sum(ending)? <= entity.ending_liabilities
            }
            Self::Scorp(owner) => {
                let entity = if alternative {
                    owner.entity.alternative.as_ref().ok_or(UNAVAILABLE)?
                } else {
                    &owner.entity.regular
                };
                contributions == entity.additions
                    && distributions == entity.distributions
                    && advances == sum(owner.entity.loans.iter().map(|v| v.regular.advances))?
                    && repayments == sum(owner.entity.loans.iter().map(|v| v.regular.repayments))?
            }
        };
        if matches { Ok(()) } else { Err(UNAVAILABLE) }
    }
}

// Resolve only explicit same-parent bindings. Source validation owns accepted
// identities; calculation also rejects incomplete or stale working-source links.
fn investment_sources(
    source: &K1ActivitySources,
    properties: &[crate::K1InvestmentProperty],
) -> Result<Vec<(i64, i64)>, SourceError> {
    if properties.is_empty() {
        return Ok(Vec::new());
    }
    let rows = source.investment_property_sources.as_ref().ok_or(UNAVAILABLE)?;
    if rows.len() != properties.len() {
        return Err(UNAVAILABLE);
    }
    let native =
        crate::investment_credit::property_credit_sources(properties).ok_or(UNAVAILABLE)?;
    let mut seen = alloc::collections::BTreeSet::new();
    native
        .into_iter()
        .zip(properties)
        .map(|(value, property)| {
            let row = rows.iter().find(|r| r.property == value.property_id).ok_or(UNAVAILABLE)?;
            if !seen.insert(row.property)
                || property.rental_real_estate()
                || !source
                    .components
                    .iter()
                    .any(|c| c.id == row.component && c.kind == K1StatementKind::TradeBusiness)
            {
                return Err(UNAVAILABLE);
            }
            Ok((row.component, value.basis_decrease_cents))
        })
        .collect()
}

impl Output {
    pub(crate) fn apply_section179(
        mut self,
        selected: Option<&crate::Section179PreparedAllocation>,
    ) -> Result<Self, SourceError> {
        let Some(selected) = selected else {
            return if self.components.iter().any(|component| {
                component.section179 != 0 || component.section179_basis_suspended != 0
            }) {
                Err(UNAVAILABLE)
            } else {
                Ok(self)
            };
        };
        for component in &mut self.components {
            let allowed = selected.native_allowed(component.id);
            if allowed < 0 || allowed > component.section179 {
                return Err(UNAVAILABLE);
            }
            component.section179_allowed = allowed;
            component.section179_carry = component.section179 - allowed;
        }
        for &(id, capacity) in &self.section179_risk {
            let allowed = sum(self
                .components
                .iter()
                .filter(|c| c.risk_group == id)
                .map(|c| c.section179_allowed))?;
            // The §179/ordinary risk cycle needs its own staged facts when binding.
            if allowed > capacity {
                return Err(UNAVAILABLE);
            }
        }
        for group in &mut self.groups {
            let allowed = sum(self
                .components
                .iter()
                .filter(|c| c.passive_group == group.id)
                .map(|c| c.section179_allowed))?;
            if self.components.iter().any(|c| c.passive_group == group.id && c.section179 != 0)
                && !group.material
            {
                return Err(UNAVAILABLE);
            }
            group.current = group.current.checked_sub(allowed).ok_or_else(range_error)?;
            group.nonpassive = group.nonpassive.checked_sub(allowed).ok_or_else(range_error)?;
        }
        Ok(self)
    }
}
