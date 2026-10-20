// Pub. 590-B inherited basis; separate Form 8606 per beneficiary/decedent.
// Unit6 PRINT (2026-10-13): PartI rows use this owner's actual QCD/funding
// exclusions, whole-dollar opening basis/recovery/carry and full-fraction
// arithmetic. A year-end balance alone does not require a distribution form.
// Tests: print_inherited_ira_* and corpus per-copy Form8606 footings.
// Scoped disaster correction (2026-10-13): the same group calculation now
// precedes Form8915 designation and collection. Linked designated cents use
// the group's after-QCD/funding taxable share, with prefix residual allocation.
// Tests: inherited_traditional_disaster_uses_its_group_basis_and_ira_spread;
// the original no-disaster basis/tax/carry oracles remain unchanged.
use crate::FederalResult;
use crate::federal_rules::valid_birth;
use crate::kernel::lines::{LineValue, emit as emit_lines, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::{dollars, fraction};
use alloc::string::String;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Inherited IRA
/// One for each person you inherited from and each kind of IRA: all their traditional, SEP and SIMPLE IRAs together, or all their Roth IRAs together. The Forms 1099-R you choose it on supply the distributions and withholding. A spouse who treated the IRA as their own enters it as their own IRA instead.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct InheritedIraAccount {
    /// Name of the person you inherited from
    #[libertas_size(min = 1)]
    pub decedent: String,
    /// Date they died
    #[libertas_date_only]
    // Tax structure: a current-return distribution or its source event cannot be after December 31, 2026.
    #[libertas_number(min = 19750101, max = 20261231)]
    pub died: u32,
    /// Kind of IRA
    pub kind: InheritedIraKindFacts,
}

/// Kind of IRA
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum InheritedIraKindFacts {
    /// Traditional, SEP or SIMPLE IRA
    /// All of them from this person together.
    Traditional {
        /// After-tax basis passed to you
        /// Leave empty when the person had no nondeductible contributions: their Forms 8606 and the estate's allocation show it.
        /// [NullText]
        /// No after-tax traditional IRA basis passed to me
        /// [SetText]
        /// The after-tax basis allocated to me is
        basis: Option<InheritedIraBasis>,
        /// Federal estate tax on this IRA
        /// Leave empty unless a federal estate tax return was filed and tax paid.
        /// No estate-tax deduction is claimed without these facts.
        /// [NullText]
        /// Do not claim an estate-tax deduction for this inherited IRA
        /// [SetText]
        /// The paid federal estate-tax facts for this IRA are
        #[libertas_default($null)]
        estate_tax: Option<InheritedEstateTax>,
    },
    /// Roth IRA
    /// All Roth IRAs from this person together.
    Roth {
        /// Their first Roth IRA year
        /// The first tax year they contributed to or converted into any Roth IRA. The five-year clock continues after the death.
        // Tax structure: Roth IRA history starts with 1998 and cannot begin after this tax year.
        #[libertas_number(min = 1998, max = 2026)]
        first_year: u32,
        /// Contributions and conversions passed to you
        /// Needed only when the five years are not over. Leave empty if you don't have it: the App then taxes the whole withdrawal and tells you.
        /// [NullText]
        /// No inherited Roth basis record is needed or available
        /// [SetText]
        /// The inherited contributions and conversions allocated to me are
        basis: Option<crate::InheritedRothBasis>,
    },
}

/// Inherited IRA
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalInheritedIra {
    /// Name of the person inherited from
    #[libertas_size(min = 1)]
    pub decedent: String,
    /// Original owner's death date
    #[libertas_date_only]
    pub died: u32,
    /// Kind of IRA and its allocated basis
    pub kind: InheritedIraKindFacts,
}

