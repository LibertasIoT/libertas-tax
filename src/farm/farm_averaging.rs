// Coverage: Schedule J — Partial (updated 2026-10-11).
//   Lines: Elected ordinary Schedule F income (eligible-income limit after owner deductions/QBI,
//   capped at taxable income), 1/3 stacking on 2023–2025 base years with base-year rate schedules,
//   prior Schedule J adjustments, negative base income, retained foreign exclusions, AMT line 10
//   consistency; election is user-chosen, not optimized. Additional source/history limits
//   omit only averaging, preserving ordinary tax. Pending: capital-gain election, fishing,
//   farm wages, K-1/crop-share farming, two farming spouses with separate histories, mixed QBI
//   caps, prior NOLs, Archer MSA, young-filer tax, unentered 2555/4952 facts.
//   Evidence: farm_averaging.rs, income_tax.rs;
//   farm.rs::farm_averaging_recomputes_base_year_rate_schedules_not_filing_tables,
//   farm.rs::farm_averaging_negative_base_income_uses_retained_loss_components,
//   farm_specials.rs::farm_averaging_named_history_and_source_limits_preserve_ordinary_tax_and_records,
//   farm.rs::farm_averaging_warns_but_does_not_replace_the_taxpayers_election; i1040sj, i6251 line
//   10

