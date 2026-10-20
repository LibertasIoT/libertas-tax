// Coverage: Form 1099-QA with Form 5329 Part II (ABLE distributions) — Supported (unit B6a,
//   2026-10-09; the stored ABLE kind reshaped into the per-form document).
//   Lines: every box the recipient needs of the Rev. April 2025 form: 1 gross distribution (it
//   includes amounts the beneficiary meant to roll over and the value of a non-qualifying
//   beneficiary change), 2 earnings, 3 basis, 6 the recipient is not the designated beneficiary;
//   box 4 program-to-program transfers and box 5 account terminated carry no tax and are not read.
//   Publication 907 Table 1 / section 529A(c)(1): earnings times the qualified disability expenses
//   (with the first 60 days of 2027 elected for 2026) over the distribution are excluded; a rollover
//   to an ABLE account within 60 days is removed with its earnings; a payment after the
//   beneficiary's death is not taxed; a timely returned excess contribution (box 6) is taxed to the
//   contributor as its earnings with no 10% tax. Taxable earnings are Schedule 1 line 8q; the 10%
//   additional tax is Form 5329 Part II lines 5-8 per filer (retirement.rs account_additional_tax).
//   Entry: IncomeDocument::Qa1099 (Form 1099-QA, importable) on the Other income page; the
//   treatment is Annual (the beneficiary's own withdrawal), Death or Corrective (box 6).
//   Exclusions: a beneficiary change to someone who is not an eligible brother or sister is already
//   in box 1; the Saver's Credit lookback reads these forms.
//   Contributions (Form 5329 Part VIII, the 2026 draft's ABLE excess: the $20,000 limit, the
//   working-beneficiary allowance, 6% of the lesser of the excess and the year-end value, to
//   Schedule 2): IncomeDocument::AbleContributions (AbleContributionsPage, unit B6a) on the same
//   page, one for each beneficiary by owner (ABLE_CONTRIBUTIONS_TWICE), stored on
//   screening.able_contributions. The records-complete confirmation is gone: the form's own
//   figures decide; a spouse's record needs a joint return (ISSUE_ABLE_BENEFICIARY_NOT_ON_RETURN).
//   Tests: src/tests/health/account_documents.rs able_contributions_*, src/tests/family/health_accounts.rs able_excess_*.
//   Tests: src/tests/family/health_accounts.rs (fixtures F11-F13), src/tests/family/education_accounts.rs, src/tests/family/interview.rs
//   qa1099_*. Sources: https://www.irs.gov/pub/irs-pdf/f1099qa.pdf and i1099qa.pdf (Rev. April
//   2025), https://www.irs.gov/pub/irs-pdf/p907.pdf (2025), https://www.irs.gov/pub/irs-dft/f5329--dft.pdf
//   (2026 draft), 26 U.S.C. 529A.
// Native source projection controls: src/tests/attribution/unit13_account_sources.rs (recipient,
// source cents, penalty character, correction coordination and Save/restart).
use crate::kernel::lines::{LineValue, emit as emit_lines, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::{fraction, positive};
use crate::year_constants::ABLE_ANNUAL_CONTRIBUTION_LIMIT_DOLLARS;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// 1099-QA
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read one Form 1099-QA (Distributions From ABLE Accounts) exactly. Its revision has no tax year printed in the title; check the calendar year box against the supplied year and report a mismatch as an error. Use a corrected form in place of the original. Report an illegible amount, or two values that conflict, as an error. Return amounts as decimal numbers without commas or currency signs; return an amount shown in parentheses as a negative number. Never infer a value that is not printed. Read box 1 gross distribution, box 2 earnings and box 3 basis; a blank box 1, 2 or 3 is an error. For treatment return Corrective when box 6 is checked and Annual otherwise. Do not read boxes 4 and 5, the payer or recipient name, address, TIN or account number, and do not report what the money paid for, a rollover, a death or the contribution year; the filer answers those. The file may hold several copies of this form, such as Copy B and Copy C; copies of the same form are one form.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*"
)]
pub struct Qa1099 {
    /// Box 1 gross distribution
    /// Enter only a form addressed to you or your spouse. A form in a dependent's name goes on the dependent's own return.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub gross: i64,
    /// Box 2 earnings
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    pub earnings: i64,
    /// Box 3 basis
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub basis: i64,
    /// What the payment was
    #[libertas_llm_doc_field]
    #[libertas_default(Annual)]
    pub treatment: AbleTreatment,
}