/// Basis allocated to this inherited IRA group
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct InheritedIraBasis {
    /// Unrecovered basis before this year's beneficiary payments
    /// Use this beneficiary's allocated basis from filed Form 8606 and estate/custodian records after prior withdrawals. For a 2026 death, use the completed estate allocation before this beneficiary's payments, including any death-year basis adjustment. Never use the entire decedent's basis when other beneficiaries received shares.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub prior: i64,
    /// Total December 31, 2026 value of this inherited IRA group
    /// Include every inherited traditional, SEP and SIMPLE IRA from this deceased owner belonging to this recipient, including accounts with no withdrawal. Exclude Roth IRAs, other deceased owners and the recipient's personal accounts. No transfer may be omitted from the year-end value; unresolved in-transit amounts need source reconciliation.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub year_end: i64,
}
/// Estate-return facts for inherited IRA income
/// The App calculates this year's section 691(c) deduction. Keep the administrator's remaining allocation for later distributions; never transfer another beneficiary's share or reuse value already used for a deduction.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct InheritedEstateTax {
    /// Federal estate tax after credits, actually paid
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub paid: i64,
    /// Estate tax after credits without net income-in-respect-of-decedent value
    /// Use the estate administrator's recomputation excluding all IRD value net of related deductible expenses. Do not subtract unrelated estate assets or state inheritance taxes.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub without_ird: i64,
    /// Total estate-tax value of all income in respect of the decedent
    /// Gross IRD value in the entire estate, before related expenses, including other beneficiaries' shares and non-IRA items. This is the allocation denominator, not just this IRA's value.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub total_ird: i64,
    /// Unused estate-tax IRD value allocated to this beneficiary's IRA group
    /// Start with the administrator's estate-tax value for this beneficiary and deceased owner, excluding after-tax basis and amounts already used for prior-year deductions. Do not include later investment growth or another beneficiary's share.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub remaining_value: i64,
    /// Final estate values, payment and unused beneficiary allocation confirmed
    /// No unresolved estate adjustment or duplicate allocation to another income entry, beneficiary or year. The App does not prepare Form 706. Surviving-annuitant special computations and capital-gain IRD use different calculations. No takes no estate-tax deduction this year, with a review note.
    pub confirmed: bool,
}
impl InheritedEstateTax {
    fn valid(&self) -> bool {
        [self.paid, self.without_ird, self.total_ird, self.remaining_value]
            .iter()
            .all(|v| (0..=100000000000).contains(v))
    }
    pub(crate) fn deduction(&self, r: &mut FederalResult, id: i64, taxable: Option<i64>) -> i64 {
        crate::require_localized!(
            r,
            // An unconfirmed estate-tax record was left out before the rules ran.
            self.confirmed
                && self.without_ird <= self.paid
                && self.total_ird > 0
                && self.remaining_value <= self.total_ird
                && taxable.is_some(),
            &crate::IssueContext::Review,
            "Inherited estate tax",
            "ISSUE_CONFIRM_PAID_FEDERAL_ESTATE_TAX_THE_ESTATE_WIDE_RECOMPUTATION_AND_G_2E262284",
            "Confirm paid federal estate tax, the estate-wide recomputation and gross IRD denominator, and this beneficiary's unused allocation. A shared QCD cap or deductible-contribution adjustment across several sources needs an explicit source allocation before this deduction can be determined.",
        );
        let used = phase::cap(phase::floor(taxable.unwrap_or(0), 0), [self.remaining_value]);
        let deduction = if self.total_ird > 0 {
            fraction(phase::floor(self.paid - self.without_ird, 0), used, self.total_ird)
        } else {
            0
        };
        emit_lines(
            r,
            &mut (),
            [
                (
                    "estate_tax_deduction",
                    deduction,
                    msg!("EXPLAIN_INHERITED_IRA_N_N_ESTATE_TAX_DEDUCTION"),
                ),
                (
                    "estate_ird_value_remaining",
                    self.remaining_value - used,
                    msg!("EXPLAIN_INHERITED_IRA_N_N_ESTATE_IRD_VALUE_REMAINING"),
                ),
            ],
            |(suffix, value, explanation), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!("inherited_ira_{id}_{suffix}"),
                    LineValue::MoneyDollars(dollars(value)),
                    &["income.inherited_ira.estate_tax"],
                    "https://www.irs.gov/publications/p559",
                    explanation,
                ))
            },
        );
        deduction
    }
}
impl InheritedIraAccount {
    // The page rule, checked again on save with the same key: the Roth IRA's
    // first year is not after the year of death.
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        matches!(&self.kind, InheritedIraKindFacts::Roth { first_year, .. }
                if *first_year > self.died / 10000)
        .then(|| {
            crate::validation::ValidationIssue::new("INHERITED_ROTH_FIRST_YEAR_AFTER_DEATH")
                .at_field("first_year")
                .in_field("kind")
        })
    }
    pub(crate) fn record(&self) -> FederalInheritedIra {
        FederalInheritedIra {
            decedent: self.decedent.clone(),
            died: self.died,
            kind: self.kind.clone(),
        }
    }
}
impl FederalInheritedIra {
    pub(crate) fn account(&self) -> InheritedIraAccount {
        InheritedIraAccount {
            decedent: self.decedent.clone(),
            died: self.died,
            kind: self.kind.clone(),
        }
    }
    pub(crate) fn roth(&self) -> bool {
        matches!(self.kind, InheritedIraKindFacts::Roth { .. })
    }
    // Section 408A(d)(2): a distribution is qualified once the original
    // owner's five-tax-year period is over; death satisfies the age leg.
    pub(crate) fn roth_qualified(&self) -> bool {
        matches!(self.kind, InheritedIraKindFacts::Roth { first_year, .. } if first_year <= 2021)
    }
    pub(crate) fn valid(&self) -> bool {
        valid_birth(self.died)
            && !self.decedent.trim().is_empty()
            && match &self.kind {
                InheritedIraKindFacts::Traditional { basis, estate_tax } => {
                    estate_tax.as_ref().is_none_or(InheritedEstateTax::valid)
                        && basis.as_ref().is_none_or(|b| {
                            [b.prior, b.year_end].iter().all(|v| (0..=100000000000).contains(v))
                        })
                }
                InheritedIraKindFacts::Roth { first_year, basis } => {
                    (1998..=self.died / 10000).contains(first_year)
                        && basis.as_ref().is_none_or(|v| {
                            [v.regular, v.converted].iter().all(|v| (0..=100000000000).contains(v))
                        })
                }
            }
    }
    // The traditional record's facts.
    pub(crate) fn basis(&self) -> Option<&InheritedIraBasis> {
        match &self.kind {
            InheritedIraKindFacts::Traditional { basis, .. } => basis.as_ref(),
            InheritedIraKindFacts::Roth { .. } => None,
        }
    }
    pub(crate) fn estate_tax(&self) -> Option<&InheritedEstateTax> {
        match &self.kind {
            InheritedIraKindFacts::Traditional { estate_tax, .. } => estate_tax.as_ref(),
            InheritedIraKindFacts::Roth { .. } => None,
        }
    }
    pub(crate) fn roth_basis(&self) -> Option<&crate::InheritedRothBasis> {
        match &self.kind {
            InheritedIraKindFacts::Roth { basis, .. } => basis.as_ref(),
            InheritedIraKindFacts::Traditional { .. } => None,
        }
    }
    // The traditional record's taxable amount for the linked payments'
    // `gross`. No basis entered means none: the whole group is taxed.
    pub(crate) fn taxable(
        &self,
        r: &mut FederalResult,
        id: i64,
        gross: i64,
        qualified_charity: i64,
        funding: i64,
        funding_basis: i64,
    ) -> i64 {
        let distributed = phase::floor(gross - qualified_charity - funding, 0);
        let gross = dollars(distributed);
        let prior = self.basis().map_or(0, |b| dollars(phase::floor(b.prior - funding_basis, 0)));
        let denominator = gross + self.basis().map_or(0, |b| dollars(b.year_end));
        let recovered = if denominator == 0 {
            0
        } else {
            phase::cap(
                phase::cap(fraction(gross, phase::cap(prior, [denominator]), denominator), [prior]),
                [gross],
            )
        };
        let ordinary_taxable = gross - recovered;
        let taxable = ordinary_taxable * 100 + qualified_charity;
        let carried = crate::kernel::carry::native_remaining::<crate::kernel::carry::Dollars>(
            prior, recovered,
        )
        .get();
        emit_lines(
            r,
            &mut (),
            [
                ("taxable", dollars(taxable), msg!("EXPLAIN_INHERITED_IRA_N_N_TAXABLE")),
                ("basis_recovered", recovered, msg!("EXPLAIN_INHERITED_IRA_N_N_BASIS_RECOVERED")),
                ("basis_remaining", carried, msg!("EXPLAIN_INHERITED_IRA_N_N_BASIS_REMAINING")),
            ],
            |(suffix, amount, explanation), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!("inherited_ira_{id}_{suffix}"),
                    LineValue::MoneyDollars(amount),
                    &["income.inherited_ira"],
                    "https://www.irs.gov/instructions/i8606",
                    explanation,
                ))
            },
        );
        // Only this decedent's remaining basis and ordinary distributions
        // establish its form; personal and other decedents' pools stay separate.
        if distributed > 0 && self.basis().is_some_and(|b| b.prior > funding_basis) {
            emit_lines(
                r,
                &mut (),
                [
                    ("1", LineValue::MoneyDollars(0)),
                    ("2", LineValue::MoneyDollars(prior)),
                    ("3", LineValue::MoneyDollars(prior)),
                    ("4", LineValue::MoneyDollars(0)),
                    ("5", LineValue::MoneyDollars(prior)),
                    ("6", LineValue::MoneyDollars(denominator - gross)),
                    ("7", LineValue::MoneyDollars(gross)),
                    ("8", LineValue::MoneyDollars(0)),
                    ("9", LineValue::MoneyDollars(denominator)),
                    (
                        "10",
                        LineValue::RatioTenThousandths(if denominator == 0 {
                            0
                        } else {
                            fraction(10_000, phase::cap(prior, [denominator]), denominator)
                        }),
                    ),
                    ("11", LineValue::MoneyDollars(0)),
                    ("12", LineValue::MoneyDollars(recovered)),
                    ("13", LineValue::MoneyDollars(recovered)),
                    ("14", LineValue::MoneyDollars(carried)),
                    ("15a", LineValue::MoneyDollars(ordinary_taxable)),
                ],
                |(number, value), _, _| {
                    Some(calculation_row(
                        Some(crate::FormLine::new(crate::TaxForm::Form8606, number)),
                        &alloc::format!("inherited_ira_{id}_form_line{number}"),
                        value,
                        &["income.inherited_ira", "income.retirement"],
                        "https://www.irs.gov/pub/irs-dft/i8606--dft.pdf",
                        msg!("EXPLAIN_INHERITED_IRA_FORM"),
                    ))
                },
            );
        }
        taxable
    }
}