// Section 1301; retained prior-return facts, not user-calculated current tax.
// https://www.irs.gov/instructions/i1040sj
// https://www.irs.gov/instructions/i6251 (line 10)
pub use crate::farm_averaging_source::{
    AveragingBaseYear, AveragingCapital, AveragingForeign, AveragingForeignForm,
    AveragingInvestment, AveragingLoss, AveragingNol, AveragingOtherTax, AveragingSource,
    FarmAveraging,
};
use crate::kernel::lines::{LineValue, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::{dollars, fraction, positive};
use crate::line_emission::emit as emit_lines;
use crate::line_emission::line;
use crate::{FederalResult, FederalStatus};
const SOURCE: &str = "https://www.irs.gov/instructions/i1040sj";

impl FarmAveraging {
    pub(crate) fn unavailable(&self, d: &crate::FederalDraft) -> Option<&'static str> {
        use crate::FarmAveragingHistoryScope as H;
        if !matches!(self.selection.history_scope, H::CompleteSupported) {
            Some("WARN_FARM_AVERAGING_HISTORY_LEFT_OUT")
        } else if d.income.iter().any(|entry| matches!(&entry.income, crate::FederalIncome::Business {data} if matches!(&data.activity,
            crate::BusinessActivity::Farming {data:farm} if match &farm.accounting {
                crate::FarmAccounting::Cash {receipts} => receipts.iter().any(|receipt| matches!(receipt,
                    crate::FarmReceipt::CustomHire {amount} | crate::FarmReceipt::Resale {amount,..} if *amount > 0)),
                crate::FarmAccounting::Accrual {custom_hire,..} => *custom_hire > 0,
            }))) {
            // Gross receipts alone do not establish net attributable income
            // from excluded contract harvesting or merely reselling products.
            Some("WARN_FARM_AVERAGING_PRODUCTION_SOURCE_LEFT_OUT")
        } else if self.selection.capital_elected > 0 {
            // The capital algorithm requires actual attributable property-gain
            // output from the shared disposition owner, not raw sale proceeds.
            Some("WARN_FARM_AVERAGING_CAPITAL_SOURCE_LEFT_OUT")
        } else if self.selection.sources.iter().any(|id| !d.income.iter().any(|entry| entry.id == *id && matches!(&entry.income, crate::FederalIncome::Business {data} if matches!(data.activity, crate::BusinessActivity::Farming {..})))) {
            // Existing ordinary F sources are automatic. Additional W2/K1/
            // fishing/share-rent sources still need their final attribution.
            Some("WARN_FARM_AVERAGING_ADDITIONAL_SOURCE_LEFT_OUT")
        } else {
            None
        }
    }
    pub(crate) fn sources_issue(
        &self,
        d: &crate::FederalDraft,
    ) -> Option<crate::validation::ValidationIssue> {
        let catalog = crate::farm_specials::source_choices(d);
        self.selection
            .sources
            .iter()
            .position(|id| !catalog.iter().any(|source| source.id == *id))
            .map(|index| {
                crate::validation::ValidationIssue::new("FARM_AVERAGING_SOURCES_INVALID")
                    .at_list_item("sources", index)
                    .in_field("selection")
            })
    }
    pub(crate) fn sources_valid(&self, d: &crate::FederalDraft) -> bool {
        self.sources_issue(d).is_none()
    }
}
impl AveragingBaseYear {
    fn retained(&self, r: &mut FederalResult) -> (i64, i64) {
        match &self.source {
            AveragingSource::ScheduleJ { taxable, tax, .. } => (dollars(*taxable), dollars(*tax)),
            AveragingSource::Return { agi, deductions, tax, other_tax, loss } => {
                let other: i64 = other_tax.iter().map(|v| v.amount).sum();
                crate::require_localized!(
                    r,
                    other <= *tax,
                    &crate::IssueContext::Section(crate::TaxSection::Farming),
                    "Farm averaging",
                    "ISSUE_OTHER_TAXES_CANNOT_EXCEED_THE_RETAINED_FORM_1040_LINE_16_AMOUNT_A3DD421C",
                    "Other taxes cannot exceed the retained Form 1040 line 16 amount.",
                );
                let mut taxable = dollars(*agi) - dollars(*deductions);
                crate::require_localized!(
                    r,
                    (taxable < 0) == loss.is_some(),
                    &crate::IssueContext::Section(crate::TaxSection::Farming),
                    "Farm averaging",
                    "ISSUE_SUPPLY_NEGATIVE_INCOME_RECORDS_EXACTLY_WHEN_THE_BASE_YEAR_S_AGI_MIN_DF6BD01E",
                    "Supply negative-income records exactly when the base year's AGI minus deductions is negative.",
                );
                if let Some(l) = loss {
                    let cap = dollars(l.capital_deduction + l.capital_carry - l.capital_loss);
                    crate::require_localized!(
                        r,
                        cap >= 0 && cap <= dollars(l.capital_deduction),
                        &crate::IssueContext::Section(crate::TaxSection::Farming),
                        "Farm averaging",
                        "ISSUE_RECONCILE_THE_BASE_YEAR_CAPITAL_LOSS_DEDUCTION_AND_CARRYOVER_THE_UN_C2A79762",
                        "Reconcile the base-year capital-loss deduction and carryover; the unused deduction must be between zero and the deduction claimed.",
                    );
                    let nol = match l.nol {
                        AveragingNol::None => 0,
                        AveragingNol::Current { amount, prior_deduction } => {
                            taxable += dollars(prior_deduction);
                            dollars(amount)
                        }
                        AveragingNol::Unused { amount } => dollars(amount),
                    };
                    taxable += cap + nol;
                    crate::require_localized!(
                        r,
                        taxable <= 0,
                        &crate::IssueContext::Section(crate::TaxSection::Farming),
                        "Farm averaging",
                        "ISSUE_THE_RETAINED_NEGATIVE_INCOME_ADJUSTMENTS_EXCEED_THE_BASE_YEAR_DEFIC_3CA2371E",
                        "The retained negative-income adjustments exceed the base-year deficit; reconcile the NOL and capital-loss worksheets.",
                    );
                }
                (taxable, dollars(*tax - other))
            }
        }
    }
    fn tax(&self, taxable: i64, r: &mut FederalResult) -> i64 {
        self.tax_with_prefix(taxable, r, "farm_averaging")
    }
    fn tax_with_prefix(&self, taxable: i64, r: &mut FederalResult, prefix: &str) -> i64 {
        let capital = &self.capital;
        let (mut gain, mut line18, mut line19) = (
            phase::cap(dollars(capital.long_term), [dollars(capital.net)]),
            dollars(capital.collectibles),
            dollars(capital.unrecaptured),
        );
        let mut dividends = dollars(self.dividends);
        if let Some(investment) = &self.investment {
            let elected = dollars(investment.elected);
            let capital = phase::cap(elected, [dollars(investment.capital)]);
            let dividend_election = elected - capital;
            crate::require_localized!(
                r,
                dividend_election <= dividends,
                &crate::IssueContext::Section(crate::TaxSection::Farming),
                "Farm averaging",
                "ISSUE_THE_RETAINED_FORM_4952_DIVIDEND_ELECTION_EXCEEDS_THE_BASE_YEAR_QUAL_047D93F4",
                "The retained Form 4952 dividend election exceeds the base-year qualified dividends; reconcile lines 4g and 4e, including any dotted-line allocation.",
            );
            gain = positive(gain - capital);
            dividends = positive(dividends - dividend_election);
            emit_lines(
                r,
                &mut (),
                [("investment_capital", capital), ("investment_dividends", dividend_election)],
                |(suffix, value), _, _| {
                    Some(calculation_row(
                        None,
                        &alloc::format!("{prefix}_{}_{suffix}", self.year),
                        LineValue::MoneyDollars(value),
                        &["screening.farm_averaging.years.investment"],
                        SOURCE,
                        msg!("EXPLAIN_N_N_N"),
                    ))
                },
            );
        }
        let mut stacking = 0;
        if let Some(foreign) = &self.foreign {
            crate::require_localized!(
                r,
                foreign.forms.iter().all(|v| {
                    v.owner == crate::Owner::Taxpayer || self.status == FederalStatus::Joint
                }) && foreign
                    .forms
                    .iter()
                    .enumerate()
                    .all(|(i, v)| !foreign.forms[..i].iter().any(|p| p.owner == v.owner)),
                &crate::IssueContext::Section(crate::TaxSection::Farming),
                "Farm averaging",
                "ISSUE_RETAIN_ONLY_ONE_FORM_2555_PER_BASE_YEAR_FILER_A_SPOUSE_S_FORM_BELON_B6664F34",
                "Retain only one Form 2555 per base-year filer; a spouse's form belongs only on that year's joint return.",
            );
            stacking = foreign.stacking();
            // Schedule J's foreign worksheet is skipped for nonpositive income.
            let excess = if taxable > 0 { positive(dividends + gain - taxable) } else { 0 };
            if taxable > 0 {
                let adjusted = crate::income_tax::Preference { dividends, gain, line18, line19 }
                    .foreign(taxable);
                dividends = adjusted.dividends;
                gain = adjusted.gain;
                line18 = adjusted.line18;
                line19 = adjusted.line19;
            }
            emit_lines(
                r,
                &mut (),
                [
                    ("foreign_stacking", stacking),
                    ("foreign_gain_excess", excess),
                    ("foreign_net_gain", gain),
                    ("foreign_dividends", dividends),
                    ("foreign_line18", line18),
                    ("foreign_line19", line19),
                ],
                |(suffix, value), _, _| {
                    Some(calculation_row(
                        None,
                        &alloc::format!("{prefix}_{}_{suffix}", self.year),
                        LineValue::MoneyDollars(value),
                        &[
                            "screening.farm_averaging.years.foreign",
                            "screening.farm_averaging.years.capital",
                        ],
                        SOURCE,
                        msg!("EXPLAIN_N_N_N_2"),
                    ))
                },
            );
            if taxable <= 0 {
                return 0;
            }
        }
        let unrecaptured = phase::cap(line19, [gain]);
        let collectibles = phase::cap(line18, [gain - unrecaptured]);
        let rates = crate::income_tax::Rates::for_year(self.year, self.status).unwrap();
        positive(
            rates.schedule_d(
                positive(taxable) + stacking,
                dividends + gain,
                collectibles + unrecaptured,
                unrecaptured,
                collectibles > 0,
            ) - dollars(rates.ordinary_cents(stacking)),
        )
    }
}

