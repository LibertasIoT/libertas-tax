// PRINT projection (2026-10-13): disaster/projection.rs emits2026 draft
// PartI columns2-5c/limits/unused gross, conditional8-26/28-32 and actual
// traditional/Roth/payer21 components. Copies keep filer and disaster year;
// terminal and per-document PartIV components are supporting rows only.
// Financial amounts, source cents, repayments, timing and old source IDs stay.
// Tests: print_form_8915_*, all supported() corpus Form8915 footings.
// Unit6 inherited-IRA correction (2026-10-13): qualified traditional and
// Roth beneficiary payments consume their existing group taxable/designated
// owners and remain in the IRA category. Existing spread and cent rounding
// are unchanged. Tests: inherited_*_disaster_uses_* and combination controls.
// Coverage: Form 8915-F — Supported (unit T3-8915, 2026-09-27; audited Missing 2026-09-24).
//   Calculated, per owner and per disaster year (item B), one 2026 Form 8915-F each:
//   Part I from the 1099-R fold "This was a qualified disaster distribution" (FEMA DR number,
//   incident beginning and declaration dates, distribution date, designated amount, earlier
//   qualified distributions for the same disaster = line 1b, spread election, repayments by the
//   filing date). Line 1e is $22,000 per disaster less line 1b; each designated amount is
//   allocated to its own disaster, which is the instructions' "reasonable method" for lines 2-5b
//   column (b) and makes Worksheet 1B unnecessary. Line 6 (no Form 5329 additional tax: the
//   designated part's taxable amount leaves Form 5329 line 1 in retirement::early_tax), line 7
//   (the undesignated rest keeps its 1099-R treatment). Part II lines 8-15: line 9 cost is the
//   designated share of the entry's non-taxable amount (pro rata to the payer or Simplified Method
//   taxable amount); line 11 one third or all (election); line 12 Worksheet 2 = line 11 of the
//   2024 and 2025 Forms 8915-F for the same disaster year; line 14 Worksheet 3 = line 3c (2025
//   line 14 less line 13, less the part carried back) + line 4 (2026 repayments of 2024/2025
//   distributions and of this year's, taxable share only); line 15 to Form 1040 line 5b.
//   An owner who died in 2026 includes the rest of every spread on the final return (lines 13 and 24:
//   the 2026 distributions' whole taxable amount and the 2027 installment of a 2025 form's spread).
//   Part III lines 16-26 the same for IRAs (lines 18/20 are the designated share of the entry's
//   Form 8606 or payer taxable amount; line 26 to line 4b). Excess repayment (line 14 above line
//   13) is shown for next year or an amended earlier return; the 2027 and 2028 installments are
//   shown.
//   Unit B9-4 (2026-10-10, map retirement-special-codes R11): Part IV lines 28-32 (a hardship
//   distribution or IRA first-home distribution to buy or build a main home, DisasterDistribution::
//   HomePurchase with its own DR number and dates: line 32 = line 28 - cost - the taxable part
//   repaid stays in income and keeps the row's 10% path); the distribution period (Part I table)
//   and the Part IV window are rules on the 1099-R page and its server twin (DISASTER_OUTSIDE_PERIOD,
//   DISASTER_HOME_OUTSIDE_PERIOD, ...), not refusals; the designated part is a share of the entry's
//   taxable amount before any rollover after a gift to charity, so the undesignated rest keeps its
//   rollover, conversion, exception or QCD (DISASTER_AND_ROLLOVER_ABOVE_GROSS), and line 9 cost is
//   the designated share of the nontaxable amount whatever method produced it.
//   Refused by name: a disaster without a FEMA major-disaster (DR) number (item C); designated
//   amounts above line 1e; different spread elections on one form (lines 11/22);
//   a 2023-or-earlier
//   Form 8915-F (its spread has ended; 2026 repayments of those distributions are carried back by
//   amended returns). Ordinary and inherited Roth designated earnings feed line19 from
//   their existing Form8606 owners; qualified-only payments retain their zero taxable share. Forms
//   8915-D and 8915-E (2019/2020 disasters) have no 2026 income or repayment period and are not
//   routed to 2026 by the instructions.
//   Line audit F10 (2026-10-10): a repayment made during 2026 of a 2026 IRA distribution
//   (DisasterRepayment::Yes.in_return_year, of the repaid amount) is in the December 31
//   value and reduces Form 8606 line 6 (repaid_in_return_year here, IraBasis::adjusted_year_end); a 2025
//   Form 8915-F of any disaster year (item B) spreads over 2025-2027, so a filer who died in 2026
//   takes its 2027 remainder on the final return. Tests: tests/cross/line_audit_f10.rs.
//   Evidence: disaster.rs, retirement.rs early_tax, federal_rules.rs (line 4b/5b adjustment),
//   interview.rs (R1099 fold, Form8915 data answer); tests/retirement/disaster.rs. Sources: Form 8915-F
//   2026 draft (Rev. December 2026) and instructions (Rev. December 2025), irs.gov,
//   fetched2026-10-13; the draft adds5c after5a(total)/5b(nonqualified).

mod projection;

