pub use crate::foreign_allocation_source::ForeignAllocationSource;
use crate::foreign_allocation_source::class;
use crate::kernel::source_fact::SourceFact;
use crate::{
    FederalDraft, FederalIncome, ForeignOrdinaryTax, ForeignTaxBasis, ForeignTaxCategory,
    ForeignTaxClaim, ForeignWithholding, Owner, SourceCents,
};
use alloc::{format, string::String, vec::Vec};
use libertas_macros::*;

/// Fiduciary allocation of passive portfolio income and foreign tax
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EstateForeignPortfolioAllocation {
    /// IRS country code from the fiduciary's foreign-source statement
    /// Use the country of the income and tax, never the fiduciary's address.
    #[libertas_size(min = 2, max = 3)]
    #[libertas_llm_doc_field]
    pub country: String,
    /// Actual foreign withholding treatment of this payment
    /// Keep payments with different withholding treatments separate.
    #[libertas_llm_doc_field]
    pub withholding: ForeignWithholding,
    /// Eligible foreign income tax allocated to this payment
    /// The fiduciary's current-year beneficiary allocation, in U.S. dollars. Exclude refundable, contested or otherwise ineligible tax. These allocations must total box 14 code B; do not enter them again among the ordinary foreign-income sources.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_llm_doc_field(allow_empty = false)]
    pub tax: i64,
    /// Regular-tax source amounts
    #[libertas_llm_doc_field]
    pub regular: EstateForeignPortfolioAmounts,
    /// Independently established alternative-tax source amounts
    /// Copy the fiduciary's AMT allocation; aggregate box 12 code A does not establish foreign-source income.
    #[libertas_llm_doc_field]
    pub alternative: EstateForeignPortfolioAmounts,
}

/// Actual beneficiary foreign-source portfolio amounts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EstateForeignPortfolioAmounts {
    /// Foreign interest already included in box 1
    pub interest: SourceCents,
    /// Foreign ordinary dividends already included in box 2a, including qualified dividends
    pub dividends: SourceCents,
    /// Qualified dividends included in these foreign ordinary dividends
    pub qualified: SourceCents,
    /// Deductions definitely related to this income, excluding interest
    /// This portfolio allocation currently calculates only when records establish no directly related expense. Other beneficiary deductions retain their ordinary allocation.
    pub direct_expenses: SourceCents,
    /// Interest expense definitely related to this income
    /// This allocation currently calculates only when records establish no directly related interest expense.
    pub interest_expense: SourceCents,
}

impl EstateForeignPortfolioAmounts {
    pub(crate) fn values(&self) -> Option<[i64; 5]> {
        let value = |source: &SourceCents| match source.source_fact().ok()? {
            SourceFact::Reported(amount) => Some(amount),
            SourceFact::NotReported => Some(0),
            SourceFact::Unavailable => None,
        };
        Some([
            value(&self.interest)?,
            value(&self.dividends)?,
            value(&self.qualified)?,
            value(&self.direct_expenses)?,
            value(&self.interest_expense)?,
        ])
    }
}

fn key(parent: i64, country: &str, withholding: &ForeignWithholding) -> String {
    format!("estate/{parent}/{}/{}", country.trim().to_ascii_uppercase(), class(withholding))
}
// Transient Form 1116 values combine accepted sources without inheriting a
// single source's amount bounds. Only the original source records are stored.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ForeignDerivedSource {
    pub label: String,
    pub country: String,
    pub owner: Owner,
    pub category: ForeignTaxCategory,
    pub income: i64,
    pub tax: i64,
}
impl From<&ForeignOrdinaryTax> for ForeignDerivedSource {
    fn from(value: &ForeignOrdinaryTax) -> Self {
        Self {
            label: value.label.clone(),
            country: value.country.clone(),
            owner: value.owner,
            category: value.category.clone(),
            income: value.income,
            tax: value.tax,
        }
    }
}
impl From<&ForeignDerivedSource> for ForeignDerivedSource {
    fn from(value: &ForeignDerivedSource) -> Self {
        value.clone()
    }
}
impl ForeignDerivedSource {
    pub(crate) fn qualified(&self) -> i64 {
        self.category.qualified()
    }
    pub(crate) fn group(&self) -> usize {
        self.category.group(&self.country, self.tax)
    }
}