// Compute each traditional group once before consumers need its taxable share.
// The same result is collected into Form1040 later; no second basis engine.
#[derive(Default)]
pub(crate) struct InheritedResult {
    pub taxable: alloc::collections::BTreeMap<i64, i64>,
    pub designated: alloc::collections::BTreeMap<i64, i64>,
}
pub(crate) fn calculate(
    d: &crate::FederalDraft,
    r: &mut FederalResult,
    qcd: &crate::retirement::QcdResult,
    funding: &crate::hsa_funding::FundingResult,
) -> InheritedResult {
    let mut result = InheritedResult::default();
    for entry in &d.income {
        let crate::FederalIncome::InheritedIra { data } = &entry.income else {
            continue;
        };
        if data.roth() {
            continue;
        }
        let gross = gross(d, entry.id);
        let gift = qcd.for_entry(entry.id);
        let funded = funding.for_inherited(entry.id);
        let taxable = data.taxable(
            r,
            entry.id,
            gross,
            gift,
            funded,
            funding.inherited_basis_used.get(&entry.id).copied().unwrap_or(0),
        );
        result.taxable.insert(entry.id, taxable);
        result.designated.extend(allocate_designated(
            d,
            r,
            entry.id,
            phase::floor(gross - gift - funded, 0),
            phase::floor(taxable - gift, 0),
        ));
    }
    result
}

