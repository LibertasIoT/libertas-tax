// Coverage: Form5329 PRINT PartIX (2026-10-13): per-filer lines52a–55
//   reuse actual minimums, eligible timely amounts, waiver requests and the
//   existing10%/25% buckets. Other plans' excess distributions cannot offset
//   a shortfall. Waivers remain requested, never represented as IRS approval.
//   Evidence: retirement.rs RMD controls and corpus footings; IRS2026 draft
//   f5329 and i5329 lines53/54 (per-plan cap and RC waiver presentation).
// Form 5329 Part IX, using calculated or administrator-supplied requirements.
use crate::federal_rules::valid_date;
use crate::kernel::lines::{LineValue, emit as emit_lines, row as calculation_row};
use crate::kernel::lines::{line, line_cents};
use crate::kernel::phase;
use crate::kernel::ratio::{dollars, fraction};
use crate::{FederalDraft, FederalResult, Owner};
use alloc::string::String;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Required-distribution account group
/// One entry per plan or legally aggregatable group. Never offset one group's shortfall with another group's excess. Choose an administrator's required amount or calculate from own IRA balances and beneficiary facts.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RequiredDistribution {
    /// Person required to withdraw
    pub owner: Owner,
    /// Plan or inherited-owner label
    /// Use a descriptive synthetic label, not an account number or tax ID.
    pub label: String,
    /// Required-minimum source
    pub required: crate::RmdMinimum,
    /// Amount satisfying these requirements by their deadlines
    /// Include eligible qualified charitable distributions. Exclude corrective payments after the deadline, rollovers and unrelated accounts. If two deadlines fall in 2026, do not count a payment made after the first deadline against its missed requirement or count a payment twice.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub timely: i64,
    /// Later correction and additional-tax return
    pub correction: RmdCorrection,
    /// Reasonable-cause waiver request, if any
    /// IRS approval is not automatic. The requested amount is removed from the reported shortfall pending IRS review; retain and file the explanation with Form 5329.
    pub waiver: Option<RmdWaiver>,
}

// The Retirement topic's kind (unit u5-adjust): one account group as the
// page asks it; the page item's owner completes the stored record.
/// Missed required minimum distribution
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MissedDistribution {
    /// Plan or IRA group
    /// A short name, such as "Traditional IRAs" or "Employer 401(k)". Do not enter an account number. Own traditional IRAs may be grouped; inherited IRAs only with the same deceased owner; each employer plan is separate.
    #[libertas_size(min = 1)]
    pub label: String,
    /// Required minimum for 2026
    pub required: crate::RmdMinimum,
    /// Taken by the deadline
    /// Include qualified charitable distributions. Leave out payments made after the deadline and rollovers.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub timely: i64,
    /// Correction
    pub correction: RmdCorrection,
    /// Waiver requested for reasonable cause
    /// Leave empty for no waiver request. The IRS decides; the estimate removes the amount pending its review. IRS approval is not automatic: retain and file the explanation with Form 5329.
    /// [NullText]
    /// Do not request a reasonable-cause waiver
    /// [SetText]
    /// I request a reasonable-cause waiver for
    #[libertas_default($null)]
    pub waiver: Option<RmdWaiver>,
}
impl MissedDistribution {
    pub(crate) fn record(&self, owner: Owner) -> RequiredDistribution {
        RequiredDistribution {
            owner,
            label: self.label.clone(),
            required: self.required.clone(),
            timely: self.timely,
            correction: self.correction.clone(),
            waiver: self.waiver.clone(),
        }
    }
    pub(crate) fn from_record(v: &RequiredDistribution) -> Self {
        Self {
            label: v.label.clone(),
            required: v.required.clone(),
            timely: v.timely,
            correction: v.correction.clone(),
            waiver: v.waiver.clone(),
        }
    }
}

/// Correction records
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RmdCorrection {
    /// Full correction or a dated filing commitment is missing
    Uncorrected,
    /// Full correction received; return filed or planned
    Corrected {
        /// Date the full shortfall was withdrawn from the responsible plan or eligible account group
        /// Partial correction does not qualify this group for the reduced rate. Do not split an uncorrected plan to obtain 10% treatment on only its corrected portion.
        #[libertas_date_only]
        received: u32,
        /// Return reflecting this additional tax
        filed: RmdFiling,
        /// Deficiency-notice mailing date, if any
        /// [NullText]
        /// No deficiency notice has been mailed for this tax
        /// [SetText]
        /// The deficiency notice was mailed on
        #[libertas_date_only]
        notice: Option<u32>,
        /// Assessment date, if any
        /// [NullText]
        /// This additional tax has not been assessed
        /// [SetText]
        /// This additional tax was assessed on
        #[libertas_date_only]
        assessed: Option<u32>,
    },
}