#[derive(Clone)]
pub(crate) struct ForeignCatalogSource {
    pub reference: ForeignAllocationSource,
    pub value: ForeignDerivedSource,
}
impl core::ops::Deref for ForeignCatalogSource {
    type Target = ForeignDerivedSource;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl ForeignCatalogSource {
    pub(crate) fn direct(value: impl Into<ForeignDerivedSource>) -> Self {
        let value = value.into();
        Self { reference: ForeignAllocationSource::Direct { label: value.label.clone() }, value }
    }
}
impl ForeignAllocationSource {
    pub(crate) fn matches(&self, source: &ForeignCatalogSource) -> bool {
        self.same_as(&source.reference)
    }
}

pub(crate) fn choices(d: &FederalDraft) -> Vec<crate::RecordChoice> {
    d.income
        .iter()
        .filter(|e| {
            matches!(&e.income, FederalIncome::EstateTrustPortfolio { data }
        if data.statements.foreign_statement.as_ref().is_some_and(|s| matches!(s.treatment,
            crate::EstateForeignTreatment::PassivePortfolio { .. })))
        })
        .map(|e| crate::RecordChoice {
            id: e.id,
            label: crate::income_names::income_display_label(e, d.dependents()).encode(),
        })
        .collect()
}

pub(crate) fn requested(d: &FederalDraft) -> bool {
    d.income.iter().any(|e| {
        matches!(&e.income, FederalIncome::EstateTrustPortfolio { data }
        if data.statements.foreign_statement.as_ref().is_some_and(|s| matches!(s.treatment,
            crate::EstateForeignTreatment::PassivePortfolio { .. })))
    })
}

// Source admission only. Form 1116 owns every allocation and credit calculation.
pub(crate) fn boundary(
    data: &crate::FederalEstateTrustPortfolioK1,
    claim: &ForeignTaxClaim,
) -> Option<&'static str> {
    let Some(statement) = &data.foreign_statement else {
        return None;
    };
    let crate::EstateForeignTreatment::PassivePortfolio { basis, allocations } =
        &statement.treatment
    else {
        return None;
    };
    match claim {
        ForeignTaxClaim::Ordinary { basis: chosen, .. } if chosen != basis => {
            return Some("WARN_ESTATE_FOREIGN_BASIS_LEFT_OUT");
        }
        ForeignTaxClaim::Deduction { .. } if *basis == ForeignTaxBasis::Accrued => {
            return Some("WARN_ESTATE_FOREIGN_BASIS_LEFT_OUT");
        }
        ForeignTaxClaim::Deduction { .. } => {
            return if allocations.is_empty() {
                Some("WARN_ESTATE_FOREIGN_ALLOCATION_LEFT_OUT")
            } else {
                None
            };
        }
        ForeignTaxClaim::No => return None,
        ForeignTaxClaim::Statements { .. } => return Some("WARN_ESTATE_FOREIGN_FORM1116_NEEDED"),
        _ => {}
    }
    if let ForeignTaxClaim::Ordinary { sources, .. } = claim
        && sources.iter().any(|s| matches!(s.category, ForeignTaxCategory::PriorPassiveTax { .. }))
        && allocations.iter().any(|row| {
            matches!(row.withholding, ForeignWithholding::None)
                && row.tax > 0
                && row
                    .regular
                    .values()
                    .is_some_and(|v| v[0].checked_add(v[1]).is_some_and(|income| income > 0))
        })
    {
        return Some("WARN_ESTATE_FOREIGN_SOURCE_YEAR_LEFT_OUT");
    }
    if allocations.is_empty() {
        return Some("WARN_ESTATE_FOREIGN_ALLOCATION_LEFT_OUT");
    }
    if crate::estate_trust::foreign_portfolio_capacities(data).is_none() {
        return Some("WARN_ESTATE_FOREIGN_AMT_LEFT_OUT");
    }
    for row in allocations {
        if crate::year_constants::is_section_901j_country(&row.country) {
            return Some("WARN_ESTATE_FOREIGN_ALLOCATION_LEFT_OUT");
        }
        for amounts in [&row.regular, &row.alternative] {
            let Some(values) = amounts.values() else {
                return Some("WARN_ESTATE_FOREIGN_FACTS_LEFT_OUT");
            };
            if values[3] != 0 || values[4] != 0 {
                return Some("WARN_ESTATE_FOREIGN_EXPENSES_LEFT_OUT");
            }
        }
    }
    None
}