// Form8606 sends this group's taxable share of its designated distributions
// to Form8915. Source policy is proportional within the beneficiary/decedent
// pool; monetary allocation uses the existing ratio and prefix kernels so
// linked payments conserve the exact designated cents, including the residual.
pub(crate) fn allocate_designated(
    d: &crate::FederalDraft,
    r: &mut FederalResult,
    group: i64,
    available: i64,
    taxable: i64,
) -> alloc::collections::BTreeMap<i64, i64> {
    let mut rows: alloc::vec::Vec<_> = payments(d, group)
        .into_iter()
        .filter_map(|(entry, _, _)| {
            let disaster = match &entry.income {
                crate::FederalIncome::Retirement { data } => &data.disaster,
                crate::FederalIncome::Roth { data } => &data.disaster,
                _ => return None,
            };
            match disaster {
                crate::DisasterDistribution::Qualified { amount, .. } => Some((entry.id, *amount)),
                _ => None,
            }
        })
        .collect();
    rows.sort_by_key(|(id, _)| *id);
    let designated: i64 = rows.iter().map(|(_, amount)| amount).sum();
    if designated == 0 {
        return Default::default();
    }
    let owner = d
        .income
        .iter()
        .find(|entry| entry.id == rows[0].0)
        .expect("existing inherited disaster payment");
    crate::require_localized!(
        r,
        designated <= available,
        &crate::interview::income_issue_context(owner),
        "Inherited IRA qualified disaster distributions",
        "ISSUE_INHERITED_DISASTER_OVERLAP",
        "The group's qualified disaster distributions cannot include amounts already excluded for a qualified charitable distribution or qualified HSA funding. Reconcile the actual linked payments.",
    );
    let designated_taxable = if available == 0 {
        0
    } else {
        fraction(taxable, phase::cap(designated, [available]), available)
    };
    let mut prefix = crate::kernel::ratio::Prefix::default();
    rows.into_iter()
        .map(|(id, amount)| {
            (
                id,
                prefix.next(amount, |cumulative| {
                    fraction(designated_taxable, cumulative, designated)
                }),
            )
        })
        .collect()
}