use crate::kernel::lines::{LineValue, emit as emit_lines, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::fraction;
use crate::{FederalDraft, FederalIncome, FederalResult, Owner, RetirementKind, RetirementTaxable};
use alloc::collections::BTreeMap;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

// Form 8606 line 6 (i8606, "Certain 2026 retirement plan distribution repayments treated as
// rollovers"): the part of this owner's 2026 IRA qualified disaster distributions that was
// repaid during 2026 is already in the December 31 value and comes off it.
pub(crate) fn repaid_in_return_year(d: &FederalDraft, owner: Owner) -> i64 {
    d.income
        .iter()
        .filter(|e| e.owner == owner)
        .filter_map(|e| match &e.income {
            FederalIncome::Retirement { data } if data.kind == RetirementKind::Ira => {
                match data.disaster {
                    DisasterDistribution::Qualified { ref repaid, .. } => {
                        Some(repaid.in_return_year())
                    }
                    _ => None,
                }
            }
            _ => None,
        })
        .sum()
}

// Per-disaster limit for 2021 and later disasters (Form 8915-F line 1a): owned by year_constants.rs.
pub(crate) const LIMIT: i64 = crate::year_constants::DISASTER_DISTRIBUTION_LIMIT_CENTS;

/// This was a qualified disaster distribution
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum DisasterDistribution {
    /// No
    No,
    /// Yes: my main home was in a FEMA major disaster area and the disaster caused me an economic loss
    /// Loss of or damage to property, displacement from your home, or lost work. No 10% additional tax applies to the part you treat as a qualified disaster distribution.
    Qualified {
        /// FEMA disaster number
        /// The digits of the DR number at fema.gov/disaster/declarations, for example 4834 for DR-4834-FL.
        #[libertas_number(min = 1)]
        fema: u32,
        /// Disaster beginning date
        /// The first day of the incident period.
        #[libertas_date_only]
        // Tax structure: a current-return distribution or its source event cannot be after December 31, 2026.
        #[libertas_number(min = 20210126, max = 20261231)]
        began: u32,
        /// Disaster declaration date
        #[libertas_date_only]
        // Tax structure: a current-return distribution or its source event cannot be after December 31, 2026.
        #[libertas_number(min = 20200101, max = 20261231)]
        declared: u32,
        /// Date of this distribution
        #[libertas_date_only]
        // Tax structure: a current-return distribution or its source event cannot be after December 31, 2026.
        #[libertas_number(min = 20260101, max = 20261231)]
        distributed: u32,
        /// Amount you treat as a qualified disaster distribution
        /// Up to box 1. For one disaster, all your plans and years together allow at most $22,000.
        #[libertas_money("USD", 2)]
        // IRC 72(t)(11): qualified disaster distributions and their repayments share the $22,000 limit.
        #[libertas_number(min = 1, max = $DISASTER_DISTRIBUTION_LIMIT_CENTS)]
        amount: i64,
        /// Qualified disaster distributions for this disaster in earlier years
        /// Form 8915-F line 1b. Enter the same amount on each 1099-R for this disaster.
        #[libertas_money("USD", 2)]
        // IRC 72(t)(11): qualified disaster distributions and their repayments share the $22,000 limit.
        #[libertas_number(min = 0, max = $DISASTER_DISTRIBUTION_LIMIT_CENTS)]
        #[libertas_default(0)]
        earlier: i64,
        /// When to include the taxable part
        #[libertas_default(OverThreeYears)]
        spread: DisasterSpread,
        /// Repaid to a retirement plan or IRA by the time you file
        /// Within 3 years of the distribution. Not a required minimum distribution or a beneficiary's distribution.
        #[libertas_default(No)]
        repaid: DisasterRepayment,
    },
    /// Yes, to buy or build a main home in a disaster area
    /// A hardship distribution from a 401(k) or 403(b) plan, or an IRA first-home distribution, for a main home in the disaster area that was not bought or built because of the disaster (Form 8915-F Part IV).
    HomePurchase {
        /// FEMA disaster number
        /// The digits of the DR number at fema.gov/disaster/declarations, for example 4834 for DR-4834-FL.
        #[libertas_number(min = 1)]
        fema: u32,
        /// Disaster beginning date
        /// The first day of the incident period.
        #[libertas_date_only]
        // Form 8915-F home-purchase relief: the incident and declaration window includes 2027.
        #[libertas_number(min = 20210126, max = 20271231)]
        began: u32,
        /// Disaster ending date
        /// The last day of the incident period.
        #[libertas_date_only]
        // Form 8915-F home-purchase relief: the incident and declaration window includes 2027.
        #[libertas_number(min = 20210126, max = 20271231)]
        ended: u32,
        /// Disaster declaration date
        #[libertas_date_only]
        // Form 8915-F home-purchase relief: the incident and declaration window includes 2027.
        #[libertas_number(min = 20210126, max = 20271231)]
        declared: u32,
        /// Date of this distribution
        #[libertas_date_only]
        // Tax structure: a current-return distribution or its source event cannot be after December 31, 2026.
        #[libertas_number(min = 20260101, max = 20261231)]
        distributed: u32,
        /// Taxable part repaid to a retirement plan or IRA
        /// Repaid by 180 days after the later of the beginning and declaration dates, and before you file. Repayments after you file need an amended return. Do not include a repayment of an after-tax amount.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        taxable_repaid: i64,
    },
    /// Yes, but the disaster has no FEMA DR number or I am not sure
    NotDeclared,
}

/// Repaid to a retirement plan or IRA by the time you file
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum DisasterRepayment {
    /// Nothing repaid
    No,
    /// Repaid
    Yes {
        /// Total repaid by the time you file
        #[libertas_money("USD", 2)]
        // IRC 72(t)(11): qualified disaster distributions and their repayments share the $22,000 limit.
        #[libertas_number(min = 1, max = $DISASTER_DISTRIBUTION_LIMIT_CENTS)]
        total: i64,
        /// Of that, repaid during 2026
        /// Only the deposits made on or before December 31, 2026. For an IRA distribution this amount is already in your December 31 IRA value, so Form 8606 line 6 subtracts it.
        #[libertas_money("USD", 2)]
        // IRC 72(t)(11): qualified disaster distributions and their repayments share the $22,000 limit.
        #[libertas_number(min = 0, max = $DISASTER_DISTRIBUTION_LIMIT_CENTS)]
        #[libertas_default(0)]
        in_return_year: i64,
    },
}
impl DisasterRepayment {
    pub(crate) fn total(&self) -> i64 {
        match self {
            Self::No => 0,
            Self::Yes { total, .. } => *total,
        }
    }
    pub(crate) fn in_return_year(&self) -> i64 {
        match self {
            Self::No => 0,
            Self::Yes { in_return_year, .. } => *in_return_year,
        }
    }
}

/// When to include the taxable part
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum DisasterSpread {
    /// Over 2026, 2027 and 2028
    OverThreeYears,
    /// All in 2026
    AllInReturnYear,
}

/// Earlier Form 8915-F
/// A 2024 or 2025 Form 8915-F whose spread income or repayments continue in 2026.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EarlierForm8915 {
    /// Tax year of that form (item A)
    pub year: Form8915Year,
    /// Year the disasters began (item B)
    // Form 8915-F: the retained disaster-year worksheet precedes the 2026 tax year.
    #[libertas_number(min = 2021, max = 2025)]
    pub disaster_year: u32,
    /// Line 11
    /// Employer-plan income spread over 3 years. Enter 0 if you checked the box on line 11.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub line_11: i64,
    /// Line 22
    /// IRA income spread over 3 years. Enter 0 if you checked the box on line 22.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub line_22: i64,
    /// Repayments after you filed that form
    pub repayments: Form8915Repayments,
}