/// Additional-tax return submission
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RmdFiling {
    /// Already submitted
    Submitted {
        /// Actual submission date
        #[libertas_date_only]
        date: u32,
    },
    /// Prepare an estimate for a planned submission
    Planned {
        /// Intended submission date
        /// The reduced-rate estimate depends on actually submitting the return reflecting this tax by the correction-window deadline. This prototype does not file it. Update the calculation if filing is delayed or an IRS notice or assessment arrives first.
        #[libertas_date_only]
        date: u32,
    },
}
impl RmdFiling {
    fn date(&self) -> u32 {
        match self {
            Self::Submitted { date } | Self::Planned { date } => *date,
        }
    }
}

/// Requested reasonable-cause relief
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RmdWaiver {
    /// Shortfall amount for which relief is requested
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
    pub amount: i64,
    /// Explanation of reasonable error and corrective steps
    pub explanation: String,
}
impl RequiredDistribution {
    pub(crate) fn valid(&self) -> bool {
        self.required.valid()
            && (0..=100000000000).contains(&self.timely)
            && self.waiver.as_ref().is_none_or(|v| (1..=100000000000).contains(&v.amount))
            && match &self.correction {
                RmdCorrection::Uncorrected => true,
                RmdCorrection::Corrected { received, filed, notice, assessed } => {
                    [Some(*received), Some(filed.date()), *notice, *assessed]
                        .into_iter()
                        .flatten()
                        .all(|v| valid_date(v) && v >= 20260101)
                }
            }
    }
}
// Completed native per-owner taxes; omitted zero form rows remain actual zeros.
#[derive(Default)]
pub(crate) struct RmdTaxes {
    pub total: i64,
    pub owners: [i64; 2],
}
pub(crate) fn calculate(d: &FederalDraft, r: &mut FederalResult) -> RmdTaxes {
    let Some(a) = &d.adjustments else {
        return RmdTaxes::default();
    };
    let mut buckets = [[0_i64; 2]; 2];
    // Owner, correction-rate class, then required/timely/waiver source cents.
    let mut form = [[[0_i64; 3]; 2]; 2];
    let mut required = [false; 2];
    for (index, entry) in a.required_distributions.iter().enumerate() {
        let source = crate::IssueContext::owner(crate::TaxSection::Retirement, entry.owner);
        let context = crate::IssueContext::Record {
            section: crate::TaxSection::Retirement,
            identity: source.item("required_distributions", index).identity(),
            label: if entry.label.trim().is_empty() {
                source.label()
            } else {
                crate::Message::new(
                    "NOTE_WITH_LABEL",
                    alloc::vec![
                        crate::MessageArg::Nested(alloc::boxed::Box::new(source.label())),
                        crate::MessageArg::Text(entry.label.clone()),
                    ],
                )
            },
        };
        crate::require_localized!(
            r,
            !entry.label.trim().is_empty()
                && (entry.owner == Owner::Taxpayer
                    || (d
                        .setup
                        .as_ref()
                        .is_some_and(|s| s.filing_status() == crate::FederalStatus::Joint)
                        && d.people.as_ref().is_some_and(|p| p.spouse.is_some()))),
            &context,
            "Required distributions",
            "ISSUE_CONFIRM_THE_SOURCE_MINIMUM_TIMELY_PAYMENTS_LEGAL_ACCOUNT_GROUP_AND_2AAF74CB",
            "Enter a name for the group and check that the filer is on this return. Unresolved deadlines or source calculations require review.",
        );
        let minimum = entry.required.calculate(d, r, entry.owner, index, entry.timely);
        let shortfall = phase::floor(minimum - entry.timely, 0);
        let reduced = match &entry.correction {
            RmdCorrection::Uncorrected => false,
            RmdCorrection::Corrected { received, filed, notice, assessed } => {
                let end = [Some(20281231), *notice, *assessed].into_iter().flatten().min().unwrap();
                let reduced = *received <= end && filed.date() <= end;
                if reduced && matches!(filed, RmdFiling::Planned { .. }) {
                    line_cents(
                        r,
                        &alloc::format!("rmd_{index}_conditional_reduced_shortfall"),
                        shortfall,
                        &["adjustments.required_distributions"],
                        "https://www.irs.gov/instructions/i5329",
                        msg!("EXPLAIN_RMD_N_CONDITIONAL_REDUCED_SHORTFALL"),
                    );
                }
                reduced
            }
        };
        let waived = entry.waiver.as_ref().map_or(0, |v| {
            crate::require_localized!(r, v.amount <= shortfall && !v.explanation.trim().is_empty(),
                &context,
"Required distributions", "ISSUE_A_WAIVER_REQUEST_NEEDS_A_REASONABLE_ERROR_EXPLANATION_AND_REMEDIAL_AC7AD5EB",
"A waiver request needs a reasonable-error explanation and remedial steps; its amount cannot exceed the actual shortfall. IRS approval remains pending.");
            phase::cap(v.amount, [shortfall])
        });
        let owner = usize::from(entry.owner == Owner::Spouse);
        let rate = usize::from(!reduced);
        buckets[owner][rate] += shortfall - waived;
        form[owner][rate][0] += minimum;
        // i5329 line53 excludes excess from a different nonaggregatable plan.
        // The accepted entry already names one legally aggregatable group.
        form[owner][rate][1] += phase::cap(entry.timely, [minimum]);
        form[owner][rate][2] += waived;
        required[owner] |= shortfall > 0;
        emit_lines(
            r,
            &mut (),
            [("shortfall", shortfall), ("waiver_requested", waived)],
            |(suffix, amount), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!("rmd_{index}_{suffix}"),
                    LineValue::MoneyCents(amount),
                    &["adjustments.required_distributions"],
                    "https://www.irs.gov/instructions/i5329",
                    msg!("EXPLAIN_RMD_N_N"),
                ))
            },
        );
    }
    let mut total = 0;
    let mut owners = [0; 2];
    for (owner, buckets) in buckets.iter().enumerate() {
        // Each spouse reports a separate Form 5329, with distinct 10% and 25% lines.
        let shortfall = |rate: usize| {
            crate::forms::form_5329::rmd_shortfall(
                form[owner][rate][0],
                form[owner][rate][1],
                form[owner][rate][2],
            )
        };
        let taxes = [fraction(shortfall(0), 10, 100), fraction(shortfall(1), 25, 100)];
        let tax = taxes[0] + taxes[1];
        total += tax;
        if required[owner] {
            let who = if owner == 0 { "taxpayer" } else { "spouse" };
            emit_lines(
                r,
                &mut (),
                [
                    ("52a", LineValue::MoneyDollars(dollars(form[owner][0][0]))),
                    ("52b", LineValue::MoneyDollars(dollars(form[owner][1][0]))),
                    ("53a", LineValue::MoneyDollars(dollars(form[owner][0][1]))),
                    ("53b", LineValue::MoneyDollars(dollars(form[owner][1][1]))),
                    ("54a", LineValue::MoneyDollars(taxes[0])),
                    ("54b", LineValue::MoneyDollars(taxes[1])),
                    ("55", LineValue::MoneyDollars(tax)),
                ],
                |(number, value), _, _| {
                    Some(calculation_row(
                        Some(crate::FormLine::new(crate::TaxForm::Form5329, number)),
                        &alloc::format!("{who}_rmd_line{number}"),
                        value,
                        &["adjustments.required_distributions"],
                        "https://www.irs.gov/pub/irs-dft/f5329--dft.pdf",
                        msg!("EXPLAIN_RMD_FORM_COMPARISON"),
                    ))
                },
            );
            for (rate, number) in [(0, "54a"), (1, "54b")] {
                let waived = form[owner][rate][2];
                if waived > 0 {
                    emit_lines(
                        r,
                        &mut (),
                        [calculation_row(
                            Some(crate::FormLine::subrow(crate::TaxForm::Form5329, number)),
                            &alloc::format!("{who}_rmd_waiver{number}"),
                            LineValue::MoneyDollars(dollars(waived)),
                            &["adjustments.required_distributions.waiver"],
                            "https://www.irs.gov/instructions/i5329",
                            msg!("EXPLAIN_RMD_FORM_WAIVER_REQUEST"),
                        )],
                        |row, _, _| Some(row),
                    );
                }
            }
        }
        owners[owner] = tax;
        if buckets.iter().any(|v| *v != 0) {
            line(
                r,
                &alloc::format!("rmd_{owner}_tax"),
                tax,
                &["adjustments.required_distributions"],
                "https://www.irs.gov/instructions/i5329",
                msg!("EXPLAIN_RMD_N_TAX"),
            );
        }
    }
    line(
        r,
        "required_distribution_tax",
        total,
        &["adjustments.required_distributions"],
        "https://www.irs.gov/instructions/i5329",
        msg!("EXPLAIN_REQUIRED_DISTRIBUTION_TAX"),
    );
    RmdTaxes { total, owners }
}