// Finish inherited form copies from the same designated-taxable owners
// used by Form8915. Other repayment/home-purchase treatments keep their
// separate source-owner acceptance; no zero relief is fabricated for them.
pub(crate) fn project_relief(
    d: &crate::FederalDraft,
    r: &mut FederalResult,
    basis: &crate::retirement::BasisResult,
) {
    for entry in &d.income {
        let crate::FederalIncome::InheritedIra { data } = &entry.income else {
            continue;
        };
        let special = payments(d, entry.id).iter().any(|(payment, _, _)| match &payment.income {
            crate::FederalIncome::Retirement { data } => !matches!(
                data.disaster,
                crate::DisasterDistribution::No | crate::DisasterDistribution::Qualified { .. }
            ),
            crate::FederalIncome::Roth { data } => {
                !matches!(
                    data.disaster,
                    crate::DisasterDistribution::No | crate::DisasterDistribution::Qualified { .. }
                ) || data.repaid.is_some()
                    || data.rolled_over > 0
            }
            _ => false,
        });
        if special {
            continue;
        }
        let kind = if data.roth() { "roth" } else { "ira" };
        let prefix = alloc::format!("inherited_{kind}_{}_form_line", entry.id);
        let (before, relief, after) =
            if data.roth() { ("25a", "25b", "25c") } else { ("15a", "15b", "15c") };
        let Some(taxable) = r
            .lines
            .iter()
            .find(|row| row.id == alloc::format!("{prefix}{before}"))
            .map(|row| row.amount)
        else {
            continue;
        };
        if data.roth() && taxable == 0 {
            continue;
        }
        let designated: i64 = payments(d, entry.id)
            .iter()
            .map(|(payment, _, _)| {
                if data.roth() {
                    basis.roth.designated.get(&payment.id)
                } else {
                    basis.inherited.designated.get(&payment.id)
                }
                .copied()
                .unwrap_or(0)
            })
            .sum();
        emit_lines(
            r,
            &mut (),
            [(relief, designated), (after, taxable - designated)],
            |(number, amount), _, _| {
                Some(calculation_row(
                    Some(crate::FormLine::new(crate::TaxForm::Form8606, number)),
                    &alloc::format!("{prefix}{number}"),
                    LineValue::MoneyCents(amount),
                    &["income.inherited_ira", "income.retirement"],
                    "https://www.irs.gov/pub/irs-dft/i8606--dft.pdf",
                    if data.roth() {
                        msg!("EXPLAIN_INHERITED_ROTH_FORM")
                    } else {
                        msg!("EXPLAIN_INHERITED_IRA_FORM")
                    },
                ))
            },
        );
    }
}