/// What the payment was
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum AbleTreatment {
    /// Paid to you as the account's beneficiary
    /// Box 6 is not checked. A change to a beneficiary who is not your eligible brother or sister is already in box 1.
    Annual {
        /// Put back into an ABLE account within 60 days
        #[libertas_default(None)]
        rollover: AbleRollover,
        /// Qualified disability expenses paid in 2026
        /// Costs related to your disability that maintain or improve your health, independence or quality of life: education, housing, transportation, employment training and support, assistive technology, health, legal and financial services, funeral costs and the like. With two ABLE accounts, count each expense for one account only.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        expenses: i64,
        /// Expenses paid in the first 60 days of 2027 you treat as 2026
        /// Optional. They cannot also count for 2027.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        following_expenses: i64,
    },
    /// Paid after the beneficiary's death
    /// To the estate, an heir, a state Medicaid claim or outstanding disability expenses. Not taxed.
    Death,
    /// Returned excess contribution (box 6)
    /// Paid back to you as the person who contributed, by the beneficiary's return due date.
    Corrective {
        /// Year of the contribution returned
        /// Earnings of a contribution for 2025 belong on your 2025 return.
        // Tax-year structure: the recorded date/year cannot be later than this applicable tax-year window.
        #[libertas_number(min = 2015, max = 2026)]
        year: u32,
    },
}

/// Put back into an ABLE account within 60 days
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum AbleRollover {
    /// None
    None,
    /// Part or all of it
    /// Into your own ABLE account or that of an eligible brother or sister; one rollover into your account in 12 months.
    Qualified {
        /// Amount put back
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        amount: i64,
        /// Earnings in that amount
        /// Use the program's earnings share for the amount rolled over.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        earnings: i64,
    },
}

/// ABLE account distribution — Form 1099-QA
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct AbleDistribution {
    /// Box 1 gross distribution
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub gross: i64,
    /// Box 2 earnings
    #[libertas_money("USD", 2)]
    pub earnings: i64,
    /// What the payment was
    pub treatment: AbleTreatment,
}

// Final annual/correction/death income and additional-tax base, in cents.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct AbleAmounts {
    pub(crate) taxable: i64,
    pub(crate) penalized: i64,
}