pub(crate) fn sources(d: &FederalDraft, amt: bool) -> Vec<ForeignCatalogSource> {
    let mut out: Vec<ForeignCatalogSource> = Vec::new();
    let Some(credits) = &d.credits else {
        return out;
    };
    let claim = &credits.foreign_tax;
    if d.income.iter().any(|entry| matches!(&entry.income, FederalIncome::EstateTrustPortfolio { data } if source_issue(&crate::FederalEstateTrustPortfolioK1::from(data)).is_some())) { return out; }
    for entry in crate::community::entries(d) {
        let Ok(original) = crate::source_associations::income_source(d, entry.id) else {
            continue;
        };
        let FederalIncome::EstateTrustPortfolio { data: accepted } = &original.income else {
            continue;
        };
        if boundary(&crate::FederalEstateTrustPortfolioK1::from(accepted), claim).is_some() {
            continue;
        }
        // Original source eligibility is settled before community projection.
        // An allocated cent may become zero without changing its payment class.
        let FederalIncome::EstateTrustPortfolio { data } = &entry.income else {
            continue;
        };
        let data = &crate::FederalEstateTrustPortfolioK1::from(data);
        let Some(statement) = &data.foreign_statement else {
            continue;
        };
        let crate::EstateForeignTreatment::PassivePortfolio { allocations, .. } =
            &statement.treatment
        else {
            continue;
        };
        for row in allocations {
            let Some(values) = (if amt { &row.alternative } else { &row.regular }).values() else {
                continue;
            };
            let label = key(entry.id, &row.country, &row.withholding);
            let reference = ForeignAllocationSource::Estate {
                parent: entry.id,
                country: row.country.clone(),
                withholding: row.withholding.clone(),
            };
            if let Some(existing) = out.iter_mut().find(|source| reference.matches(source)) {
                // Validated nonnegative source sums are bounded by this parent's
                // exact box amounts, before and after community projection.
                existing.value.income += values[0] + values[1];
                existing.value.tax += row.tax;
                if let ForeignTaxCategory::Passive { qualified, .. } = &mut existing.value.category
                {
                    *qualified += values[2];
                }
            } else {
                out.push(ForeignCatalogSource {
                    reference,
                    value: ForeignDerivedSource {
                        label,
                        country: row.country.clone(),
                        owner: entry.owner.owned(),
                        category: ForeignTaxCategory::Passive {
                            withholding: row.withholding.clone(),
                            qualified: values[2],
                            sales: Vec::new(),
                        },
                        income: values[0] + values[1],
                        tax: row.tax,
                    },
                });
            }
        }
    }
    out
}

pub(crate) fn warn(d: &FederalDraft, r: &mut crate::FederalResult) {
    let Some(credits) = &d.credits else {
        return;
    };
    for entry in &d.income {
        if let FederalIncome::EstateTrustPortfolio { data } = &entry.income
            && let data = &crate::FederalEstateTrustPortfolioK1::from(data)
            && source_issue(data).is_none()
            && let Some(key) = boundary(data, &credits.foreign_tax)
        {
            crate::federal_rules::note_once_scoped(
                r,
                crate::Message::new(key, Vec::new()),
                crate::interview::income_issue_context(entry),
            );
        }
    }
}