pub(crate) struct Current {
    pub status: FederalStatus,
    pub taxable: i64,
    pub regular_tax: i64,
    pub preference: crate::income_tax::Preference,
    pub eligible: i64,
    pub excluded: i64,
    pub foreign_election: bool,
    pub shared_history_required: bool,
    pub scope_ready: bool,
}

// A comparison retains the chosen election subject to its own taxable-income
// ceiling. Reuse validated source years if that ceiling reduces the allocation.
#[derive(Clone, Copy)]
pub(crate) struct Worksheet<'a> {
    elected: i64,
    base_delta: i64,
    years: &'a [AveragingBaseYear],
}
impl Worksheet<'_> {
    // Reuse the validated Schedule J owner for reduced-tax comparisons.
    // Its explanatory rows belong only to the main calculation.
    pub(crate) fn tax_value(self, current: crate::income_tax::RegularTaxWorksheet) -> i64 {
        self.tax(current, &mut FederalResult::empty(0))
    }
    pub(crate) fn tax(
        self,
        current: crate::income_tax::RegularTaxWorksheet,
        r: &mut FederalResult,
    ) -> i64 {
        let elected = phase::cap(self.elected, [positive(current.taxable)]);
        let share = fraction(elected, 1, 3);
        let base_delta = if elected == self.elected {
            self.base_delta
        } else if elected == 0 {
            // No hypothetical election means no base-year allocation. Do not
            // turn rounding differences in filed base-year tax into new tax.
            0
        } else {
            let mut delta = 0;
            for year in self.years {
                let (base, old_tax) = year.retained(r);
                let new_tax = year.tax_with_prefix(base + share, r, "farm_installment_averaging");
                delta += new_tax - old_tax;
                emit_lines(
                    r,
                    &mut (),
                    [
                        (
                            "allocated_income",
                            if year.year == 2023 { positive(base + share) } else { base + share },
                        ),
                        ("recomputed_tax", new_tax),
                    ],
                    |(suffix, value), _, _| {
                        Some(calculation_row(
                            None,
                            &alloc::format!("farm_installment_averaging_{}_{suffix}", year.year),
                            LineValue::MoneyDollars(value),
                            &["screening.farm_averaging"],
                            SOURCE,
                            msg!("EXPLAIN_FARM_INSTALLMENT_AVERAGING_N_N"),
                        ))
                    },
                );
            }
            delta
        };
        line(
            r,
            "farm_installment_averaging_elected_income",
            elected,
            &["screening.farm_averaging"],
            SOURCE,
            msg!("EXPLAIN_FARM_INSTALLMENT_AVERAGING_ELECTED_INCOME"),
        );
        positive(current.less(elected).tax() + base_delta)
    }
}