impl AbleRollover {
    fn valid(self, gross: i64, total_earnings: i64) -> bool {
        match self {
            Self::None => true,
            Self::Qualified { amount, earnings } => {
                (0..=gross).contains(&amount)
                    && (0..=amount.min(total_earnings.max(0))).contains(&earnings)
                    && total_earnings - earnings <= gross - amount
            }
        }
    }
    fn excluded(self) -> (i64, i64) {
        match self {
            Self::Qualified { amount, earnings } => (amount, earnings),
            Self::None => (0, 0),
        }
    }
}
impl Qa1099 {
    pub(crate) fn record(&self) -> AbleDistribution {
        AbleDistribution {
            gross: self.gross,
            earnings: self.earnings,
            treatment: self.treatment.clone(),
        }
    }
    // Native source validation, shared by Save and preview.
    pub(crate) fn rule(&self) -> Option<&'static str> {
        if self.earnings.checked_add(self.basis) != Some(self.gross) {
            return Some("QA1099_BOXES_DO_NOT_ADD_UP");
        }
        match self.treatment {
            AbleTreatment::Annual {
                rollover: AbleRollover::Qualified { amount, earnings },
                ..
            } => {
                if amount > self.gross {
                    Some("QA1099_ROLLOVER_ABOVE_GROSS")
                } else if earnings > amount {
                    Some("QA1099_ROLLOVER_EARNINGS_ABOVE_AMOUNT")
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    // Field metadata for the existing native predicates, not a second check.
    pub(crate) fn rule_issue(&self, key: &'static str) -> crate::validation::ValidationIssue {
        let issue = crate::validation::ValidationIssue::new(key);
        match key {
            "QA1099_BOXES_DO_NOT_ADD_UP" => issue.at_field("gross"),
            "QA1099_ROLLOVER_ABOVE_GROSS" => {
                issue.at_field("amount").in_field("rollover").in_field("treatment")
            }
            "QA1099_ROLLOVER_EARNINGS_ABOVE_AMOUNT" => {
                issue.at_field("earnings").in_field("rollover").in_field("treatment")
            }
            _ => issue,
        }
    }
}
impl AbleDistribution {
    // The page shows box 3, which the record leaves out: gross less earnings.
    pub(crate) fn page(&self) -> Qa1099 {
        Qa1099 {
            gross: self.gross,
            earnings: self.earnings,
            basis: self.gross - self.earnings,
            treatment: self.treatment.clone(),
        }
    }
    pub(crate) fn valid(&self) -> bool {
        (0..=100000000000).contains(&self.gross)
            && (-100000000000..=self.gross).contains(&self.earnings)
            && match self.treatment {
                AbleTreatment::Annual { expenses, following_expenses, rollover } => {
                    [expenses, following_expenses].iter().all(|v| (0..=100000000000).contains(v))
                        && rollover.valid(self.gross, self.earnings)
                }
                AbleTreatment::Corrective { year } => (2015..=2026).contains(&year),
                AbleTreatment::Death => true,
            }
    }
    // The rollover (amount, earnings) an annual payment put back.
    pub(crate) fn rollover(&self) -> (i64, i64) {
        match self.treatment {
            AbleTreatment::Annual { rollover, .. } => rollover.excluded(),
            _ => (0, 0),
        }
    }
    // Cents. Round the excluded earnings before subtraction, as in Publication
    // 907 Table 1; annual tax lines are rounded by the caller. A loss is not
    // deductible, so negative earnings tax nothing.
    pub(crate) fn amounts(&self) -> AbleAmounts {
        let taxable = match self.treatment {
            AbleTreatment::Annual { expenses, following_expenses, .. } => {
                let (rolled, rolled_earnings) = self.rollover();
                let gross = self.gross - rolled;
                let earnings = self.earnings - rolled_earnings;
                if earnings <= 0 {
                    0
                } else {
                    let qualified = expenses + following_expenses;
                    earnings - fraction(earnings, phase::cap(qualified, [gross]), gross.max(1))
                }
            }
            AbleTreatment::Corrective { year } => {
                if year == 2026 {
                    positive(self.earnings)
                } else {
                    0
                }
            }
            AbleTreatment::Death => 0,
        };
        AbleAmounts { taxable, penalized: if self.penalized() { taxable } else { 0 } }
    }
    pub(crate) fn taxable(&self) -> i64 {
        self.amounts().taxable
    }
    // The 10% additional tax applies to the taxable annual withdrawals, never to
    // post-death payments or corrections.
    pub(crate) fn penalized(&self) -> bool {
        matches!(self.treatment, AbleTreatment::Annual { .. })
    }
}

/// ABLE contributions
/// Contributions to the ABLE accounts of one beneficiary on this return for 2026, to figure the 6% tax on contributions above the limit (Form 5329 Part VIII). One for each beneficiary; choose whose accounts they are by the owner. Contributions do not reduce taxable income, and qualified ABLE-to-ABLE transfers are not counted.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct AbleContributionsPage {
    /// Beneficiary's own cash contributions for 2026
    /// Exclude timely returned principal; retain program records. Do not subtract associated earnings from contributions. Do not include a section 529 tuition-plan rollover here.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub own: i64,
    /// Other contributions and section 529 tuition-plan rollovers for 2026
    /// Include contributions from every other person, and all tuition-plan rollovers regardless of contributor. Exclude timely returned principal, qualified ABLE-to-ABLE rollovers or transfers, and full direct Trump-account-to-ABLE rollovers for a 17-year-old beneficiary. Other contributors cannot use the beneficiary's working-person allowance.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub other: i64,
    /// Working-beneficiary additional allowance
    #[libertas_default(No)]
    pub work: AbleWork,
    /// Account value on December 31, 2026
    /// The 6% tax cannot exceed 6% of this value. Sum year-end values of every 2026 ABLE account for this beneficiary, including an additional account still open; do not enter just current contributions.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub year_end: i64,
    /// Additional 2026 account still open
    /// Select Yes when this beneficiary has a second ABLE account that has not yet been fully corrected or transferred into the preexisting account. Include both accounts' 2026 contributions and year-end values once; enter each account's distributions on its own Form 1099-QA. The additional account remains an ABLE account through 2026; if neither complete correction nor consolidation occurs by the beneficiary's return deadline, including extensions, it ceases to be an ABLE account immediately after that deadline. This later change is not a 2026 deemed distribution.
    #[libertas_default(false)]
    pub additional_open: bool,
}

/// ABLE contributions for one designated beneficiary
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct AbleContributions {
    /// Account beneficiary on this return
    pub owner: crate::Owner,
    /// Beneficiary's own cash contributions for 2026
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub own: i64,
    /// Other contributions and section 529 tuition-plan rollovers for 2026
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub other: i64,
    /// Working-beneficiary additional allowance
    pub work: AbleWork,
    /// Account value on December 31, 2026
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub year_end: i64,
    /// Additional 2026 account still open
    #[libertas_default(false)]
    pub additional_open: bool,
}
impl AbleContributionsPage {
    pub(crate) fn record(&self, owner: crate::Owner) -> AbleContributions {
        AbleContributions {
            owner,
            own: self.own,
            other: self.other,
            work: self.work.clone(),
            year_end: self.year_end,
            additional_open: self.additional_open,
        }
    }
}
impl AbleContributions {
    pub(crate) fn page(&self) -> AbleContributionsPage {
        AbleContributionsPage {
            own: self.own,
            other: self.other,
            work: self.work.clone(),
            year_end: self.year_end,
            additional_open: self.additional_open,
        }
    }
}

/// Working-beneficiary allowance
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum AbleWork {
    /// No additional working-beneficiary allowance
    No,
    /// Employed or self-employed beneficiary
    Working {
        /// Beneficiary's compensation for 2026 under section 219(f)(1)
        /// Use qualifying compensation records, including applicable self-employment adjustments; do not include investment income, pensions or a spouse's compensation.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        compensation: i64,
        /// State of residence for the poverty guideline
        residence: crate::MarketplaceResidence,
        /// No contribution to a disqualifying retirement plan for this beneficiary for 2026
        /// This includes contributions by anyone to a defined-contribution plan under section 401(a) or 403(a), a 403(b) annuity or a governmental 457(b) plan. Check the beneficiary's Form W-2 box 13 "Retirement plan" and plan statements. No removes the additional allowance.
        no_plan: bool,
    },
}
impl AbleContributions {
    pub(crate) fn valid(&self) -> bool {
        [self.own, self.other, self.year_end].iter().all(|v| (0..=100000000000).contains(v))
            && match self.work {
                AbleWork::No => true,
                AbleWork::Working { compensation, .. } => {
                    (0..=100000000000).contains(&compensation)
                }
            }
    }
}
pub(crate) fn valid_contributions(values: &[AbleContributions]) -> bool {
    values
        .iter()
        .enumerate()
        .all(|(i, v)| v.valid() && !values[..i].iter().any(|old| old.owner == v.owner))
}
pub(crate) fn excess_tax(
    values: &[AbleContributions],
    status: crate::FederalStatus,
    r: &mut crate::FederalResult,
) -> [i64; 2] {
    use crate::kernel::ratio::{dollars, positive};
    let mut tax = [0; 2];
    for v in values {
        if v.additional_open {
            r.review_notes.push(crate::federal_model::ReviewNote::localized(#[cfg(test)] "Additional ABLE account: 2026 contributions, account values and distributions must include both accounts once. Unless the additional account is fully corrected or consolidated by the beneficiary's return deadline, including extensions, it ceases to be an ABLE account immediately after that deadline; review the following year separately.", crate::Message::new("NOTE_ADDITIONAL_ABLE_ACCOUNT_RETURN_YEAR_CONTRIBUTIONS_ACCOUNT_VALUES_AND_DI_24653906", alloc::vec![])));
        }
        crate::require_localized!(
            r,
            v.owner == crate::Owner::Taxpayer || status == crate::FederalStatus::Joint,
            &crate::IssueContext::owner(crate::TaxSection::OtherIncome, v.owner),
            "ABLE contributions",
            "ISSUE_ABLE_BENEFICIARY_NOT_ON_RETURN",
            "An ABLE beneficiary other than you is on this return only when you file jointly.",
        );
        let total = dollars(v.own + v.other);
        let additional = match v.work {
            AbleWork::No => 0,
            AbleWork::Working { compensation, residence, no_plan } => {
                let potential = phase::cap(
                    phase::cap(dollars(v.own), [dollars(compensation)]),
                    [crate::marketplace::poverty(residence, 1)],
                );
                if no_plan { potential } else { 0 }
            }
        };
        let excess = positive(total - ABLE_ANNUAL_CONTRIBUTION_LIMIT_DOLLARS - additional);
        let charge = fraction(phase::cap(excess, [dollars(v.year_end)]), 6, 100);
        tax[usize::from(v.owner == crate::Owner::Spouse)] += charge;
        emit_lines(
            r,
            &mut (),
            [
                ("contribution_limit", ABLE_ANNUAL_CONTRIBUTION_LIMIT_DOLLARS + additional),
                ("excess_contributions", excess),
                ("excess_tax", charge),
            ],
            |(suffix, amount), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!(
                        "able_{}_{}",
                        if v.owner == crate::Owner::Taxpayer { "taxpayer" } else { "spouse" },
                        suffix
                    ),
                    LineValue::MoneyDollars(amount),
                    &["screening.able_contributions"],
                    "https://www.irs.gov/instructions/i5329",
                    msg!("EXPLAIN_ABLE_N_N"),
                ))
            },
        );
    }
    tax
}