/// Tax year of that form (item A)
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum Form8915Year {
    // Actual historical form year and line layout, not a prior-year alias.
    /// 2025
    Filed2025 {
        /// Line 13
        /// Leave lines 13, 14, 24 and 25 at zero when lines 14 and 25 are blank.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        line_13: i64,
        /// Line 14
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        line_14: i64,
        /// Line 24
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        line_24: i64,
        /// Line 25
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        line_25: i64,
    },
    // Actual historical form year, not a second-prior-year alias.
    /// 2024
    Filed2024,
    /// 2023 or earlier
    Earlier,
}

/// Repayments after you filed that form
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct Form8915Repayments {
    /// Repaid to an employer plan by the time you file for 2026
    /// Of the distributions that form reported, within 3 years of each. Only the taxable part.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub plan_repaid: i64,
    /// Repaid to an IRA by the time you file for 2026
    /// Of the distributions that form reported, within 3 years of each. Only the taxable part.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub ira_repaid: i64,
    /// Line 14 excess you already carried back on an amended return
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub plan_carried_back: i64,
    /// Line 25 excess you already carried back on an amended return
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub ira_carried_back: i64,
}

/// Earlier Form 8915-F of one filer
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct DisasterCarry {
    /// Whose form
    pub owner: Owner,
    /// Form amounts
    pub form: EarlierForm8915,
}