pub(crate) fn source_issue(
    data: &crate::FederalEstateTrustPortfolioK1,
) -> Option<crate::validation::ValidationIssue> {
    use crate::validation::ValidationIssue;
    let statement = data.foreign_statement.as_ref()?;
    if !(0..=100_000_000_000).contains(&statement.tax) {
        return Some(
            ValidationIssue::new("ESTATE_FOREIGN_AMOUNT_INVALID")
                .at_field("tax")
                .in_field("foreign_statement"),
        );
    }
    let crate::EstateForeignTreatment::PassivePortfolio { allocations, .. } = &statement.treatment
    else {
        return None;
    };
    let issue = |field: &'static str, index: usize| {
        ValidationIssue::new("ESTATE_FOREIGN_ALLOCATION_INVALID")
            .at_field(field)
            .in_list("allocations", index)
            .in_field("treatment")
            .in_field("foreign_statement")
    };
    let amount_issue = |field: &'static str, system: usize, index: usize| {
        ValidationIssue::new("ESTATE_FOREIGN_ALLOCATION_INVALID")
            .at_field("amount")
            .in_field(field)
            .in_field(if system == 0 { "regular" } else { "alternative" })
            .in_list("allocations", index)
            .in_field("treatment")
            .in_field("foreign_statement")
    };
    let mut tax = 0i64;
    let mut totals = [[0i64; 3]; 2];
    for (index, row) in allocations.iter().enumerate() {
        if !crate::form_1116::country_size_valid(&row.country)
            || (row.country.chars().count() != 2 && row.country != "RIC")
        {
            return Some(issue("country", index));
        }
        if row.tax < 0 || (!matches!(row.withholding, ForeignWithholding::None) && row.tax == 0) {
            return Some(issue("tax", index));
        }
        let Some(next) = tax.checked_add(row.tax) else {
            return Some(issue("tax", index));
        };
        tax = next;
        for (system, amounts) in [&row.regular, &row.alternative].into_iter().enumerate() {
            for (field, value) in [
                ("interest", &amounts.interest),
                ("dividends", &amounts.dividends),
                ("qualified", &amounts.qualified),
                ("direct_expenses", &amounts.direct_expenses),
                ("interest_expense", &amounts.interest_expense),
            ] {
                if value.source_fact().is_err() {
                    return Some(amount_issue(field, system, index));
                }
            }
            let known =
                [&amounts.interest, &amounts.dividends, &amounts.qualified].map(
                    |value| match value {
                        SourceCents::Reported { amount } => Some(*amount),
                        SourceCents::NotReported => Some(0),
                        SourceCents::Unavailable => None,
                    },
                );
            if let (Some(dividends), Some(qualified)) = (known[1], known[2])
                && qualified > dividends
            {
                return Some(amount_issue("qualified", system, index));
            }
            for (column, amount) in known.into_iter().enumerate() {
                let Some(next) = totals[system][column].checked_add(amount.unwrap_or(0)) else {
                    return Some(amount_issue(
                        ["interest", "dividends", "qualified"][column],
                        system,
                        index,
                    ));
                };
                totals[system][column] = next;
            }
        }
    }
    if !allocations.is_empty() && tax != statement.tax {
        return Some(
            ValidationIssue::new("ESTATE_FOREIGN_TAX_TOTAL_INVALID")
                .at_field("tax")
                .in_field("foreign_statement"),
        );
    }
    let capacities = crate::estate_trust::foreign_portfolio_capacities(data);
    let regular = crate::estate_trust::regular_foreign_portfolio_capacity(data)?;
    for (system, total) in totals.iter().enumerate() {
        let capacity = if system == 0 { Some(regular) } else { capacities.map(|v| v[1]) };
        if capacity.is_some_and(|cap| total.iter().zip(cap).any(|(value, limit)| *value > limit)) {
            return Some(
                ValidationIssue::new("ESTATE_FOREIGN_INCOME_TOTAL_INVALID")
                    .at_field("treatment")
                    .in_field("foreign_statement"),
            );
        }
    }
    None
}