// Which record an inherited payment names (a Form 1099-R chosen on the
// Inherited IRA record, or an inherited Roth withdrawal). An explicit choice
// is the record's income identity; an empty choice uses the owner's only
// record of that kind ("one entered on this same page is chosen after you
// save it").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Link {
    Record(i64),
    // No record to use: none entered, or several and none chosen.
    Missing,
    // The chosen record belongs to the other filer.
    OtherOwner,
    // The chosen record is the other kind of IRA.
    WrongKind,
}
pub(crate) fn resolve(
    d: &crate::FederalDraft,
    owner: crate::Owner,
    roth: bool,
    account: Option<i64>,
) -> Link {
    use crate::FederalIncome;
    match account {
        Some(id) => match d.income.iter().find(|e| e.id == id) {
            Some(e) => match &e.income {
                FederalIncome::InheritedIra { .. } if e.owner != owner => Link::OtherOwner,
                FederalIncome::InheritedIra { data } if data.roth() != roth => Link::WrongKind,
                FederalIncome::InheritedIra { .. } => Link::Record(id),
                _ => Link::Missing,
            },
            None => Link::Missing,
        },
        None => {
            let mut records = d.income.iter().filter(|e| {
                e.owner == owner
                    && matches!(&e.income, FederalIncome::InheritedIra { data } if data.roth() == roth)
            });
            match (records.next(), records.next()) {
                (Some(e), None) => Link::Record(e.id),
                _ => Link::Missing,
            }
        }
    }
}
// The link of an entry that is an inherited payment: a Form 1099-R with
// `RetirementTaxable::Inherited` or a Roth withdrawal with an inherited link.
pub(crate) fn link_of(d: &crate::FederalDraft, e: &crate::FederalIncomeEntry) -> Option<Link> {
    use crate::{FederalIncome, InheritedLink, RetirementTaxable};
    match &e.income {
        FederalIncome::Retirement { data } => match data.taxable {
            RetirementTaxable::Inherited { account, .. } => {
                Some(resolve(d, e.owner, false, account))
            }
            _ => None,
        },
        FederalIncome::Roth { data } => match data.inherited {
            InheritedLink::Yes { account } => Some(resolve(d, e.owner, true, account)),
            InheritedLink::No => None,
        },
        _ => None,
    }
}
// The payments linked to a record: (entry, gross, withholding).
pub(crate) fn payments(
    d: &crate::FederalDraft,
    id: i64,
) -> alloc::vec::Vec<(&crate::FederalIncomeEntry, i64, i64)> {
    use crate::FederalIncome;
    d.income
        .iter()
        .filter(|e| link_of(d, e) == Some(Link::Record(id)))
        .filter_map(|e| match &e.income {
            FederalIncome::Retirement { data } => Some((e, data.gross, data.withholding)),
            FederalIncome::Roth { data } => Some((e, data.gross, data.withholding)),
            _ => None,
        })
        .collect()
}
// The record's 2026 gross distributions and withholding, summed over its payments.
pub(crate) fn gross(d: &crate::FederalDraft, id: i64) -> i64 {
    payments(d, id).iter().map(|(_, gross, _)| gross).sum()
}
// The blocking issue of a link that names the other filer's record or the
// other kind of IRA. A link with no record to use is a notice instead
// (unknowns.rs, WARN_INHERITED_IRA_RECORD_MISSING).
pub(crate) fn link_issue(r: &mut FederalResult, context: &crate::IssueContext, link: Link) {
    match link {
        Link::OtherOwner => crate::require_localized!(
            r,
            false,
            context,
            "Inherited IRA",
            "ISSUE_INHERITED_IRA_OTHER_OWNER",
            "This payment names an Inherited IRA that belongs to the other filer. Choose one of this filer's own.",
        ),
        Link::WrongKind => crate::require_localized!(
            r,
            false,
            context,
            "Inherited IRA",
            "ISSUE_INHERITED_IRA_KIND_MISMATCH",
            "This payment names an Inherited IRA of the other kind: a Roth IRA payment needs the Roth record and a traditional IRA payment the traditional one.",
        ),
        Link::Record(_) | Link::Missing => {}
    }
}