impl DisasterDistribution {
    pub(crate) fn valid(&self, gross: i64) -> bool {
        match self {
            Self::Qualified { amount, earlier, repaid, .. } => {
                (1..=gross.min(LIMIT)).contains(amount)
                    && (0..=LIMIT - amount).contains(earlier)
                    && (0..=*amount).contains(&repaid.total())
                    && (0..=repaid.total()).contains(&repaid.in_return_year())
                    && (matches!(repaid, DisasterRepayment::No) || repaid.total() > 0)
            }
            Self::HomePurchase { began, ended, declared, distributed, taxable_repaid, .. } => {
                [*began, *ended, *declared].iter().all(|d| {
                    (20210126..=20271231).contains(d) && crate::federal_rules::valid_date(*d)
                }) && (20260101..=20261231).contains(distributed)
                    && crate::federal_rules::valid_date(*distributed)
                    && (0..=100000000000).contains(taxable_repaid)
            }
            _ => true,
        }
    }
    // The client rules of the 1099-R page about the qualified disaster distribution period,
    // the home purchase window and what else happened to the money, checked again on save
    // (the keys are the client rules' keys), first failure first.
    pub(crate) fn code_issue(
        &self,
        v: &crate::R1099,
    ) -> Option<crate::validation::ValidationIssue> {
        use crate::dates::add_days;
        let plain = matches!(
            v.code,
            crate::DistributionCode::Normal { .. } | crate::DistributionCode::Early { .. }
        );
        let roth_rolled = match v.code {
            crate::DistributionCode::RothIra { rolled_over, .. } => Some(rolled_over),
            _ => None,
        };
        match self {
            Self::Qualified { began, declared, distributed, amount, .. } => {
                // Form 8915-F Part I: from the incident's beginning through 179 days after
                // the later of the beginning and declaration dates.
                if !(*distributed >= *began
                    && (*distributed <= add_days(*began, 179)
                        || *distributed <= add_days(*declared, 179)))
                {
                    return Some(
                        crate::validation::ValidationIssue::new("DISASTER_OUTSIDE_PERIOD")
                            .at_field("distributed")
                            .in_field("disaster"),
                    );
                }
                if matches!(v.code, crate::DistributionCode::DirectRollover { .. }) {
                    return Some(
                        crate::validation::ValidationIssue::new("DISASTER_DIRECT_ROLLOVER")
                            .at_field("code"),
                    );
                }
                if let Some(rolled) = roth_rolled
                    && amount + rolled > v.gross
                {
                    return Some(
                        crate::validation::ValidationIssue::new(
                            "DISASTER_AND_ROLLOVER_ABOVE_GROSS",
                        )
                        .at_field("amount")
                        .in_field("disaster"),
                    );
                }
                if plain {
                    let other = match crate::retirement::disposition_of(&v.code) {
                        Some(
                            crate::Disposition::RolledOver { amount, .. }
                            | crate::Disposition::Converted { amount }
                            | crate::Disposition::SoldAndRolledOver { amount, .. },
                        ) => *amount,
                        _ => 0,
                    };
                    if amount + other > v.gross {
                        return Some(
                            crate::validation::ValidationIssue::new(
                                "DISASTER_AND_ROLLOVER_ABOVE_GROSS",
                            )
                            .at_field("amount")
                            .in_field("disaster"),
                        );
                    }
                }
                None
            }
            Self::HomePurchase { began, ended, distributed, taxable_repaid, .. } => {
                // Form 8915-F Part IV: received no more than 180 days before the first day and
                // no later than 30 days after the last day of the disaster.
                if !(*distributed >= add_days(*began, -180) && *distributed <= add_days(*ended, 30))
                {
                    Some(
                        crate::validation::ValidationIssue::new("DISASTER_HOME_OUTSIDE_PERIOD")
                            .at_field("distributed")
                            .in_field("disaster"),
                    )
                } else if *ended < *began {
                    Some(
                        crate::validation::ValidationIssue::new("DISASTER_ENDED_BEFORE_BEGAN")
                            .at_field("ended")
                            .in_field("disaster"),
                    )
                } else if *taxable_repaid > v.taxable {
                    Some(
                        crate::validation::ValidationIssue::new(
                            "DISASTER_HOME_REPAID_ABOVE_TAXABLE",
                        )
                        .at_field("taxable_repaid")
                        .in_field("disaster"),
                    )
                } else if !plain {
                    Some(
                        crate::validation::ValidationIssue::new("DISASTER_HOME_KEPT")
                            .at_field("code"),
                    )
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    // The client rules of the 1099-R page, checked again on save.
    pub(crate) fn issue(&self, gross: i64) -> Option<crate::validation::ValidationIssue> {
        match self {
            Self::Qualified { amount, .. } if *amount > gross => Some(
                crate::validation::ValidationIssue::new("R_DISASTER_ABOVE_GROSS")
                    .at_field("amount"),
            ),
            Self::Qualified { amount, earlier, .. } if amount + earlier > LIMIT => Some(
                crate::validation::ValidationIssue::new("DISASTER_ABOVE_LIMIT").at_field("earlier"),
            ),
            Self::Qualified { amount, repaid, .. } if repaid.total() > *amount => Some(
                crate::validation::ValidationIssue::new("DISASTER_REPAID_ABOVE_AMOUNT")
                    .at_field("total")
                    .in_field("repaid"),
            ),
            Self::Qualified { repaid, .. } if repaid.in_return_year() > repaid.total() => Some(
                crate::validation::ValidationIssue::new("DISASTER_REPAID_2026_ABOVE_REPAID")
                    .at_field("in_return_year")
                    .in_field("repaid"),
            ),
            _ => None,
        }
    }
}

impl EarlierForm8915 {
    // The client rules of the kind, checked again on save.
    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        if matches!(self.year, Form8915Year::Filed2024) && self.disaster_year > 2024 {
            return Some(
                crate::validation::ValidationIssue::new("FORM_8915_DISASTER_AFTER_FORM")
                    .at_field("disaster_year"),
            );
        }
        let (plan_back, ira_back) =
            (self.repayments.plan_carried_back, self.repayments.ira_carried_back);
        let (plan_excess, ira_excess) = self.excess_2025();
        if plan_back != 0 && plan_back > plan_excess {
            Some(
                crate::validation::ValidationIssue::new("FORM_8915_CARRIED_BACK_ABOVE_EXCESS")
                    .at_field("plan_carried_back")
                    .in_field("repayments"),
            )
        } else if ira_back != 0 && ira_back > ira_excess {
            Some(
                crate::validation::ValidationIssue::new("FORM_8915_CARRIED_BACK_ABOVE_EXCESS")
                    .at_field("ira_carried_back")
                    .in_field("repayments"),
            )
        } else {
            None
        }
    }
    // Worksheets 3 and 5 line 3a: last year's line 14 (25) over line 13 (24).
    // A carried-back amount is allowed only against a 2025 form's excess.
    fn excess_2025(&self) -> (i64, i64) {
        match &self.year {
            Form8915Year::Filed2025 { line_13, line_14, line_24, line_25 } => {
                (phase::floor(line_14 - line_13, 0), phase::floor(line_25 - line_24, 0))
            }
            _ => (0, 0),
        }
    }
    fn year(&self) -> Option<u32> {
        match self.year {
            Form8915Year::Filed2025 { .. } => Some(2025),
            Form8915Year::Filed2024 => Some(2024),
            Form8915Year::Earlier => None,
        }
    }
    fn repaid(&self) -> (i64, i64, i64, i64) {
        let Form8915Repayments { plan_repaid, ira_repaid, plan_carried_back, ira_carried_back } =
            self.repayments;
        (plan_repaid, ira_repaid, plan_carried_back, ira_carried_back)
    }
    pub(crate) fn valid(&self) -> bool {
        let (a, b, c, d) = self.repaid();
        let lines = match &self.year {
            Form8915Year::Filed2025 { line_13, line_14, line_24, line_25 } => {
                [*line_13, *line_14, *line_24, *line_25]
            }
            _ => [0; 4],
        };
        (2021..=2025).contains(&self.disaster_year)
            && [self.line_11, self.line_22, a, b, c, d]
                .iter()
                .chain(lines.iter())
                .all(|v| (0..=100000000000).contains(v))
            && self.rule().is_none()
    }
}

// The incident beginning date of a designated entry, or the refusal that a disaster
// with no FEMA major-disaster (DR) number has no qualified disaster distributions
// (item C). The distribution period, the money rolled over or converted, an exception
// on the rest, a nonqualified annuity and a gift to charity out of the same 1099-R are
// page rules and ordinary treatments, not refusals: only the designated part spreads.
fn entry_refusal(v: &crate::FederalRetirement) -> Result<u32, (&'static str, &'static str)> {
    match v.disaster {
        DisasterDistribution::Qualified { began, .. } => Ok(began),
        _ => Err((
            "ISSUE_8915_MISSING_FEMA_DECLARATION",
            crate::fixture_english!(
                "Form 8915-F item C: only a disaster with a FEMA major disaster declaration (a DR number) can have qualified disaster distributions. Choose No, or enter the DR number."
            ),
        )),
    }
}

// The taxable part of a main home distribution that was repaid (Form 8915-F line 31), which
// leaves Form 5329 line 1 with the rest of line 32 kept there.
pub(crate) fn home_repaid(v: &crate::FederalRetirement, taxable: i64) -> i64 {
    match v.disaster {
        DisasterDistribution::HomePurchase { taxable_repaid, .. } => {
            phase::floor(phase::cap(taxable_repaid, [taxable]), 0)
        }
        _ => 0,
    }
}

// The entry's taxable amount before any rollover, less the gift to charity (Form 8915-F
// line 9's base): the amount the designated part is a share of.
fn taxable_before_rollover(
    e: &crate::FederalIncomeEntry,
    v: &crate::FederalRetirement,
    basis: &crate::retirement::BasisResult,
    qcd: i64,
) -> i64 {
    // A linked inherited payment's group supplies its share separately. With
    // no group, preserve the existing payer-amount fallback used by collect.
    let source_taxable = match v.taxable {
        RetirementTaxable::Inherited { payer, .. } => payer,
        _ => basis.taxable(e.id, v),
    };
    let after = phase::floor(source_taxable - qcd, 0);
    if v.taxable == RetirementTaxable::IraBasis {
        // The Form 8606 ratio applies to every dollar that was not converted or given away.
        let funding = basis.hsa_funding.get(&e.id).copied().unwrap_or(0);
        let kept =
            phase::floor(v.gross - v.rolled_amount() - v.converted_amount() - qcd - funding, 0);
        if kept == 0 {
            return after;
        }
        return fraction(after, kept + v.rolled_amount(), kept);
    }
    // A rollover takes the taxable amount first (a Roth account's is taxed anyway).
    after + phase::floor(v.taxable_before_rollover() - v.taxable_income(), 0)
}

// The designated part's taxable amount of each 2026 qualified disaster distribution
// (lines 10 and 18/20 before the spread), by entry: the part of the distribution, after
// any gift to charity, that is designated, at the entry's taxable share. It leaves Form
// 5329 line 1 (retirement::early_tax); the rest keeps its 1099-R treatment.
pub(crate) fn designated(
    d: &FederalDraft,
    basis: &crate::retirement::BasisResult,
    qcd: &crate::retirement::QcdResult,
) -> BTreeMap<i64, i64> {
    let mut out = basis.inherited.designated.clone();
    for e in &d.income {
        let FederalIncome::Retirement { data: v } = &e.income else {
            continue;
        };
        let DisasterDistribution::Qualified { amount, .. } = v.disaster else {
            continue;
        };
        if v.kind == RetirementKind::Inherited
            && matches!(
                crate::inherited_ira::link_of(d, e),
                Some(crate::inherited_ira::Link::Record(_))
            )
        {
            continue;
        }
        let gift = qcd.for_entry(e.id);
        let whole = v.gross - gift;
        if whole <= 0 || v.kind == RetirementKind::Nonqualified {
            continue;
        }
        let amount = phase::cap(amount, [whole]);
        let taxable = taxable_before_rollover(e, v, basis, gift);
        out.insert(e.id, phase::cap(fraction(taxable, amount, whole), [amount]));
    }
    out
}

#[derive(Default)]
struct Form {
    projection: projection::Projection,
    // Part II and Part III: current designated amounts, their taxable
    // parts, carried income, repayments.
    plan_amount: i64,
    plan_taxable: i64,
    ira_amount: i64,
    ira_taxable: i64,
    plan_carried: i64,
    ira_carried: i64,
    plan_excess_in: i64,
    ira_excess_in: i64,
    plan_repaid: i64,
    ira_repaid: i64,
    // Line 11 and 22 amounts of 2025 forms: their third installment is 2027.
    plan_2025: i64,
    ira_2025: i64,
    spread: Option<DisasterSpread>,
    // Part I to III content: a form with only Part IV shows no spread lines.
    spread_parts: bool,
    // Part IV lines 28, 29 and 31 (the distributions for a main home, their cost, the
    // taxable part repaid).
    home_28: i64,
    home_29: i64,
    home_31: i64,
}

// Runtime attribution of the same final adjustment; columns are owner,
// rows are the unchanged IRA/pension categories. No second worksheet.
#[derive(Default)]
pub(crate) struct DisasterResult {
    pub(crate) categories: [i64; 2],
    pub(crate) owners: [[i64; 2]; 2],
}
// Form 8915-F for each owner and disaster year: the change to the IRA
// (index 0) and pension (index 1) taxable amounts, in cents, that replaces
// the designated parts' full taxable amounts with lines 26 and 15.
pub(crate) fn calculate(
    d: &FederalDraft,
    r: &mut FederalResult,
    basis: &crate::retirement::BasisResult,
    qcd: &crate::retirement::QcdResult,
) -> DisasterResult {
    let mut forms: BTreeMap<(usize, u32), Form> = BTreeMap::new();
    let mut limits: BTreeMap<(usize, u32), (i64, i64)> = BTreeMap::new();
    let mut adjustment = [0, 0];
    let mut owners = [[0; 2]; 2];
    // The Roth IRA rows with a designated disaster share (Form 8606 line 25b, Form 8915-F line
    // 19) are IRA rows for the form: their designated taxable amount is never in Form 1040
    // line 4b, so nothing is taken out of it here.
    let mut rows: alloc::vec::Vec<(
        &crate::FederalIncomeEntry,
        alloc::borrow::Cow<'_, crate::FederalRetirement>,
        bool,
    )> = alloc::vec::Vec::new();
    for e in &d.income {
        match &e.income {
            FederalIncome::Retirement { data } => {
                rows.push((e, alloc::borrow::Cow::Borrowed(data), false));
            }
            FederalIncome::Roth { data: w }
                if matches!(w.disaster, DisasterDistribution::Qualified { .. }) =>
            {
                rows.push((
                    e,
                    alloc::borrow::Cow::Owned(crate::FederalRetirement {
                        disability: crate::DisabilityIncome::None,
                        kind: RetirementKind::Ira,
                        disability_wages: crate::DisabilityWages::None,
                        gross: w.gross - w.rolled_over,
                        taxable: RetirementTaxable::Payer { amount: 0 },
                        withholding: 0,
                        box7: 0,
                        ordinary_distribution: crate::RetirementTreatment::Ordinary,
                        disaster: w.disaster.clone(),
                        after_death: None,
                        box_10: 0,
                        box_9b: None,
                    }),
                    true,
                ));
            }
            _ => {}
        }
    }
    for (e, v, roth) in &rows {
        let (e, roth) = (*e, *roth);
        let v: &crate::FederalRetirement = v;
        if v.disaster == DisasterDistribution::No {
            continue;
        }
        #[cfg(test)]
        let source = alloc::format!("Income: {}", e.label);
        let owner = usize::from(e.owner == Owner::Spouse);
        // Part IV: a distribution for a main home. The taxable amount less the part repaid
        // stays in income (line 32) and keeps the row's 10% path (retirement::early_tax).
        if let DisasterDistribution::HomePurchase { began, taxable_repaid: repaid, .. } = v.disaster
        {
            let line_28 = phase::floor(v.gross - v.rolled_amount() - v.converted_amount(), 0);
            let form = forms.entry((owner, began / 10000)).or_default();
            form.home_28 += line_28;
            if basis.home_repaid.contains(&e.id) {
                // An IRA's cost or basis is figured on this year's Form 8606, which already
                // left the repaid amount out of line 7 (i8915-F lines 29 and 31): line 29 is
                // zero and the taxable amount is net of the repayment.
                form.projection.home_8606 = true;
                form.home_31 += phase::cap(repaid, [line_28]);
                continue;
            }
            let line_30 = phase::bound(basis.taxable(e.id, v), 0, line_28);
            form.home_29 += line_28 - line_30;
            let repaid = phase::cap(repaid, [line_30]);
            form.home_31 += repaid;
            form.projection.home(e, line_28, line_28 - line_30, repaid);
            adjustment[usize::from(v.kind != RetirementKind::Ira)] -= repaid;
            continue;
        }
        let began = match entry_refusal(v) {
            Ok(began) => began,
            Err((key, _message)) => {
                #[cfg(test)]
                let message = _message;

                crate::issue_localized!(
                    r,
                    &crate::IssueContext::Section(crate::TaxSection::Retirement),
                    &source,
                    key,
                    message,
                );
                continue;
            }
        };
        let DisasterDistribution::Qualified { fema, amount, earlier, spread, ref repaid, .. } =
            v.disaster
        else {
            continue;
        };
        let amount = phase::floor(phase::cap(amount, [v.gross - qcd.for_entry(e.id)]), 0);
        let limit = limits.entry((owner, fema)).or_insert((0, earlier));
        limit.0 += amount;
        crate::require_localized!(
            r,
            limit.1 == earlier,
            &crate::IssueContext::Section(crate::TaxSection::Retirement),
            &source,
            "ISSUE_FORM_8915_F_LINE_1B_ENTER_THE_SAME_EARLIER_QUALIFIED_DISASTER_DISTR_530EA91B",
            "Form 8915-F line 1b: enter the same earlier qualified disaster distributions on every 1099-R for this disaster.",
        );
        let taxable = if roth {
            basis.roth.designated.get(&e.id).copied().unwrap_or(0)
        } else {
            basis.disaster.get(&e.id).copied().unwrap_or(0)
        };
        let repaid_taxable =
            if amount == 0 { 0 } else { fraction(repaid.total(), taxable, amount) };
        let form = forms.entry((owner, began / 10000)).or_default();
        form.spread_parts = true;
        crate::require_localized!(
            r,
            form.spread.is_none_or(|s| s == spread),
            &crate::IssueContext::Section(crate::TaxSection::Retirement),
            &source,
            "ISSUE_FORM_8915_F_LINES_11_AND_22_ONE_ELECTION_APPLIES_TO_EVERY_QUALIFIED_BC403371",
            "Form 8915-F lines 11 and 22: one election applies to every qualified disaster distribution of the same disaster year on this filer's form. Choose the same timing on each.",
        );
        form.spread = Some(spread);
        form.projection.record(d, r, e, amount, taxable);
        if v.kind.ira() {
            form.ira_amount += amount;
            form.ira_taxable += taxable;
            form.ira_repaid += repaid_taxable;
            if !roth {
                adjustment[0] -= taxable;
                owners[0][owner] -= taxable;
            }
        } else {
            form.plan_amount += amount;
            form.plan_taxable += taxable;
            form.plan_repaid += repaid_taxable;
            adjustment[1] -= taxable;
            owners[1][owner] -= taxable;
        }
    }
    for ((owner, fema), (amount, earlier)) in &limits {
        crate::federal_rules::require_message(
            r,
            amount + earlier <= LIMIT,
            &crate::IssueContext::owner(
                crate::TaxSection::Retirement,
                if *owner == 0 { Owner::Taxpayer } else { Owner::Spouse },
            ),
            #[cfg(test)]
            "Form 8915-F",
            #[cfg(test)]
            alloc::format!(
                "Line 1e: qualified disaster distributions for disaster DR-{fema} of {} cannot exceed $22,000 less earlier years; treat less as qualified.",
                if *owner == 0 { "you" } else { "your spouse" }
            ),
            crate::Message::new(
                if *owner == 0 {
                    "ISSUE_DISASTER_LIMIT_TAXPAYER"
                } else {
                    "ISSUE_DISASTER_LIMIT_SPOUSE"
                },
                alloc::vec![crate::MessageArg::Text(alloc::format!("{fema}"))],
            ),
        );
    }
    let carries = d.screening.as_ref().map_or(&[][..], |s| &s.disaster_carries[..]);
    for (i, carry) in carries.iter().enumerate() {
        let f = &carry.form;
        let owner = usize::from(carry.owner == Owner::Spouse);
        let Some(year) = f.year() else {
            crate::require_localized!(
                r,
                false,
                &crate::IssueContext::Review,
                "Form 8915-F",
                "ISSUE_A_FORM_8915_F_FROM_2023_OR_EARLIER_HAS_NO_2026_INCOME_A_2026_REPAYM_8F28E5B6",
                "A Form 8915-F from 2023 or earlier has no 2026 income; a 2026 repayment of those distributions is carried back to 2023-2025 on amended returns (Form 1040-X, Form 8915-F lines 14 and 25), which this App does not prepare.",
            );
            continue;
        };
        crate::require_localized!(
            r,
            f.valid()
                && !carries[..i].iter().any(|c| {
                    c.owner == carry.owner
                        && c.form.year() == Some(year)
                        && c.form.disaster_year == f.disaster_year
                }),
            &crate::IssueContext::owner(crate::TaxSection::Retirement, carry.owner)
                .item("disaster_carries", i),
            "Form 8915-F",
            "ISSUE_ENTER_EACH_EARLIER_FORM_8915_F_ONCE_WITH_A_DISASTER_YEAR_NO_LATER_T_087EE17B",
            "Enter each earlier Form 8915-F once, with a disaster year no later than its tax year and a carried-back amount no larger than its line 14 or 25 excess.",
        );
        let (plan_repaid, ira_repaid, plan_back, ira_back) = f.repaid();
        let (plan_excess, ira_excess) = f.excess_2025();
        let form = forms.entry((owner, f.disaster_year)).or_default();
        form.spread_parts = true;
        form.plan_carried += f.line_11;
        form.ira_carried += f.line_22;
        // Any 2025 Form 8915-F spreads over 2025-2027, whichever year the
        // disaster began (item B).
        if year == 2025 {
            form.plan_2025 += f.line_11;
            form.ira_2025 += f.line_22;
        }
        form.plan_excess_in += phase::floor(plan_excess - plan_back, 0);
        form.ira_excess_in += phase::floor(ira_excess - ira_back, 0);
        form.plan_repaid += plan_repaid;
        form.ira_repaid += ira_repaid;
    }
    let mut available = projection::available(d);
    for ((owner, year), form) in &forms {
        let person = d
            .people
            .as_ref()
            .and_then(|p| if *owner == 0 { Some(&p.taxpayer) } else { p.spouse.as_ref() });
        let prefix = if *owner == 0 { "taxpayer" } else { "spouse" };
        // A filer who died in 2026 includes the rest of every spread on the
        // final return (Form 8915-F lines 13 and 24): the whole taxable amount
        // of the 2026 distributions on lines 11 and 22, and the third
        // installment of a 2025 spread that 2027 would have brought.
        let died = person.is_some_and(|p| p.died.is_some());
        let spread = form.spread != Some(DisasterSpread::AllInReturnYear) && !died;
        let third = |v: i64| if spread { fraction(v, 1, 3) } else { v };
        let (rest_plan, rest_ira) = if died { (form.plan_2025, form.ira_2025) } else { (0, 0) };
        let line_11 = third(form.plan_taxable);
        let line_13 = line_11 + form.plan_carried + rest_plan;
        let line_14 = form.plan_excess_in + form.plan_repaid;
        let line_15 = phase::floor(line_13 - line_14, 0);
        let line_22 = third(form.ira_taxable);
        let line_24 = line_22 + form.ira_carried + rest_ira;
        let line_25 = form.ira_excess_in + form.ira_repaid;
        let line_26 = phase::floor(line_24 - line_25, 0);
        adjustment[1] += line_15;
        owners[1][*owner] += line_15;
        adjustment[0] += line_26;
        owners[0][*owner] += line_26;
        let installment_following_year = if spread { line_11 + line_22 } else { 0 };
        let installment_second_following_year =
            form.plan_taxable + form.ira_taxable - line_11 - line_22 - installment_following_year;
        // Part IV: line 30 is the taxable part of the distributions for a main home and
        // line 32 what stays in income after the repayment.
        let (line_30, line_32) =
            (form.home_28 - form.home_29, form.home_28 - form.home_29 - form.home_31);
        let part_iv = if form.home_28 > 0 {
            alloc::vec![
                ("line_28", form.home_28, msg!("EXPLAIN_FORM_N_N_N_N_LINE_N_16")),
                ("line_29", form.home_29, msg!("EXPLAIN_FORM_N_N_N_N_LINE_N_17")),
                ("line_30", line_30, msg!("EXPLAIN_FORM_N_N_N_N_LINE_N_18")),
                ("line_31", form.home_31, msg!("EXPLAIN_FORM_N_N_N_N_LINE_N_19")),
                ("line_32", line_32, msg!("EXPLAIN_FORM_N_N_N_N_LINE_N_20")),
            ]
        } else {
            alloc::vec![]
        };
        emit_lines(r, &mut (), part_iv, |(suffix, amount, note), _, _| {
            Some(calculation_row(
                None,
                &alloc::format!("form_8915_{prefix}_{year}_{suffix}"),
                LineValue::MoneyCents(amount),
                &["income.retirement.disaster"],
                "https://www.irs.gov/instructions/i8915f",
                note,
            ))
        });
        if !form.spread_parts {
            continue;
        }
        emit_lines(
            r,
            &mut (),
            [
                ("line_6", form.plan_amount + form.ira_amount, msg!("EXPLAIN_FORM_N_N_N_N_LINE_N")),
                ("line_8", form.plan_amount, msg!("EXPLAIN_FORM_N_N_N_N_LINE_N_2")),
                (
                    "line_9",
                    form.plan_amount - form.plan_taxable,
                    msg!("EXPLAIN_FORM_N_N_N_N_LINE_N_3"),
                ),
                ("line_10", form.plan_taxable, msg!("EXPLAIN_FORM_N_N_N_N_LINE_N_4")),
                ("line_11", line_11, msg!("EXPLAIN_FORM_N_N_N_N_LINE_N_5")),
                ("line_12", form.plan_carried, msg!("EXPLAIN_FORM_N_N_N_N_LINE_N_6")),
                ("line_13", line_13, msg!("EXPLAIN_FORM_N_N_N_N_LINE_N_7")),
                ("line_14", line_14, msg!("EXPLAIN_FORM_N_N_N_N_LINE_N_8")),
                ("line_15", line_15, msg!("EXPLAIN_FORM_N_N_N_N_LINE_N_9")),
                ("line_20", form.ira_taxable, msg!("EXPLAIN_FORM_N_N_N_N_LINE_N_10")),
                ("line_22", line_22, msg!("EXPLAIN_FORM_N_N_N_N_LINE_N_11")),
                ("line_23", form.ira_carried, msg!("EXPLAIN_FORM_N_N_N_N_LINE_N_12")),
                ("line_24", line_24, msg!("EXPLAIN_FORM_N_N_N_N_LINE_N_13")),
                ("line_25", line_25, msg!("EXPLAIN_FORM_N_N_N_N_LINE_N_14")),
                ("line_26", line_26, msg!("EXPLAIN_FORM_N_N_N_N_LINE_N_15")),
                (
                    "excess_repayment",
                    phase::floor(line_14 - line_13, 0) + phase::floor(line_25 - line_24, 0),
                    msg!("EXPLAIN_FORM_N_N_N_N_EXCESS_REPAYMENT"),
                ),
                (
                    "installment_following_year",
                    installment_following_year,
                    msg!("EXPLAIN_FORM_N_N_N_N_INSTALLMENT_N"),
                ),
                (
                    "installment_second_following_year",
                    installment_second_following_year,
                    msg!("EXPLAIN_FORM_N_N_N_N_INSTALLMENT_N_2"),
                ),
            ],
            |(suffix, amount, note), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!("form_8915_{prefix}_{year}_{suffix}"),
                    LineValue::MoneyCents(amount),
                    &["income.retirement.disaster", "screening.disaster_carries"],
                    "https://www.irs.gov/instructions/i8915f",
                    note,
                ))
            },
        );
    }
    for ((owner, year), form) in &forms {
        let later = forms.iter().any(|((next_owner, next_year), next)| {
            next_owner == owner && next_year > year && !next.projection.disasters.is_empty()
        });
        projection::emit(r, *owner, *year, form, &mut available[*owner], later);
    }
    DisasterResult { categories: adjustment, owners }
}