// Community allocation follows the estate owner's qualified/REIT/ordinary
// category order. Prefix projection keeps country sums inside their parent's
// exact allocated cents, independent of editable row order.
pub(crate) fn project(
    source: &crate::FederalEstateTrustPortfolioK1,
    projected: &mut crate::FederalEstateTrustPortfolioK1,
    share: crate::community::Share,
) {
    if source_issue(source).is_some() {
        return;
    }
    let Some(ordinary_start) =
        source.qualified_dividends.checked_add(source.section199a_dividends.amount())
    else {
        return;
    };
    let (Some(original), Some(destination)) =
        (&source.foreign_statement, &mut projected.foreign_statement)
    else {
        return;
    };
    let (
        crate::EstateForeignTreatment::PassivePortfolio { allocations: rows, .. },
        crate::EstateForeignTreatment::PassivePortfolio { allocations: targets, .. },
    ) = (&original.treatment, &mut destination.treatment)
    else {
        return;
    };
    let values: Option<Vec<_>> =
        rows.iter().map(|row| Some([row.regular.values()?, row.alternative.values()?])).collect();
    let Some(values) = values else {
        return;
    };
    let mut order: Vec<_> = (0..rows.len()).collect();
    order.sort_by_key(|&i| {
        (rows[i].country.trim().to_ascii_uppercase(), class(&rows[i].withholding), values[i])
    });
    let mut tax = crate::kernel::ratio::Prefix::default();
    let mut interest: [_; 2] = core::array::from_fn(|_| crate::kernel::ratio::Prefix::default());
    let mut qualified: [_; 2] = core::array::from_fn(|_| crate::kernel::ratio::Prefix::default());
    let mut ordinary: [_; 2] = core::array::from_fn(|_| crate::kernel::ratio::Prefix::default());
    for prefix in &mut ordinary {
        prefix.next(ordinary_start, |through| share.filer(through));
    }
    let project_amount = |source: &SourceCents, amount| match source {
        SourceCents::Reported { .. } => SourceCents::Reported { amount },
        _ => source.clone(),
    };
    for index in order {
        targets[index].tax = tax.next(rows[index].tax, |through| share.filer(through));
        for system in 0..2 {
            let values = values[index][system];
            let original =
                if system == 0 { &rows[index].regular } else { &rows[index].alternative };
            let target = if system == 0 {
                &mut targets[index].regular
            } else {
                &mut targets[index].alternative
            };
            let q = qualified[system].next(values[2], |through| share.filer(through));
            let d =
                q + ordinary[system].next(values[1] - values[2], |through| share.filer(through));
            target.interest = project_amount(
                &original.interest,
                interest[system].next(values[0], |through| share.filer(through)),
            );
            target.qualified = project_amount(&original.qualified, q);
            target.dividends = project_amount(&original.dividends, d);
        }
    }
}

pub(crate) fn check(d: &FederalDraft, r: &mut crate::FederalResult) {
    for entry in &d.income {
        if let FederalIncome::EstateTrustPortfolio { data } = &entry.income {
            let data = &crate::FederalEstateTrustPortfolioK1::from(data);
            if let Some(issue) = source_issue(data) {
                crate::require_localized!(
                    r,
                    false,
                    &crate::interview::income_issue_context(entry),
                    "Fiduciary foreign-source allocation",
                    issue.key,
                    "The fiduciary foreign-source amounts are inconsistent or cannot be represented. Correct this K-1 before calculating its foreign tax."
                );
            }
            if data.foreign_statement.as_ref().is_some_and(|s| {
                matches!(s.treatment, crate::EstateForeignTreatment::PassivePortfolio { .. })
            }) {
                crate::require_localized!(
                    r,
                    crate::source_associations::income_source(d, entry.id).is_ok(),
                    &crate::interview::income_issue_context(entry),
                    "Fiduciary foreign-source identity",
                    "ESTATE_FOREIGN_PARENT_INVALID",
                    "This foreign allocation needs one unambiguous accepted estate or trust K-1 identity."
                );
            }
        }
    }
}

pub(crate) fn admitted(d: &FederalDraft, parent: i64) -> bool {
    let (Some(credits), Ok(source)) =
        (&d.credits, crate::source_associations::income_source(d, parent))
    else {
        return false;
    };
    let FederalIncome::EstateTrustPortfolio { data } = &source.income else {
        return false;
    };
    let data = &crate::FederalEstateTrustPortfolioK1::from(data);
    data.foreign_statement.as_ref().is_some_and(|statement| {
        matches!(statement.treatment, crate::EstateForeignTreatment::PassivePortfolio { .. })
    }) && source_issue(data).is_none()
        && boundary(data, &credits.foreign_tax).is_none()
}