pub(crate) struct Calculation<'a> {
    pub tax: i64,
    pub comparison: Option<Worksheet<'a>>,
}
impl Calculation<'_> {
    fn unchanged(tax: i64) -> Self {
        Self { tax, comparison: None }
    }
}

pub(crate) fn calculate<'a>(
    facts: Option<&'a FarmAveraging>,
    current: Current,
    r: &mut FederalResult,
) -> Calculation<'a> {
    let Some(facts) = facts else {
        return Calculation::unchanged(current.regular_tax);
    };
    let before = r.issues.len();
    crate::require_localized!(
        r,
        facts.valid(),
        &crate::IssueContext::Section(crate::TaxSection::Farming),
        "Farm averaging",
        "ISSUE_SUPPLY_A_POSITIVE_ELECTION_AND_ALL_THREE_VALID_BASE_YEAR_RECORDS_82AB2094",
        "Supply a positive election and all three valid base-year records.",
    );
    if !facts.valid() {
        return Calculation::unchanged(current.regular_tax);
    }
    if !current.scope_ready
        || (current.shared_history_required
            && facts.years.iter().any(|b| b.status != FederalStatus::Joint))
    {
        crate::federal_rules::note_once_scoped(
            r,
            msg!("WARN_FARM_AVERAGING_ALLOCATION_LEFT_OUT"),
            crate::IssueContext::Section(crate::TaxSection::Farming),
        );
        return Calculation::unchanged(current.regular_tax);
    }
    let elected = dollars(facts.elected);
    crate::require_localized!(
        r,
        elected > 0 && elected <= phase::cap(positive(current.eligible), [current.taxable]),
        &crate::IssueContext::Section(crate::TaxSection::Farming),
        "Farm averaging",
        "ISSUE_THE_ELECTED_AMOUNT_MUST_BE_POSITIVE_AND_CANNOT_EXCEED_ELIGIBLE_FARM_DF8EA144",
        "The elected amount must be positive and cannot exceed eligible farm income after attributable deductions or current taxable income.",
    );
    let latest = facts
        .years
        .iter()
        .filter_map(|b| match b.source {
            AveragingSource::ScheduleJ { year, .. } => Some(year),
            _ => None,
        })
        .max();
    for year in 2023..=2025 {
        crate::require_localized!(
            r,
            facts.years.iter().filter(|b| b.year == year).count() == 1,
            &crate::IssueContext::Section(crate::TaxSection::Farming),
            "Farm averaging",
            "ISSUE_ENTER_THREE_PRIOR_YEARS_EXACTLY_ONCE_11CCA3B9",
            "Enter 2023, 2024 and 2025 exactly once.",
        );
    }
    let share = fraction(elected, 1, 3);
    let reduced = crate::income_tax::RegularTaxWorksheet {
        taxable: current.taxable,
        preference: current.preference,
        excluded: current.excluded,
        foreign_election: current.foreign_election,
        status: current.status,
    }
    .less(elected)
    .tax();
    let mut result = reduced;
    for b in &facts.years {
        crate::require_localized!(
            r,
            !current.shared_history_required || b.status == FederalStatus::Joint,
            &crate::IssueContext::Section(crate::TaxSection::Farming),
            "Farm averaging",
            "ISSUE_TWO_FARMING_SPOUSES_WITH_SEPARATE_BASE_YEAR_RETURNS_REQUIRE_SEPARAT_45269592",
            "Two farming spouses with separate base-year returns require separate histories and an election allocation. A sole farmer may use differing actual filing statuses.",
        );
        let wanted = latest.filter(|year| *year >= b.year);
        let actual = match b.source {
            AveragingSource::ScheduleJ { year, .. } => Some(year),
            _ => None,
        };
        crate::require_localized!(
            r,
            wanted == actual,
            &crate::IssueContext::Section(crate::TaxSection::Farming),
            "Farm averaging",
            "ISSUE_USE_THE_MOST_RECENT_RETAINED_SCHEDULE_J_FOR_EVERY_BASE_YEAR_THAT_IT_1C243B9E",
            "Use the most recent retained Schedule J for every base year that it adjusts, and the original/corrected return for later base years.",
        );
        let (base, old_tax) = b.retained(r);
        let updated = base + share;
        let new_tax = b.tax(updated, r);
        result += new_tax - old_tax;
        emit_lines(
            r,
            &mut (),
            [
                ("base_income", base),
                ("allocated_income", if b.year == 2023 { positive(updated) } else { updated }),
                ("base_tax", old_tax),
                ("recomputed_tax", new_tax),
            ],
            |(suffix, value), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!("farm_averaging_{}_{suffix}", b.year),
                    LineValue::MoneyDollars(value),
                    &["screening.farm_averaging"],
                    SOURCE,
                    msg!("EXPLAIN_FARM_AVERAGING_N_N"),
                ))
            },
        );
    }
    emit_lines(
        r,
        &mut (),
        [
            (
                "farm_averaging_eligible_income",
                phase::cap(positive(current.eligible), [current.taxable]),
            ),
            ("farm_averaging_elected_income", elected),
            ("farm_averaging_each_year", share),
            ("farm_averaging_remaining_income", current.taxable - elected),
            ("farm_averaging_current_tax", reduced),
            ("farm_averaging_without_election", current.regular_tax),
            ("farm_averaging_tax", positive(result)),
        ],
        |(id, value), _, _| {
            Some(calculation_row(
                None,
                id,
                LineValue::MoneyDollars(value),
                &["screening.farm_averaging", "income.business"],
                SOURCE,
                msg!("EXPLAIN_FARM_AVERAGING"),
            ))
        },
    );
    if before != r.issues.len() {
        return Calculation::unchanged(current.regular_tax);
    }
    if result > current.regular_tax {
        r.review_notes.push(crate::federal_model::ReviewNote::localized(#[cfg(test)] "The selected farm-averaging election increases regular income tax. Reduce the chosen amount or remove the election to use ordinary current-year tax; no alternative is elected automatically.", crate::Message::new("NOTE_THE_SELECTED_FARM_AVERAGING_ELECTION_INCREASES_REGULAR_INCOME_TA_9202791E", alloc::vec![])));
    }
    r.review_notes.push(crate::federal_model::ReviewNote::localized(#[cfg(test)] "Schedule J changes regular income tax only. Retain the displayed base-year amounts for later elections; AGI, self-employment tax, QBI, carryovers and AMT are unchanged. Final 2026 form reconciliation remains required.", crate::Message::new("NOTE_SCHEDULE_J_CHANGES_REGULAR_INCOME_TAX_ONLY_RETAIN_THE_DISPLAYED_29693665", alloc::vec![])));
    Calculation {
        tax: positive(result),
        comparison: Some(Worksheet { elected, base_delta: result - reduced, years: &facts.years }),
    }
}
