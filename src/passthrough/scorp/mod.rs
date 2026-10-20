use crate::kernel::carry::{Amount, Cents};
// Coverage update B9-7 R11/R12 (2026-10-11): uncomputed K-1 credits and
//   oil/gas depletion leave only the benefit out, with one scoped notice;
//   code AX credit recapture remains refused. Accepted facts survive the
//   estimate and reopen; no unclaimed depletion reduces basis or SE earnings.
//   Sources: 2025 final K-1 instructions (1065/1120-S), credit code AX and
//   depletion code 20T/17R; tests/k1_items and tests/interview k1_benefit_*.
// Coverage update B9-7 R7 (2026-10-11): box 6 royalties and box 12 code I
//   reach Schedule E Part I through the shared basis and at-risk ledger, outside
//   Form 8582, QBI and SE. Shared portfolio income increases stock basis and
//   amount at risk; separate AMT balances determine the AMT deduction.
//   Sources: i1120ssk (2025 final and 2026 draft of September 18), Schedule E
//   (2026 draft, created May 6). tests/k1_items k1_royalty_*.
// Coverage: Schedule K-1 (1120S) — Partial (audited 2026-09-24).
//   Lines: Boxes 1, 4, 5a/5b, 8a–8c/9 (portfolio capital and business 1231 records), 13 code O, 15
//   code F, 16 A–D; Form 7203 stock/debt basis, loan-repayment gains, at-risk, passive, QBI (199A
//   statement), independent AMT basis. Refused via basis_complete gate: rental boxes 2/3,
//   boxes 10–12 items, other credits, foreign/PTEP, farming, stock dispositions, debt
//   forgiveness, ownership termination.
//   Evidence: scorp.rs, scorp_basis.rs, scorp_risk.rs, scorp_qbi.rs, scorp_1231.rs,
//   scorp_capital.rs, scorp_amt.rs, federal_rules.rs; tests/passthrough/passthrough.rs
//   scorp_interview_preserves_source_and_shows_basis_without_claiming_return_coverage,
//   scorp_statement_uses_allocated_wages_and_ubia_for_high_income_limit,
//   scorp_tax_exempt_sources_release_loss_without_becoming_business_income;
//   tests/passthrough/scorp_stock_basis.rs debt_regulation_examples_one_and_two_preserve_separate_notes;
//   sources i1120ssk, i7203.
//   Unit T2-31 (2026-09-26): box 12 code A cash contributions (ScorpK1.cash_charity) reach Schedule
//   A line 11 or the non-itemizer line 12f cap with the partnership gifts, reduce stock then debt
//   basis as their own Form 7203 Part III class (CHARITY_INDEX), and stay outside Form 6198, Form
//   8582 and QBI; a gift limited by basis (CHARITY_LIMITED) or joined by a prior cash carryover is
//   refused by name. Box 16 code D above stock basis is capital gain on Form 8949/Schedule D with
//   ScorpK1.stock_term (regular and AMT basis separately; Form 4952 investment gain only without
//   material participation) and Form 6198 line 2a income; refused by name: an unknown holding
//   period, a passive or unclassified activity (Form 8582 allocation) and modified AGI above the
//   NIIT threshold for a non-trading business (Form 8960 line 5c adjustment). Every other box with
//   an amount is listed in ScorpK1.unentered (ScorpK1Item) and refused by name: boxes 2/3, 10,
//   12 other codes, 13 other codes (incl. H), 14 and 16F, 15
//   beyond the AMT record and code F, 17 codes other than V, multiple activities. Box 16 code E is
//   the loan-repayment record. STORAGE_FORMAT 46. Tests: tests/passthrough/passthrough.rs
//   shareholder_k1_items_and_distribution_above_basis_reach_the_return_by_hand,
//   shareholder_cash_gift_exactly_absorbed_by_basis_is_allowed_and_one_dollar_more_is_refused,
//   shareholder_cash_gift_above_the_non_itemizer_cap_is_limited_to_1000,
//   shareholder_distribution_equal_to_basis_has_no_gain_and_one_dollar_more_is_capital_gain,
//   shareholder_distribution_gain_meets_the_investment_tax_threshold,
//   shareholder_boxes_not_calculated_and_passive_distribution_gain_are_refused_by_name.
//   Unit T2-29b (2026-09-26): ScorpK1.section179 (box 11) is one Form 4562 line 6 item of the
//   shareholder's own Part I (section179.rs): line 8 without the line 2 phaseout (Reg.
//   1.179-2(b)(3)), line 11 with the materially participating shareholder's box 1 net after basis
//   and at-risk limits (Reg. 1.179-2(c)(3)). The full box 11 amount is its own Form 7203 Part III
//   class (SECTION179_INDEX, Reg. 1.179-3(h)(1)) for regular and AMT basis; the line 12 share reaches
//   Schedule E line 28 column (j) and reduces the statement's ordinary QBI (ScorpQbiStatement::
//   activities); the line 13 carryover is reported per K-1. Refused by name: a share suspended by
//   basis (SECTION179_LIMITED) or above the remaining amount at risk (Form 6198), a shareholder
//   without established material participation (Form 8582), a share above the dollar limit; box
//   17 codes K/L stay in ScorpK1Item::OtherInformation. ScorpK1Item::Section179 is removed.
//   STORAGE_FORMAT 55. Tests: tests/passthrough/passthrough.rs
//   shareholder_section179_reaches_form_4562_schedule_e_basis_and_qbi_by_hand,
//   shareholder_section179_exactly_absorbed_by_basis_is_allowed_and_one_dollar_more_is_refused,
//   shareholder_section179_without_material_participation_is_refused_by_name; sources i4562,
//   i1120ssk, Reg. 1.179-2, 1.179-3, i8995.
//   Unit T2-31p (2026-09-27): the Schedule K-1 topic's Form 1120-S kind (k1_page.rs K1Scorp) writes
//   this record; RareSituation::PassThrough is removed. New stored ScorpK1.section179_carryover
//   (question C13, Form 4562 line 10; see depreciation.rs). ScorpK1::schedule_e_amount is the
//   page row's amount: Schedule E after basis, at-risk and the activity's own passive loss, blank
//   while refused or while it depends on the return (section 179, a net passive loss, section
//   1231 items). STORAGE_FORMAT 62.
//   Not-sure audit batch 7 (2026-10-05): risk and passive are required records; QBI loss
//   histories (ScorpLossQbi.at_risk/passive) are required lists; participation year counts are
//   required; ScorpLoan.repayment_term is required, NoRepayment for an open account or a note
//   with nothing repaid (SCORP_LOAN_REPAYMENT_TERM on both sides). No box 17 code V statement
//   (qbi_statement None) omits current reported QBI (Reg. 1.199A-6(b)(3)(iii)) with
//   WARN_K1_QBI_NO_STATEMENT, or leaves a current loss out with WARN_K1_QBI_LOSS_NO_STATEMENT.
//   Original-year qualified suspended losses still count when actually allowed (i8995,
//   Reg. 1.199A-3(b)(1)(iv)); they do not require a new current statement. STORAGE_FORMAT 84.
//   Batch 7 non-nullable QBI tracking (2026-10-05): loss_qbi and property_qbi are plain records,
//   zero and empty when nothing applies. Personal histories survive NoStatement imports in
//   separate non-imported records (STORAGE_FORMAT 86): only histories with actual prior loss
//   balances participate; unrelated retained histories are structurally checked but inactive.
//   Current reported QBI is projected to zero without a statement, while released historical
//   qualified losses retain their original-year status. Missing applicable history omits its
//   QBI loss character with WARN_SCORP_QBI_HISTORY_LEFT_OUT; supplied mismatches remain invalid.
//   Tests: income_navigation no_statement_missing_history_warns_for_each_actual_suspension_stage,
//   no_statement_partial_history_omits_only_its_character_and_rejects_supplied_mismatch,
//   no_statement_negative_current_loss_still_counts_released_qualified_history and
//   no_statement_unrelated_histories_remain_inactive_and_retained; interview
//   k1_1120s_import_no_statement_preserves_personal_records_after_save_and_reopen.
//   Unit k1-rentals (2026-10-09): boxes 2 and 3 are calculated (k1_rental.rs): ScorpK1.rentals and
//   ScorpAmt.rentals (box amounts and 2025 Form 8582 carryovers per tax system) raise stock basis
//   (Form 7203 lines 3b/3c) and Form 6198 income; their losses are Form 7203 lines 36/37
//   (RENTAL_INDEX, after section 179) and Form 6198 losses of the one activity; a rental loss
//   either limit suspends is refused by name. ScorpK1.rental_facts: participation (a shareholder
//   may be Active) and the section 199A rental amounts. The passive history confirmation also
//   covers the rental carryovers (unknowns.rs). ScorpK1Item::Rentals is renamed RentalDisposition
//   (refused by name); MultipleActivities narrows to box 18 or a box 19 statement with two
//   activities of one kind. schedule_e_amount is blank with rentals. Tests: tests/passthrough/k1_rental.rs
//   fixture_e_*, fixture_f_*; tests/passthrough/interview.rs k1_1120s_rentals_map_onto_the_record_and_reopen_unchanged.
//   Unit k3 (2026-10-09): box 16 code F foreign taxes (ScorpK1.foreign_taxes) are a Form 7203 Part III
//   line 46 deduction class of their own (FOREIGN_TAX_INDEX, after section 179; sections 1366(a)(1)(A)
//   and 1367(a)(2)(B)) and are never read by Form 1116: the credit is figured from the Schedule K-3 on
//   ScorpK1.schedule_k3 (schedule_k3.rs, which owns the Coverage: Schedule K-3 block). A share
//   limited by stock and debt basis is refused by name (FOREIGN_TAX_LIMITED). RENTAL_INDEX moves
//   to 8. ScorpK1Item::International is removed; the K-3 items ForeignCorporations and
//   DisallowedDeduction carry its refusal key. Tests: tests/foreign/foreign.rs k3_general_category_*.
//   Unit form-3468 (2026-10-09): ScorpK1.investment_property (box 13 code E; box 17 codes C, D, AV) and
//   investment_confirmed (Form 3468, investment_credit.rs) reach Form 3800 Part III lines 4k, 4a and
//   1v like the partnership's, and the section 50(c)(5) reduction lowers the stock basis
//   (StockBasis.credit_reduction, Form 7203 line 8c, never carried); the recapture code (box 17
//   code G) stays refused.
//   Unit B8 (2026-10-10): the index constants (CHARITY_INDEX ... RENTAL_INDEX) and the three basis
//   refusals (CHARITY_LIMITED, SECTION179_LIMITED, FOREIGN_TAX_LIMITED) are gone. ScorpK1::class_input
//   (scorp_capital.rs) builds one class input per K-1 and tax system from the K1LossClass catalog:
//   box 1, 2 and 3 losses, the three capital classes, section 1231, box 11 section 179, box 12 code
//   A gifts and box 16 code F foreign taxes, each with its basis carry and at-risk carry (new:
//   ScorpK1.carried, ScorpAmt.carried; Form 7203 Part III column (b)). Form 7203 Part III and Form
//   6198 share each limit pro rata across them and ScorpRiskResult.ledger (LossLedger) says what each
//   allowed and held back; the held-back part carries to 2027 (NOTE_K1_LOSS_CARRYFORWARD, AMT twin
//   NOTE_K1_AMT_CLASS_CARRYFORWARD). The basis-allowed section 179 is the Form 4562 line 6 item and
//   the basis-allowed gift is the Schedule A / section 170(p) gift. Tests: tests/passthrough/k1_loss.rs.
//   Unit B9-1 (2026-10-10): the box 10 and box 12 codes (k1_items.rs; ScorpK1.income and
//   deductions), the sale of all the shares (ScorpK1.disposition, disposal(): proceeds less the
//   ending stock basis, Form 8949 by stock_term) and the distribution above stock basis without
//   its passive refusal (a passive gain is taken as not passive, WARN_K1_GAIN_NOT_PASSIVE).
//   basis_complete becomes basis_complete(): ScorpK1.special lists the remaining situations.
//   Stored-shape change (STORAGE_FORMAT bumped by the dispatcher). Tests: tests/passthrough/k1_items.rs.

use crate::FederalResult;
use crate::k1_loss::{K1LossClass, LossLedger};
use crate::kernel::phase;
use crate::scorp_basis::{BasisError, BasisResult, LoanKind, ShareholderLoan, StockBasis};
use alloc::{borrow::Cow, vec::Vec};
use libertas_macros::*;

/// Shareholder loan documentation
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum ScorpLoanKind {
    /// Separate written note
    FormalNote,
    /// Current open account
    /// Combine advances without separate written instruments in one open account. Its opening principal cannot exceed $25,000.
    OpenAccount,
    /// Former open account that previously exceeded $25,000
    /// Keep this debt separate even if its balance has since fallen. Annual netting no longer applies; repayment gain remains ordinary.
    FormerOpenAccount,
}

/// Principal repayment of a written note
/// Exclude interest and identify the actual payment date. Keep every repayment of this note in the same history.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ScorpNoteRepayment {
    /// Payment date
    #[libertas_date_only]
    pub date: u32,
    /// Principal repaid
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
    pub principal: i64,
}

/// Written-note holding-period facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ScorpNoteHolding {
    /// No written-note principal repaid in 2026
    /// An open account, or a written note with no principal repaid during 2026. Open-account repayment gains are always ordinary.
    NoRepayment,
    /// All taxable repayments have one established holding period
    Single {
        /// Holding period established by the loan records
        term: crate::GainTerm,
    },
    /// Calculate holding periods from dated repayments
    /// For an existing original-cash-advance note with no further principal advanced during 2026. Keep the note as one basis record, even when repayments fall in different holding periods. Additional advances need a transaction-level basis history.
    Dated {
        /// Original advance date
        /// The holding period starts the next day. Do not use the date of a later replacement instrument or a basis restoration.
        #[libertas_date_only]
        advanced: u32,
        /// Principal repayments during 2026
        /// These must total the principal repaid on the loan record. Their dates determine short- or long-term treatment; list order does not affect the result.
        /// ----
        /// Repayment
        #[libertas_size(min = 1)]
        payments: Vec<ScorpNoteRepayment>,
    },
}

impl ScorpLoan {
    pub(crate) fn source_valid(&self) -> bool {
        // Arithmetic safety: basis_result and loan_gains retain native
        // additions, gain allocation and dollar-to-cent transfers.
        [self.principal, self.basis, self.advances, self.repayments]
            .into_iter()
            .all(|amount| (0..=100000000000).contains(&amount))
            && self.repayment_term.valid(self)
            && self.basis <= self.principal
            && self.repayments <= self.principal + self.advances
            // Form 7203 open-account debt is aggregated as one account while
            // its year-end balance does not exceed $25,000.
            && (self.kind != ScorpLoanKind::OpenAccount || self.principal <= 2500000)
    }

    pub(crate) fn sources_valid<'a>(mut loans: impl Iterator<Item = &'a Self> + Clone) -> bool {
        // Arithmetic safety: scorp_basis deficit, priority and basis allocations,
        // loan_gains, community loan sums and k1_totals dollar-to-cent transfers
        // retain native i64 receivers. A capped final allocation is not a bound
        // on their intermediate totals.
        loans.clone().count() <= 100
            && loans.clone().filter(|loan| loan.kind == ScorpLoanKind::OpenAccount).count() <= 1
            && loans.all(Self::source_valid)
    }

    // The server twin of SCORP_LOAN_REPAYMENT_TERM: a written note with
    // principal repaid in 2026 has a holding period, and an open account has
    // no written-note repayment.
    pub(crate) fn term_matches(&self) -> bool {
        match self.repayment_term {
            ScorpNoteHolding::NoRepayment => {
                self.kind != ScorpLoanKind::FormalNote || self.repayments == 0
            }
            _ => self.kind == ScorpLoanKind::FormalNote,
        }
    }
}

impl ScorpNoteHolding {
    fn valid(&self, loan: &ScorpLoan) -> bool {
        loan.term_matches()
            && match self {
                Self::NoRepayment | Self::Single { .. } => true,
                Self::Dated { advanced, payments } => {
                    crate::federal_rules::valid_date(*advanced)
                        && *advanced < 20260101
                        && loan.principal > 0
                        && !payments.is_empty()
                        && payments.iter().all(|p| {
                            crate::federal_rules::valid_date(p.date)
                                && (20260101..=20261231).contains(&p.date)
                                && p.principal > 0
                        })
                        // Positive repayments reconcile to the actual admitted loan.
                        // Checked accumulation also covers native AI submissions.
                        && payments.iter().try_fold(0i64, |sum, p| sum.checked_add(p.principal))
                            == Some(loan.repayments)
                }
            }
    }

    fn split(&self, loan: &ScorpLoan, gain: i64) -> Result<[i64; 2], (&'static str, &'static str)> {
        match self {
            // valid() keeps this to loans with no written-note repayment,
            // which have no capital repayment gain.
            Self::NoRepayment => Err((
                "ISSUE_SCORP_LOAN_BASIS_SOURCE_MISMATCH",
                crate::fixture_english!(
                    "The shareholder-loan basis results do not match the source records."
                ),
            )),
            Self::Single { term: crate::GainTerm::Short } => Ok([gain, 0]),
            Self::Single { term: crate::GainTerm::Long } => Ok([0, gain]),
            Self::Dated { advanced, payments } => {
                if loan.advances != 0 {
                    return Err((
                        "ISSUE_SCORP_NOTE_ADVANCES_WITH_DATED_REPAYMENTS",
                        crate::fixture_english!(
                            "Dated note repayments with new principal advances require transaction-level basis and holding-period allocation."
                        ),
                    ));
                }
                let mut principal = [0; 2];
                for payment in payments {
                    principal[usize::from(payment.date > advanced + 10000)] += payment.principal;
                }
                // With no intervening advances, repayments preserve the ratio
                // of adjusted basis to principal. Restore basis before the
                // first repayment; allocate that year's computed gain once,
                // preserving source-cent allocation across both holding terms.
                let split = crate::kernel::ratio::allocate_remaining(gain, &principal);
                Ok([split[0], split[1]])
            }
        }
    }
}

/// Direct shareholder loan
/// Include bona fide debt owed directly to this shareholder. A guarantee or co-signature alone gives no basis. Enter each written note separately; exclude interest from principal repayments.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct ScorpLoan {
    /// Loan documentation
    pub kind: ScorpLoanKind,
    /// Holding periods of written-note repayments
    /// Required for a written note with principal repaid in 2026: the note and your loan records show the date the money was advanced and each repayment date. Use one classification when every repayment has the same holding period, or supply the dated repayments of an existing note without new advances. More than one year is long-term; never split one note into artificial loans to allocate gain. Choose no repayment for an open account or a note with nothing repaid.
    pub repayment_term: ScorpNoteHolding,
    /// Opening principal owed
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub principal: i64,
    /// Opening adjusted debt basis
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub basis: i64,
    /// Principal advanced during the year
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub advances: i64,
    /// Principal repaid during the year
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub repayments: i64,
}

/// Prior passive loss and ordinary-pool eligibility
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ScorpPassiveHistory {
    /// Ordinary losses suspended by passive limits at the end of 2025
    /// From the retained activity worksheet. These losses already passed basis and at-risk limits; do not include those separate balances here.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub loss: i64,
    /// Same continuing activity, with no special passive rules
    /// Confirm no disposition, grouping change, self-charged interest, significant-participation income recharacterization or other special passive rule. Current participation determines whether older passive losses first offset this activity’s current income. Additional cases need their separate worksheets. It covers the box 2 and box 3 rental activities' carryovers too. The prior-year Form 8582 worksheets for these activities show it; No leaves the suspended losses out of this estimate.
    pub complete: bool,
}

/// Other K-1 box with an amount
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum ScorpK1Item {
    /// Rental property disposed of by installment sale, exchange, casualty or theft, or to a related buyer
    /// Reported with box 2 or 3 on the corporation's statement, in box 9, box 10 or box 17.
    RentalDisposition,
    /// Other income or loss — box 10 codes not entered above
    OtherIncome,
    /// Credits — box 13 codes not entered above
    /// Your share of credits this App does not figure. They are left out of the estimate; the rest of the K-1 is calculated.
    Credits,
    /// Carbon oxide sequestration credit recapture — box 13 code AX
    CreditRecapture,
    /// Oil and gas depletion information — box 17 code R
    /// Your depletion deduction is left out of the estimate; the rest of the K-1 is calculated.
    Depletion,
    /// Alternative minimum tax items — box 15
    /// Only items the separate AMT records and the private-activity bond interest above do not already include.
    AmtItems,
    /// Other information — box 17 codes that report an amount the App does not calculate
    /// Such as recapture, look-back interest or section 179 dispositions other than fully taxable sales. The investment income and expense codes, the section 199A statement (code V) and the investment credit property (codes C, D and AV) are not listed here.
    OtherInformation,
    /// Cooperative items on the section 199A statement — box 17 code V
    CooperativeQbi,
    /// More than one activity of one kind — box 18, or box 19
    /// Box 18 checked, or a box 19 statement listing more than one trade or business, more than one rental real estate activity or more than one other rental activity.
    MultipleActivities,
}

impl ScorpK1Item {
    pub(crate) fn left_out(self) -> bool {
        self.item().is_some()
    }
    pub(crate) fn item(self) -> Option<&'static str> {
        match self {
            Self::Credits => Some("K1_ITEM_SCORP_CREDITS"),
            Self::Depletion => Some("K1_ITEM_SCORP_DEPLETION"),
            _ => None,
        }
    }
    #[cfg(test)]
    pub(crate) fn refusal(self) -> &'static str {
        self.refusal_info().1
    }
    pub(crate) fn refusal_key(self) -> &'static str {
        self.refusal_info().0
    }
    fn refusal_info(self) -> (&'static str, &'static str) {
        match self {
            Self::RentalDisposition => (
                "ISSUE_SCORP_K1_RENTAL_DISPOSITION",
                crate::fixture_english!(
                    "An S corporation rental activity's installment sale, like-kind exchange, casualty or theft, or disposition to a related buyer needs separate property and passive-loss worksheets that this App does not calculate."
                ),
            ),
            Self::OtherIncome => (
                "ISSUE_SCORP_K1_OTHER_INCOME",
                crate::fixture_english!(
                    "Schedule K-1 (1120-S) box 10 codes B, D, E, F, G, I, M, N, O and ZZ, a code A portfolio loss or REMIC residual interest, a code C straddle and gambling of a gambling business are not calculated."
                ),
            ),
            Self::Credits | Self::Depletion => {
                unreachable!("left-out K-1 benefits are removed before calculation")
            }
            Self::CreditRecapture => crate::PartnerK1Item::CreditRecapture.refusal_info(),
            Self::AmtItems => (
                "ISSUE_SCORP_K1_AMT_ITEMS",
                crate::fixture_english!(
                    "Schedule K-1 (1120-S) box 15 alternative minimum tax items beyond the separate AMT records and the code F private-activity bond interest (Form 6251) are not calculated."
                ),
            ),
            Self::OtherInformation => (
                "ISSUE_SCORP_K1_OTHER_INFORMATION",
                crate::fixture_english!(
                    "Schedule K-1 (1120-S) box 17 recapture, look-back interest, section 179 dispositions other than fully taxable sales, installment and section 1260 interest, inversion gain, section 48A–48D and section 174A items (codes E through Q, AP, AS through AU, BA, and ZZ with an amount) are not calculated."
                ),
            ),
            Self::CooperativeQbi => crate::PartnerK1Item::CooperativeQbi.refusal_info(),
            Self::MultipleActivities => (
                "ISSUE_SCORP_K1_MULTIPLE_ACTIVITIES",
                crate::fixture_english!(
                    "Schedule K-1 (1120-S) box 18, more than one activity for at-risk purposes, or a box 19 statement with more than one trade or business, rental real estate or other rental activity (Form 6198 and Form 8582 by activity) is not calculated."
                ),
            ),
        }
    }
}

// Box 12 code A cash gifts of every S-corporation K-1 on the return that the
// stock and debt basis allows (this year's gift plus a released carry), in
// cents.
pub(crate) fn cash_charity(d: &crate::FederalDraft) -> i64 {
    d.income
        .iter()
        .filter_map(|e| match &e.income {
            crate::FederalIncome::Scorp { data } => Some(data.allowed_cash_charity()),
            _ => None,
        })
        .sum()
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum ScorpTradeKind {
    /// Ordinary trade or business
    /// Not a business trading financial instruments or commodities. Material participation determines whether ordinary operating income is included in investment income.
    Ordinary,
    /// Trading financial instruments or commodities for owners’ accounts
    /// This activity is nonpassive regardless of participation and remains subject to net investment income tax. Separately stated capital and portfolio items still need their own source records.
    FinancialTrading,
}

/// S-corporation K-1 and shareholder basis records
/// Enter one ordinary nonfarming U.S. business of one corporation and one shareholder per document. Complete regular and AMT basis, at-risk, participation and QBI records before return totals can be shown. Other shareholder items and special transactions require their separate worksheets. Employee W-2 wages belong in a separate wage document.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ScorpK1 {
    /// Entity's reporting tax period
    /// The actual beginning and ending dates of the entity year reported by this K-1. Leave unknown when its source records do not establish the period.
    pub reporting_period: Option<crate::K1EntityTaxPeriod>,
    /// Shareholder health plan, entered on Self-employed benefits
    /// Keep premiums separate from K-1 operating income and existing W-2 wages.
    #[libertas_default(None)]
    pub health_insurance: crate::BusinessHealthInsurance,
    /// Box 1 ordinary business income or loss
    /// Enter the signed K-1 amount before shareholder loss limits.
    #[libertas_money("USD", 2)]
    pub ordinary: i64,
    /// Box 4 portfolio interest income
    /// Taxable interest from this K-1, excluded from passive operating income and QBI. Do not repeat it in an interest document. Confirm its at-risk activity allocation below.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub interest: i64,
    /// Box 5a ordinary dividends
    /// Include the qualified portion entered below. Do not repeat these dividends in another income document. This is portfolio income, not corporate operating income or QBI.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub dividends: i64,
    /// Box 5b qualified dividends included in box 5a
    /// Only dividends eligible for qualified-dividend treatment on this return. Included once in ordinary dividends, with preferential rates determined by the return calculation.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub qualified_dividends: i64,
    /// Box 6 royalties
    /// Reported on Schedule E line 4. Box 12 code I deductions belong in the deductions list.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub royalties: i64,
    /// Optional portfolio capital amounts and suspended-loss records
    /// Omit only when there are no current capital amounts or prior basis/at-risk capital losses for this K-1. Other capital items require their separate worksheets.
    pub capital: Option<crate::ScorpCapital>,
    /// Optional business-property section 1231 records
    /// Omit only when box 9, business-asset box 8c and all related suspended losses are absent. These are separate from portfolio capital and ordinary operating amounts.
    pub section1231: Option<crate::K1Section1231>,
    /// Section 179 property the entity sold
    /// Each property on the fully taxable sale statement; use this tax system's depreciation. Do not include installment sales, exchanges, casualties or theft.
    /// ----
    /// Property
    #[libertas_unordered]
    pub section179_sales: Vec<crate::K1Section179Sale>,
    /// Rental activities — boxes 2 and 3
    pub rentals: crate::K1RentalAmounts,
    /// Your facts about the rental activities — boxes 2 and 3
    pub rental_facts: crate::K1RentalFacts,
    /// Box 13 code O backup withholding
    /// Your allocated federal backup withholding from this K-1. Do not repeat it on another income document or in estimated payments. This is a tax payment, not a business expense or QBI deduction.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub withholding: i64,
    /// Business activity for passive losses and investment tax
    pub trade_kind: ScorpTradeKind,
    /// Opening stock basis
    /// From the prior stock-basis worksheet; exclude shareholder debt.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub opening: i64,
    /// Capital contributions and additional stock basis
    /// Exclude loans; use adjusted tax basis rather than fair market value for contributed property.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub additions: i64,
    /// Box 16 code A tax-exempt interest
    /// Include specified private-activity bond interest reported separately below. Do not repeat this K-1 amount in an interest document. This increases shareholder basis and is reported on Form 1040 line 2a.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub exempt_interest: i64,
    /// Specified private-activity bond interest included above
    /// From the K-1 box 15 code F statement. Enter only the portion that is an AMT preference, already included in box 16 code A. Do not repeat it in another interest document or AMT adjustment.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub private_activity: i64,
    /// Box 16 code B other tax-exempt income
    /// Exclude cancellation-of-debt income excluded by the corporation.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub exempt: i64,
    /// Box 16 code D nondividend distributions
    /// Exclude dividends reported separately on Form 1099-DIV.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub distributions: i64,
    /// Box 16 code C nondeductible expenses
    /// Do not include deductible business expenses already reflected in box 1.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub nondeductible: i64,
    /// Prior ordinary business losses suspended by basis
    /// From Form 7203 column (e), ordinary business loss. Exclude losses already allowed through basis but suspended by at-risk or passive limits.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub prior_loss: i64,
    /// Prior nondeductible expense carryforward
    /// Only from a continuing losses-first election; otherwise enter zero.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub prior_nondeductible: i64,
    /// Continuing election to apply losses before nondeductible expenses
    /// Regulations section 1.1367-1(g). This records an election already effective for the corporation; choosing Yes here does not file an election.
    pub losses_first: bool,
    /// Shareholder loans
    /// ----
    /// Debt record
    pub loans: Vec<ScorpLoan>,
    /// At-risk history and financing worksheet
    /// From your 2025 Form 6198 for this activity (lines 19b and 21 and its carryovers) and the corporation's financing records; a shareholder who started in 2026 has a new history with no carryovers.
    pub risk: crate::ScorpRisk,
    /// Optional work and participation history
    /// Missing facts leave passive/nonpassive classification unresolved.
    pub participation: Option<crate::ScorpParticipation>,
    /// Prior passive-loss records
    /// From your 2025 Form 8582 worksheets for this activity: the ordinary loss carried forward because of the passive activity rules, zero when none was carried.
    pub passive: ScorpPassiveHistory,
    /// QBI character and loss-limit histories for ordinary losses
    /// This year's QBI loss portion comes from the section 199A statement; the histories from the QBI loss-tracking records kept with your 2025 return. Zero and empty without an ordinary loss or without a statement. A basis deduction is not automatically a QBI deduction.
    pub loss_qbi: crate::ScorpLossQbi,
    /// Section 1231 QBI statement and loss histories
    /// Separate from ordinary QBI; zero and empty without a section 1231 item or without a statement. These are regular-tax source facts; AMT uses the regular QBI deduction.
    pub property_qbi: crate::ScorpPropertyQbi,
    /// Current section 199A statement
    /// The K-1's box 17 code V statement: allocated QBI income, wage/property limits and service-business classification. Absent only when the K-1 came with no such statement; it then adds no qualified business income (Reg. 1.199A-6(b)(3)(iii)) and a loss is left out of the QBI deduction with a notice.
    pub qbi_statement: Option<crate::ScorpQbiStatement>,
    /// Optional separate AMT basis and loss records
    /// Missing records leave AMT unresolved; regular balances are never substituted.
    pub amt: Option<crate::ScorpAmt>,
    /// Box 12 code A cash contributions (60%)
    /// The corporation's cash gifts to charities, deducted on Schedule A like your own cash gifts. They reduce your stock basis; they are not a business deduction or QBI item.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub cash_charity: i64,
    /// Section 179 deduction — box 11
    /// Your share of the corporation's section 179 expense. It joins your own elections on Form 4562 Part I under your dollar and business-income limits and reduces your stock basis in full this year even when part is carried to 2027. Enter zero when the box is empty.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub section179: i64,
    /// Section 179 deduction from this K-1 you could not deduct in 2025
    /// Your 2025 Form 4562 line 13 carryover of this corporation's section 179 amounts. It joins Form 4562 line 10 without reducing your basis again. Enter zero when there is none.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub section179_carryover: i64,
    /// Part of this section 179 carryover belonging to property the entity sold
    /// From your own Form 4562 carryover records, not the K-1. This amount restores the sold property's basis and is no longer available as a deduction.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub section179_carryover_sold: i64,
    /// Foreign taxes paid or accrued — box 16 code F
    /// A Form 7203 Part III deduction item that reduces your stock basis. The credit is figured from the Schedule K-3. Enter zero when the code is absent.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub foreign_taxes: i64,
    /// Losses carried because of your basis or amount at risk — rentals, section 179, gifts and foreign taxes
    pub carried: crate::K1LossCarryforwards,
    /// Schedule K-3 — K-1 box 14
    /// Leave absent when box 14 is not checked.
    pub schedule_k3: Option<crate::ScheduleK3>,
    /// How long you have held your shares
    /// Needed only when box 16 code D distributions exceed your stock basis: the excess is a capital gain, long-term when you have held the shares more than one year. Leave absent otherwise.
    pub stock_term: Option<crate::GainTerm>,
    /// Other boxes on this K-1 with an amount
    /// ----
    /// Box with an amount
    /// General business credits — box 13
    /// Your share of the low-income housing, work opportunity, tip, small employer retirement plan, health insurance and paid family and medical leave credits. They reach Form 3800 Part III with this corporation's EIN; other box 13 codes belong among the other boxes.
    /// ----
    /// Credit
    pub credits: Vec<crate::K1Credit>,
    /// Investment credit property (Form 3468)
    pub investment_property: Vec<crate::K1InvestmentProperty>,
    /// Investment credit property confirmed
    pub investment_confirmed: Option<bool>,
    /// Low-income housing interest acquired after 1989 (Some(false) selects the pre-1990 rules)
    pub housing_confirmed: Option<bool>,
    pub unentered: Vec<ScorpK1Item>,
    /// Final K-1
    pub final_k1: bool,
    /// Other income — box 10
    /// ----
    /// Box 10 amount
    // Tax structure: one row per distinct Form 1120-S income code in the fixed catalog.
    #[libertas_size(max = 7)]
    pub income: Vec<crate::ScorpIncome>,
    /// Other deductions — box 12 codes other than A
    /// ----
    /// Box 12 amount
    // Tax structure: one row per distinct Form 1120-S deduction code in the fixed catalog.
    #[libertas_size(max = 20)]
    pub deductions: Vec<crate::ScorpDeduction>,
    /// What happened to the shares in 2026
    pub disposition: crate::K1Disposition,
    /// Situations this App does not calculate
    /// ----
    /// Situation
    // Tax structure: one row per distinct S-corporation special situation in the fixed catalog.
    #[libertas_size(max = 3)]
    pub special: Vec<crate::ScorpSpecial>,
    /// AMT section 179 income-limit carryover history
    /// From your own prior AMT Form 4562 records, independently of the regular carryover. Leave absent when unavailable; enter actual zero amounts when the AMT carryover is zero.
    /// [NullText]
    /// AMT carryover history is unavailable
    /// [SetText]
    /// Actual AMT carryover history, including an actual zero
    #[libertas_default($null)]
    pub amt_section179_income_carry: Option<crate::Section179IncomeCarryHistory>,
    /// Individual QBI aggregation group
    /// Leave absent when this source is not part of an individual aggregation election. The group election and annual disclosures are entered with income documents; the statement's printed amounts stay separate.
    /// [NullText]
    /// No individual QBI aggregation group is selected for this source
    /// [SetText]
    /// This source is in individual QBI aggregation group
    #[libertas_size(min = 1)]
    pub qbi_aggregation_group: Option<alloc::string::String>,
}

// Ordinary and section 1231 amounts must retain both tax character and
// passive/nonpassive origin for Form 4797 and the later NIIT calculation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ActivityCharacters {
    pub passive: [i64; 2],
    pub nonpassive: [i64; 2],
    pub prior_released: [i64; 2],
}

impl ActivityCharacters {
    // An elected activity has one section 469(f) income limit, while each
    // source retains its own loss and tax character. First apply the existing
    // character owner to the combined activity; then apportion each released
    // character among its actual source losses using the shared ratio owner.
    // Reg. 1.469-9(e)(1), (4); Reg. 1.469-1(f)(4); i8582 Part IX.
    pub(crate) fn grouped_former_passive(
        sources: &[([i64; 2], [i64; 2])],
        disposed: bool,
    ) -> Result<alloc::vec::Vec<Self>, (&'static str, &'static str)> {
        let mut current = [0; 2];
        let mut prior = [0; 2];
        for (source_current, source_prior) in sources {
            for character in 0..2 {
                current[character] += source_current[character];
                prior[character] += source_prior[character];
            }
        }
        let combined = Self::from_current_and_prior(current, prior, true, disposed)?;
        let released: alloc::vec::Vec<_> = (0..2)
            .map(|character| {
                crate::kernel::ratio::allocate_remaining(
                    combined.prior_released[character],
                    &sources
                        .iter()
                        .map(|(_, prior)| prior[character])
                        .collect::<alloc::vec::Vec<_>>(),
                )
            })
            .collect();
        Ok(sources
            .iter()
            .enumerate()
            .map(|(index, (current, prior))| {
                let prior_released = [released[0][index], released[1][index]];
                Self {
                    passive: [prior_released[0] - prior[0], prior_released[1] - prior[1]],
                    nonpassive: [current[0] - prior_released[0], current[1] - prior_released[1]],
                    prior_released,
                }
            })
            .collect())
    }

    pub(crate) fn from_current_and_prior(
        current: [i64; 2],
        prior: [i64; 2],
        nonpassive: bool,
        disposed: bool,
    ) -> Result<Self, (&'static str, &'static str)> {
        // Section 469(g)(1)(A): the whole interest disposed of in a fully
        // taxable sale releases every suspended loss of the activity.
        let nonpassive = nonpassive || disposed;
        if !nonpassive {
            return Ok(Self {
                passive: [current[0] - prior[0], current[1] - prior[1]],
                nonpassive: [0, 0],
                prior_released: [0, 0],
            });
        }
        // Section 469(f): only net current income of THIS activity releases
        // its old losses. Current nonpassive losses never enter Form 8582.
        // Part IX first offsets income against losses on the same form/part,
        // then apportions the unallowed balance using the residual net losses.
        // With two characters and opposite signs, current net income can
        // only remain in the positive character. Offset the current loss first;
        // it is fully nonpassive and cannot become a new passive suspension.
        // https://www.irs.gov/instructions/i8582 (former activities; Part IX)
        let total_prior = prior.iter().sum::<i64>();
        let released = if disposed {
            total_prior
        } else {
            phase::cap(phase::floor(current.iter().sum::<i64>(), 0), [total_prior])
        };
        let allocated = if released > 0 && released < total_prior && prior.iter().all(|v| *v > 0) {
            let income = if current[0] < 0 {
                [0, released]
            } else if current[1] < 0 {
                [released, 0]
            } else {
                current
            };
            let nets = [income[0] - prior[0], income[1] - prior[1]];
            let nets = nets.into_iter().enumerate().collect::<alloc::vec::Vec<_>>();
            let crate::kernel::source_fact::SourceFact::Reported(suspended) =
                crate::kernel::loss_limits::passive_characters(
                    crate::kernel::source_fact::SourceFact::Reported(
                        crate::kernel::loss_limits::CharacterInput {
                            nets: &nets,
                            suspended:
                                crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
                                    total_prior - released,
                                )
                                .expect("partial prior release leaves a positive suspension"),
                        },
                    ),
                )
                .map_err(|_| {
                    (
                        "ISSUE_SCORP_THE_FORMER_PASSIVE_CHARACTER_BALANCES_CANNOT_BE_B5669279",
                        crate::fixture_english!(
                            "The former-passive character balances cannot be reconciled."
                        ),
                    )
                })?
            else {
                unreachable!("reported characters remain reported")
            };
            alloc::vec![prior[0] - suspended[0].held(), prior[1] - suspended[1].held()]
        } else {
            crate::kernel::ratio::allocate_remaining(released, &prior)
        };
        let prior_released = [allocated[0], allocated[1]];
        Ok(Self {
            passive: [-(prior[0] - allocated[0]), -(prior[1] - allocated[1])],
            nonpassive: [current[0] - allocated[0], current[1] - allocated[1]],
            prior_released,
        })
    }
}

// Whole-dollar activity amounts after basis and at-risk limits.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ScorpActivity {
    /// Original annual-Dollar operands after basis/risk, before passive netting.
    /// Ordinary includes the same allowed activity items as `combined`.
    pub current_ordinary_with_items: i64,
    pub current_section1231: i64,
    // Net ordinary and section 1231 items entering the same passive activity.
    pub characters: Option<ActivityCharacters>,
    pub passive: i64,
    pub nonpassive: i64,
    pub prior_released: i64,
    // The prior unallowed loss that remains in the passive pool (Form 8582
    // Part V column (c)): the old loss less any released against this
    // activity's own income.
    pub prior_pool: i64,
    // What the basis and at-risk limits allowed and held back of this K-1.
    pub ledger: LossLedger,
    // Unit B9-1: the activity items (code S) after the limits, and the
    // activity's passive result with them (Reg. 1.469-2T(c)(2)); the same as
    // the fields above without any.
    pub items: crate::k1_items::ActivityItems,
    pub combined: ScorpCombined,
}

// The passive result of the S corporation activity with its code S items.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ScorpCombined {
    pub passive: i64,
    pub nonpassive: i64,
    pub prior_released: i64,
    pub prior_pool: i64,
    pub characters: Option<ActivityCharacters>,
}

// Computed property events can have a different community fraction from
// distributive-share income. This transient context never changes source facts.
#[derive(Clone, Copy)]
pub(crate) struct ScorpEvents {
    pub disposal: crate::k1_items::Disposal,
    pub loans: [i64; 3],
}

// Runtime PRINT projection only; stored debt facts and financial owners stay here.
mod projection;
pub(crate) use projection::RepaymentRow;

impl ScorpK1 {
    pub(crate) fn valid(&self) -> bool {
        self.valid_with_sale_carry(true, crate::scorp_amt::ScorpSourceCapacity::regular())
    }

    pub(crate) fn valid_with_sale_carry(
        &self,
        check_sale_carry: bool,
        capacity: crate::scorp_amt::ScorpSourceCapacity,
    ) -> bool {
        // Arithmetic safety: basis_result, pool_activity and k1_totals still
        // combine native i64 sources and transfer their dollars back to cents.
        let money = |v: i64| (0..=100000000000).contains(&v);
        self.health_insurance.valid()
            && crate::K1Credit::valid_list(&self.credits)
            && crate::K1InvestmentProperty::valid_list(&self.investment_property)
            && (self.investment_property.iter().all(|p| p.own_form().is_some())
                || self.investment_confirmed.is_some())
            && (!self.credits.iter().any(|credit| credit.code.housing())
                || self.housing_confirmed.is_some())
            && (-capacity.limit()..=capacity.limit()).contains(&self.ordinary)
            && [
                self.withholding,
                self.interest,
                self.dividends,
                self.qualified_dividends,
                self.royalties,
                self.opening,
                self.additions,
                self.exempt,
                self.exempt_interest,
                self.private_activity,
                self.distributions,
                self.nondeductible,
                self.prior_loss,
                self.prior_nondeductible,
            ]
            .into_iter()
            .all(money)
            && self.capital.as_ref().is_none_or(|v| v.valid())
            && self.section1231.as_ref().is_none_or(|v| v.valid())
            && crate::K1Section179Sale::valid_source_list(&self.section179_sales)
            && (!check_sale_carry
                || crate::K1Section179Sale::carryover_rule(
                    &self.section179_sales,
                    self.section179_carryover,
                    self.section179_carryover_sold,
                )
                .is_none())
            && self.rentals.valid()
            && self.rental_facts.valid()
            && self.qualified_dividends <= self.dividends
            && self.private_activity <= self.exempt_interest
            && (self.losses_first || self.prior_nondeductible == 0)
            && money(self.cash_charity)
            && money(self.section179)
            && money(self.section179_carryover)
            && money(self.section179_carryover_sold)
            && self
                .amt_section179_income_carry
                .as_ref()
                .is_none_or(|history| history.rule().is_none())
            && money(self.foreign_taxes)
            && self.carried.valid()
            && self.schedule_k3.as_ref().is_none_or(crate::ScheduleK3::valid)
            && crate::ScorpIncome::valid_list(&self.income)
            && crate::ScorpDeduction::valid_list(&self.deductions)
            && crate::ScorpSpecial::valid_list(&self.special)
            && self.disposition.valid()
            && ScorpLoan::sources_valid(self.loans.iter())
            && self.amt.as_ref().is_none_or(|v| v.valid(self))
            && self.qbi_statement.as_ref().is_none_or(|v| v.valid(self))
            && self.loss_qbi.valid_retained()
            && self.property_qbi.valid_retained()
            && (self.qbi_statement.is_some()
                || self.loss_qbi.supplied_histories_valid([
                    self.prior_loss,
                    self.risk.prior_loss,
                    self.passive.loss,
                ]) && self.section1231.as_ref().is_none_or(|property| {
                    self.property_qbi.losses.supplied_histories_valid([
                        property.basis_loss,
                        property.risk_loss,
                        property.passive_loss,
                    ])
                }))
            && if self.loss_tracking() {
                let source = self.qbi_projection();
                source.loss_qbi.valid(&source)
            } else {
                true
            }
            && if self.property_tracking() {
                let source = self.qbi_projection();
                source.property_qbi.valid(&source)
            } else {
                true
            }
            && self.risk.valid(self)
            && self.participation.as_ref().is_none_or(|v| v.valid())
            && (0..=100000000000).contains(&self.passive.loss)
    }

    // Section 1.469-1T(e)(6): trading for owners' accounts is nonpassive
    // independently of material participation. Keep actual participation facts
    // separate for rules that require them, including the minimum QBI deduction.
    pub(crate) fn nonpassive(&self) -> crate::participation::Participation {
        let material = self
            .participation
            .as_ref()
            .map(|p| p.material())
            .unwrap_or(crate::participation::Participation::Unavailable);
        match self.trade_kind {
            ScorpTradeKind::FinancialTrading => crate::participation::Participation::Yes,
            ScorpTradeKind::Ordinary => material,
        }
    }

    // Section 163(d)(5)(A)(ii): a nonpassive business interest is investment
    // property when its owner does not materially participate. This is not NIIT.
    pub(crate) fn investment_interest_eligible(
        &self,
    ) -> Result<bool, (&'static str, &'static str)> {
        let material = self
            .participation
            .as_ref()
            .map(|p| p.material())
            .unwrap_or(crate::participation::Participation::Unavailable);
        if self.trade_kind == ScorpTradeKind::Ordinary
            || material == crate::participation::Participation::Yes
        {
            return Ok(false);
        }
        if material == crate::participation::Participation::Unavailable {
            return Err((
                "ISSUE_SCORP_RESOLVE_TRADING_BUSINESS_CLASSIFICATION_AND_MATERIAL_PARTICIPA_EDFB91CC",
                crate::fixture_english!(
                    "Resolve trading-business classification and material participation for the investment-interest income limit."
                ),
            ));
        }
        Ok(true)
    }

    pub(crate) fn investment_interest_income(&self) -> Result<i64, (&'static str, &'static str)> {
        crate::scorp_amt::ScorpSource::from(self).investment_interest_income()
    }
    pub(crate) fn investment_interest_income_in(
        &self,
        capacity: crate::scorp_amt::ScorpSourceCapacity,
    ) -> Result<i64, (&'static str, &'static str)> {
        if !self.investment_interest_eligible()? {
            return Ok(0);
        }
        let activity = self.pool_activity_in(None, capacity)?;
        // An unreleased old loss is not a current investment expense. Old
        // losses used against this business still need expense attribution.
        if activity.prior_released != 0 {
            return Err((
                "ISSUE_SCORP_TRADING_INCOME_WITH_RELEASED_PRIOR_PASSIVE_8CD8F8E2",
                crate::fixture_english!(
                    "Trading income with released prior passive losses needs separate investment-interest expense allocation."
                ),
            ));
        }
        Ok(crate::kernel::ratio::dollars(
            activity.characters.map_or(activity.nonpassive, |parts| parts.nonpassive[0]),
        ))
    }

    pub(crate) fn activity_result(&self) -> Result<ScorpActivity, (&'static str, &'static str)> {
        crate::scorp_amt::ScorpSource::from(self).activity_result()
    }
    pub(crate) fn activity_result_in(
        &self,
        capacity: crate::scorp_amt::ScorpSourceCapacity,
    ) -> Result<ScorpActivity, (&'static str, &'static str)> {
        if self.business_property()?.is_some() {
            return Err((
                "ISSUE_SCORP_SECTION_1231_BUSINESS_PROPERTY_ITEMS_NEED_COORDINATED_2B9506E8",
                crate::fixture_english!(
                    "Section 1231 business-property items need coordinated passive/QBI treatment and final return integration before ordinary activity results are final."
                ),
            ));
        }
        self.pool_activity_in(None, capacity)
    }

    pub(crate) fn pool_activity_result(
        &self,
    ) -> Result<ScorpActivity, (&'static str, &'static str)> {
        self.pool_activity_with_events(None)
    }

    pub(crate) fn pool_activity_with_events(
        &self,
        events: Option<&ScorpEvents>,
    ) -> Result<ScorpActivity, (&'static str, &'static str)> {
        crate::scorp_amt::ScorpSource::from(self).pool_activity_with_events(events)
    }
    pub(crate) fn pool_activity_in(
        &self,
        events: Option<&ScorpEvents>,
        capacity: crate::scorp_amt::ScorpSourceCapacity,
    ) -> Result<ScorpActivity, (&'static str, &'static str)> {
        let classification = self.nonpassive();
        if classification == crate::participation::Participation::Unavailable {
            return Err((
                "ISSUE_SCORP_COMPLETE_THE_BUSINESS_CLASSIFICATION_AND_APPLICABLE_05C04B0A",
                crate::fixture_english!(
                    "Complete the business classification and applicable participation facts before applying passive losses."
                ),
            ));
        }
        let history = Some(&self.passive).filter(|v| v.complete).ok_or((
            "ISSUE_SCORP_COMPLETE_THE_PRIOR_PASSIVE_LOSS_HISTORY_AND_EEFE19D3",
            crate::fixture_english!(
                "Complete the prior passive-loss history and ordinary-pool eligibility."
            ),
        ))?;
        let basis = self.basis_result_in(capacity)?;
        let risk = self
            .risk
            .calculate_in(self, &basis, events, capacity)
            .map_err(|error| error.message())?;
        if risk.recapture.income.get() != 0 {
            return Err((
                "ISSUE_SCORP_AT_RISK_RECAPTURE_NEEDS_COORDINATED_PASSIVE_INCOME_AND_A32538D4",
                crate::fixture_english!(
                    "At-risk recapture needs coordinated passive-income and QBI treatment plus separate next-year deduction records before return totals can be calculated."
                ),
            ));
        }
        // Portfolio income can enter the same at-risk activity's loss limit,
        // but cannot become passive operating income or release passive losses.
        let current =
            phase::floor(self.ordinary, 0) - risk.ledger.risk_allowed[K1LossClass::Ordinary].get();
        let prior = history.loss;
        let disposed = self.disposition.ended();
        let items = self.k1_items().flows(&risk.ledger).activity();
        let nonpassive = classification == crate::participation::Participation::Yes;
        let rental_property_losses =
            self.rentals.property_allowed(&risk.ledger)?.iter().sum::<i64>();
        if let Some(property) = self.business_property()? {
            let current_1231 = property.gain
                - (risk.ledger.risk_allowed[K1LossClass::Section1231].get()
                    - rental_property_losses);
            let prior_1231 = property.passive_loss;
            let characters = ActivityCharacters::from_current_and_prior(
                [current, current_1231],
                [prior, prior_1231],
                nonpassive,
                disposed,
            )?;
            let current_ordinary_with_items = current + items.net();
            let with_items = ActivityCharacters::from_current_and_prior(
                [current_ordinary_with_items, current_1231],
                [prior, prior_1231],
                nonpassive,
                disposed,
            )?;
            let prior_released: i64 = characters.prior_released.iter().sum();
            let released_with_items: i64 = with_items.prior_released.iter().sum();
            return Ok(ScorpActivity {
                current_ordinary_with_items,
                current_section1231: current_1231,
                passive: characters.passive.iter().sum(),
                nonpassive: characters.nonpassive.iter().sum(),
                prior_released,
                prior_pool: prior + prior_1231 - prior_released,
                ledger: risk.ledger,
                characters: Some(characters),
                items,
                combined: ScorpCombined {
                    passive: with_items.passive.iter().sum(),
                    nonpassive: with_items.nonpassive.iter().sum(),
                    prior_released: released_with_items,
                    prior_pool: prior + prior_1231 - released_with_items,
                    characters: Some(with_items),
                },
            });
        }
        // Form 8582, Former Passive Activities: absorb the old loss with this
        // activity's income first. Only the remainder enters the common pool.
        // A sale of all the shares releases it altogether.
        let pooled =
            |net: i64| crate::partnership::PooledNets::of(net, prior, nonpassive, disposed);
        let plain = pooled(current);
        let current_ordinary_with_items = current + items.net();
        let with_items = pooled(current_ordinary_with_items);
        Ok(ScorpActivity {
            current_ordinary_with_items,
            current_section1231: 0,
            characters: None,
            passive: plain.passive,
            nonpassive: plain.nonpassive,
            prior_released: plain.prior_released,
            prior_pool: plain.prior_pool,
            ledger: risk.ledger,
            items,
            combined: ScorpCombined {
                passive: with_items.passive,
                nonpassive: with_items.nonpassive,
                prior_released: with_items.prior_released,
                prior_pool: with_items.prior_pool,
                characters: None,
            },
        })
    }

    // The box 2 and 3 rental activities of this tax system's record, after
    // its basis and at-risk limits have allowed them (the ledger of this
    // K-1 in the same tax system).
    pub(crate) fn rental_activities(
        &self,
        ledger: &LossLedger,
        professional_status: bool,
        group_disposed: bool,
    ) -> Result<alloc::vec::Vec<crate::k1_rental::RentalActivity>, (&'static str, &'static str)>
    {
        crate::k1_rental::activities(
            &self.rentals,
            &self.rental_facts,
            ledger,
            false,
            professional_status,
            self.disposition.ended(),
            group_disposed,
        )
    }

    pub(crate) fn portfolio_income(&self) -> i64 {
        // Qualified dividends are a subset, not additional basis or income.
        self.interest + self.dividends + self.royalties
    }

    // A missing current statement excludes current reported QBI, not the
    // original-year qualification of an actual suspended loss (i8995).
    // Unrelated retained histories remain structurally valid but inactive.
    pub(crate) fn loss_tracking(&self) -> bool {
        self.qbi_statement.is_some()
            || !self.loss_qbi.missing_history([
                self.prior_loss,
                self.risk.prior_loss,
                self.passive.loss,
            ]) && [self.prior_loss, self.risk.prior_loss, self.passive.loss]
                .into_iter()
                .zip([&self.loss_qbi.history, &self.loss_qbi.at_risk, &self.loss_qbi.passive])
                .any(|(balance, history)| balance > 0 && !history.is_empty())
    }
    pub(crate) fn property_tracking(&self) -> bool {
        self.qbi_statement.is_some()
            || self.section1231.as_ref().is_some_and(|property| {
                !self.property_qbi.losses.missing_history([
                    property.basis_loss,
                    property.risk_loss,
                    property.passive_loss,
                ]) && [property.basis_loss, property.risk_loss, property.passive_loss]
                    .into_iter()
                    .zip([
                        &self.property_qbi.losses.history,
                        &self.property_qbi.losses.at_risk,
                        &self.property_qbi.losses.passive,
                    ])
                    .any(|(balance, history)| balance > 0 && !history.is_empty())
            })
    }

    // Project only QBI facts. The ordinary tax source and accepted personal
    // histories stay intact. Without a statement, current qualification is
    // unknown and omitted; prior qualified losses keep their original status.
    pub(crate) fn qbi_projection(&self) -> Cow<'_, Self> {
        if self.qbi_statement.is_some() {
            return Cow::Borrowed(self);
        }
        let mut source = self.clone();
        let retain = |records: &mut crate::ScorpLossQbi, balances: [i64; 3]| {
            records.current = 0;
            for (history, balance) in
                [&mut records.history, &mut records.at_risk, &mut records.passive]
                    .into_iter()
                    .zip(balances)
            {
                if balance == 0 {
                    history.clear();
                }
            }
        };
        retain(&mut source.loss_qbi, [self.prior_loss, self.risk.prior_loss, self.passive.loss]);
        let property_balances = self.section1231.as_ref().map_or([0; 3], |property| {
            [property.basis_loss, property.risk_loss, property.passive_loss]
        });
        retain(&mut source.property_qbi.losses, property_balances);
        source.property_qbi.gain = 0;
        source.property_qbi.qualified_1250 = None;
        // This confirmation applies to the missing current statement, not to
        // the separately validated original-year suspended-loss worksheets.
        source.property_qbi.complete = true;
        Cow::Owned(source)
    }
    // Loss tracking with something to report, unless a current loss with a
    // statement reports it with the statement's QBI (federal_rules).
    pub(crate) fn loss_reported_with_basis(&self) -> bool {
        self.loss_tracking()
            && self.qbi_left_out().is_none()
            && (!self.loss_qbi.is_empty() || self.has_ordinary_loss())
            && (self.qbi_statement.is_none() || self.ordinary >= 0)
    }

    // A current or carried ordinary loss: with a section 199A statement it
    // needs the QBI loss tracking (SCORP_QBI_LOSS_RECORDS_NEEDED).
    pub(crate) fn has_ordinary_loss(&self) -> bool {
        self.ordinary < 0
            || self.prior_loss > 0
            || self.risk.prior_loss > 0
            || self.passive.loss > 0
    }

    // The box 12 code A gift the stock and debt basis allows, in cents: this
    // year's gift plus a released carry, less what the limit holds back. A
    // K-1 whose limits cannot be figured is refused elsewhere and counts its
    // box.
    pub(crate) fn allowed_cash_charity(&self) -> i64 {
        if self.cash_charity == 0 && self.carried.charity == 0 {
            return 0;
        }
        match self.ledger() {
            Ok(ledger) => ledger.basis_allowed[K1LossClass::Charity].get(),
            Err(_) => self.cash_charity,
        }
    }

    // What the stock and debt basis and the at-risk limit allow and hold
    // back of this K-1 in this tax system, without the passive stage.
    pub(crate) fn ledger(&self) -> Result<LossLedger, (&'static str, &'static str)> {
        crate::scorp_amt::ScorpSource::from(self).ledger()
    }
    pub(crate) fn ledger_in(
        &self,
        events: Option<&ScorpEvents>,
        capacity: crate::scorp_amt::ScorpSourceCapacity,
    ) -> Result<LossLedger, (&'static str, &'static str)> {
        self.risk_in(events, capacity).map(|value| value.ledger)
    }

    // The K-1's Form 7203 inputs, built once: stock basis before the debt
    // stage (the shareholder loans are separate inputs, `shareholder_loans`).
    fn stock_basis<'a>(&self, losses: &'a crate::k1_loss::LossAmounts) -> StockBasis<'a> {
        StockBasis {
            opening: Amount::<Cents>::new(self.opening).expect("bounded basis result"),
            additions: Amount::<Cents>::new(self.additions).expect("bounded basis result"),
            income: Amount::<Cents>::new(
                phase::floor(self.ordinary, 0)
                    + self.portfolio_income()
                    + self.capital_gains()
                    + self.section1231_gain()
                    + self.rentals.income()
                    + self.k1_items().income()
                    + self.exempt
                    + self.exempt_interest,
            )
            .expect("bounded basis result"),
            distributions: Amount::<Cents>::new(self.distributions).expect("bounded basis result"),
            nondeductible: Amount::<Cents>::new(self.nondeductible).expect("bounded basis result"),
            credit_reduction: Amount::<Cents>::new(crate::investment_credit::k1_basis_decrease(
                &self.investment_property,
            ))
            .expect("bounded basis result"),
            prior_nondeductible: Amount::<Cents>::new(self.prior_nondeductible)
                .expect("bounded basis result"),
            losses_first: self.losses_first,
            losses,
        }
    }

    fn shareholder_loans(&self) -> Vec<ShareholderLoan> {
        self.loans
            .iter()
            .map(|l| ShareholderLoan {
                kind: match l.kind {
                    ScorpLoanKind::FormalNote => LoanKind::FormalNote,
                    ScorpLoanKind::OpenAccount => LoanKind::OpenAccount,
                    ScorpLoanKind::FormerOpenAccount => LoanKind::FormerOpenAccount,
                },
                principal: Amount::<Cents>::new(l.principal).expect("bounded basis result"),
                basis: Amount::<Cents>::new(l.basis).expect("bounded basis result"),
                advances: Amount::<Cents>::new(l.advances).expect("bounded basis result"),
                repayments: Amount::<Cents>::new(l.repayments).expect("bounded basis result"),
            })
            .collect()
    }

    pub(crate) fn basis_result(&self) -> Result<BasisResult, (&'static str, &'static str)> {
        crate::scorp_amt::ScorpSource::from(self).basis_result()
    }
    pub(crate) fn basis_result_in(
        &self,
        capacity: crate::scorp_amt::ScorpSourceCapacity,
    ) -> Result<BasisResult, (&'static str, &'static str)> {
        if !self.valid_with_sale_carry(true, capacity) {
            return Err(inconsistent());
        }
        self.sale_results()?;
        if !self.basis_complete() {
            return Err((
                "ISSUE_SCORP_CONFIRM_COMPLETE_NONFARMING_BUSINESS_AND_BASIS_F667C1A3",
                crate::fixture_english!(
                    "Confirm complete nonfarming business and basis facts; farming, additional or unknown K-1 items need their separate worksheets."
                ),
            ));
        }
        let loans = self.shareholder_loans();
        let losses = self.class_input().basis();
        let stock = self.stock_basis(&losses);
        let basis = stock.with_loans(&loans).map_err(|e| match e {
            BasisError::Overflow => ("ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26", crate::fixture_english!("Review the document amounts; they must be within the supported input range.")),
            BasisError::Invalid => ("ISSUE_SCORP_THE_STOCK_DEBT_BASIS_RECORDS_ARE_INCONSISTENT_9799BBBB", crate::fixture_english!("The stock/debt basis records are inconsistent.")),
            BasisError::RepaymentPriority => ("ISSUE_SCORP_MULTIPLE_REPAID_LOANS_NEED_AN_ADDITIONAL_003A1C12", crate::fixture_english!("Multiple repaid loans need an additional restoration-priority worksheet before their basis can be calculated.")),
            BasisError::AtRiskRecapture => ("ISSUE_SCORP_AT_RISK_RECAPTURE_REQUIRES_ITS_SEPARATE_WORKSHEET_DE430CD8", crate::fixture_english!("At-risk recapture requires its separate worksheet.")),
        })?;
        Ok(basis)
    }

    // The K-1's Schedule E line 28 amount after the basis, at-risk and the
    // activity's own prior passive loss, in cents: the Schedule K-1 page's
    // "Amount on the return" (T2-31p). None while the K-1 is refused or its
    // amount depends on the rest of the return: a section 179 share or
    // carryover (Form 4562 limits the whole return), a net passive loss
    // (Form 8582), or section 1231 items (Form 4797 netting).
    pub(crate) fn schedule_e_amount(&self) -> Option<i64> {
        // Rental activities depend on Form 8582 and the recorded real estate
        // professional status.
        if !self.basis_complete()
            || self.rentals.any()
            || !self.k1_items().is_empty()
            || self.disposition != crate::K1Disposition::Kept
            || self.carried.waits_for_return_forms()
            || !self.unentered.is_empty()
            || self.section179 != 0
            || self.section179_carryover != 0
            || self.amt.is_none()
        {
            return None;
        }
        let basis = self.basis_result().ok()?;
        self.loan_gains(&basis).ok()?;
        let activity = self.activity_result().ok()?;
        (activity.passive >= 0).then_some(activity.passive + activity.nonpassive)
    }

    // Whether the activity items are in the passive pool: the activity is
    // passive and the whole interest was not disposed of (section 469(g)).
    pub(crate) fn items_in_pool(&self) -> bool {
        self.nonpassive() != crate::participation::Participation::Yes && !self.disposition.ended()
    }

    // The box 10 and box 12 codes of this K-1.
    pub(crate) fn k1_items(&self) -> crate::k1_items::K1Items {
        let items = crate::k1_items::K1Items::scorp(&self.income, &self.deductions);
        match self.sale_results() {
            Ok(sales) => items.with_sales(&sales),
            Err(_) => items,
        }
    }

    pub(crate) fn sale_results(
        &self,
    ) -> Result<Vec<crate::k1_items::Section179SaleResult>, (&'static str, &'static str)> {
        crate::K1Section179Sale::worksheets(
            &crate::K1Section179Sale::views(&self.section179_sales, false)?,
            self.section179_carryover,
            self.section179_carryover_sold,
        )
    }

    pub(crate) fn business_property(
        &self,
    ) -> Result<Option<crate::k1_items::BusinessProperty>, (&'static str, &'static str)> {
        Ok(crate::k1_items::BusinessProperty::of(self.section1231.as_ref(), &self.sale_results()?))
    }

    // What the codes and the year-end events amount to after the stock and
    // debt basis and the at-risk limit; a K-1 whose limits cannot be figured
    // is refused elsewhere and counts its boxes whole.
    pub(crate) fn outcome(&self) -> (crate::k1_items::K1ItemFlows, crate::k1_items::Disposal) {
        self.outcome_with_events(None)
    }
    pub(crate) fn outcome_with_events(
        &self,
        events: Option<&ScorpEvents>,
    ) -> (crate::k1_items::K1ItemFlows, crate::k1_items::Disposal) {
        crate::scorp_amt::ScorpSource::from(self).outcome_with_events(events)
    }
    pub(crate) fn outcome_in(
        &self,
        events: Option<&ScorpEvents>,
        capacity: crate::scorp_amt::ScorpSourceCapacity,
    ) -> (crate::k1_items::K1ItemFlows, crate::k1_items::Disposal) {
        let limited = self.basis_result_in(capacity).ok().and_then(|basis| {
            let ledger = self.risk.calculate_in(self, &basis, events, capacity).ok()?.ledger;
            Some((
                self.k1_items().flows(&ledger),
                events.map_or_else(|| self.disposal(&basis), |value| value.disposal),
            ))
        });
        limited.unwrap_or_else(|| {
            (self.k1_items().flows_whole(), crate::k1_items::Disposal::default())
        })
    }

    // The Schedule K-1 facts of the codes in the return's totals.
    pub(crate) fn totals_facts<'a>(
        &self,
        entry: &'a crate::FederalIncomeEntry,
    ) -> crate::k1_totals::Facts<'a> {
        let (flows, disposal) = self.outcome();
        let (amt_flows, amt_disposal) = self
            .amt
            .as_ref()
            .and_then(|amt| amt.source(self).ok())
            .map_or((flows.clone(), disposal), |source| source.outcome());
        let material = self
            .participation
            .as_ref()
            .map_or(crate::participation::Participation::Unavailable, |p| p.material());
        let (nii_activity, investment_activity) =
            crate::k1_totals::classify(self.trade_kind, material);
        crate::k1_totals::Facts {
            entry,
            partner: false,
            flows,
            amt_flows,
            disposal,
            amt_disposal,
            nii_activity,
            investment_activity,
            term: self.stock_term,
            collectibles: 0,
            unrecaptured: 0,
            reit_dividends: self.qbi_statement.as_ref().map_or(0, |v| v.reit_dividends),
            deferred_tax: 0,
            source: "https://www.irs.gov/instructions/i1120ssk",
            k3: self.schedule_k3.is_some(),
        }
    }

    // No situation that leaves the whole K-1 out of the estimate (answers and
    // unknowns, decision 2).
    pub(crate) fn basis_complete(&self) -> bool {
        !self.special.iter().any(|situation| situation.whole())
    }

    // The gain of the year-end events in this tax system (unit B9-1): Form
    // 7203 Part I line 6, distributions above the stock basis before
    // distributions (section 1368(b)(2)), and the sale of all the shares at
    // the Form 7203 ending stock basis (Reg. 1.1367-1(d)(1)).
    pub(crate) fn disposal(&self, basis: &BasisResult) -> crate::k1_items::Disposal {
        let mut disposal = crate::k1_items::Disposal {
            distribution_gain: basis.distribution_gain.get(),
            ..Default::default()
        };
        if let crate::K1Disposition::Sold { proceeds } = &self.disposition {
            disposal.sold = true;
            disposal.proceeds = *proceeds;
            disposal.basis = basis.stock.get();
        }
        disposal
    }

    // Actual-cent [ordinary, short-term, long-term] repayment gains. Cash
    // flows and holding periods are shared by regular and AMT; basis is not.
    pub(crate) fn loan_gains(
        &self,
        basis: &BasisResult,
    ) -> Result<[i64; 3], (&'static str, &'static str)> {
        if basis.loans.len() != self.loans.len() {
            return Err((
                "ISSUE_SCORP_LOAN_BASIS_SOURCE_MISMATCH",
                crate::fixture_english!(
                    "The shareholder-loan basis results do not match the source records."
                ),
            ));
        }
        let mut gains = [0; 3];
        for (source, computed) in self.loans.iter().zip(&basis.loans) {
            gains[0] += computed.ordinary_gain.get();
            if computed.capital_gain.get() != 0 {
                let [short, long] =
                    source.repayment_term.split(source, computed.capital_gain.get())?;
                gains[1] += short;
                gains[2] += long;
            }
        }
        Ok(gains)
    }

    pub(crate) fn report(&self, id: i64, result: &mut FederalResult) {
        if let Some(facts) = &self.participation {
            let (_label, key) = match facts.material() {
                crate::participation::Participation::Yes => {
                    ("established", "NOTE_SCORP_MATERIAL_ESTABLISHED")
                }
                crate::participation::Participation::No => (
                    "not established by the applicable tests",
                    "NOTE_SCORP_MATERIAL_NOT_ESTABLISHED",
                ),
                crate::participation::Participation::Unavailable => (
                    "unresolved; complete facts or significant-participation aggregation are needed",
                    "NOTE_SCORP_MATERIAL_UNRESOLVED",
                ),
            };
            #[cfg(test)]
            let label = _label;

            result.review_notes.push(crate::federal_model::ReviewNote::localized(
                #[cfg(test)] alloc::format!("S-corporation document {id}, material participation: {label}. This classifies ordinary business participation only; passive-loss treatment, investment-tax classification and QBI eligibility are evaluated separately."),
                crate::Message::new(key, alloc::vec![crate::MessageArg::Number(id)]),
            ));
        }

        let basis = match self.basis_result() {
            Ok(v) => v,
            Err((key, _message)) => {
                #[cfg(test)]
                let message = _message;

                crate::issue_localized!(
                    result,
                    &crate::IssueContext::income_id(id),
                    "S-corporation basis",
                    key,
                    message,
                );
                return;
            }
        };
        let mut report = |#[cfg(test)] _suffix: &str,
                          amount: i64,
                          #[cfg(test)] _explanation: &str,
                          key: &'static str,
                          extra: alloc::vec::Vec<crate::MessageArg>| {
            let mut args =
                alloc::vec![crate::MessageArg::Number(id), crate::MessageArg::usd_cents(amount),];
            args.extend(extra);
            let message = crate::Message::new(key, args);
            result.review_notes.push(crate::federal_model::ReviewNote::localized(
                #[cfg(test)]
                message.english(),
                message,
            ));
        };
        report(
            #[cfg(test)]
            "stock_basis",
            basis.stock.get(),
            #[cfg(test)]
            "Ending stock basis for the confirmed ordinary-business worksheet; separate AMT and other basis adjustments remain outside this worksheet.",
            "NOTE_SCORP_STOCK_BASIS",
            alloc::vec![],
        );
        report(
            #[cfg(test)]
            "distribution_gain",
            basis.distribution_gain.get(),
            #[cfg(test)]
            "Nondividend distributions exceeding stock basis, capital gain on Form 8949 and Schedule D with the holding period of the shares.",
            "NOTE_SCORP_DISTRIBUTION_GAIN",
            alloc::vec![],
        );
        report(
            #[cfg(test)]
            "basis_allowed_loss",
            basis.allowed[K1LossClass::Ordinary].get(),
            #[cfg(test)]
            "Ordinary loss passing basis limits only; subsequent limitations may disallow it.",
            "NOTE_SCORP_BASIS_ALLOWED_LOSS",
            alloc::vec![],
        );
        report(
            #[cfg(test)]
            "basis_suspended_loss",
            basis.suspended[K1LossClass::Ordinary].get(),
            #[cfg(test)]
            "Ordinary loss still suspended by basis; do not combine with at-risk or passive suspended balances.",
            "NOTE_SCORP_BASIS_SUSPENDED_LOSS",
            alloc::vec![],
        );
        report(
            #[cfg(test)]
            "nondeductible_carry",
            basis.nondeductible_carry.get(),
            #[cfg(test)]
            "Unused nondeductibles carried only under the continuing losses-first election.",
            "NOTE_SCORP_NONDEDUCTIBLE_CARRY",
            alloc::vec![],
        );
        for (index, loan) in basis.loans.iter().enumerate() {
            let loan_args = alloc::vec![crate::MessageArg::Number((index + 1) as i64)];
            let (_principal_explanation, principal_key) = match loan.kind_next_year {
                LoanKind::FormalNote => (
                    "Next year: retain this separate written note and its adjusted debt basis.",
                    "NOTE_SCORP_LOAN_PRINCIPAL_FORMAL",
                ),
                LoanKind::OpenAccount => (
                    "Next year: retain this current open account; annual advance/repayment netting applies.",
                    "NOTE_SCORP_LOAN_PRINCIPAL_OPEN",
                ),
                LoanKind::FormerOpenAccount => (
                    "Next year: retain this as a separate former open account that exceeded $25,000. Do not combine it with a new open account, even if its balance is now lower. Annual netting no longer applies; repayment gain remains ordinary.",
                    "NOTE_SCORP_LOAN_PRINCIPAL_FORMER_OPEN",
                ),
            };
            #[cfg(test)]
            let principal_explanation = _principal_explanation;

            let mut principal_args = loan_args.clone();
            if matches!(loan.kind_next_year, LoanKind::FormerOpenAccount) {
                principal_args.push(crate::MessageArg::usd_dollars(25_000));
            }
            report(
                #[cfg(test)]
                &alloc::format!("loan_{}_principal", index + 1),
                loan.principal.get(),
                #[cfg(test)]
                principal_explanation,
                principal_key,
                principal_args,
            );
            report(
                #[cfg(test)]
                &alloc::format!("loan_{}_nontaxable_repayment", index + 1),
                loan.nontaxable_repayment.get(),
                #[cfg(test)]
                "Principal recovered without taxable gain; interest is separate.",
                "NOTE_SCORP_LOAN_NONTAXABLE_REPAYMENT",
                loan_args.clone(),
            );
            report(
                #[cfg(test)]
                &alloc::format!("loan_{}_basis", index + 1),
                loan.basis.get(),
                #[cfg(test)]
                "Ending basis of this debt, kept separately from stock and other loans.",
                "NOTE_SCORP_LOAN_BASIS",
                loan_args.clone(),
            );
            report(
                #[cfg(test)]
                &alloc::format!("loan_{}_restoration", index + 1),
                loan.restoration.get(),
                #[cfg(test)]
                "Income used to restore this previously reduced debt basis.",
                "NOTE_SCORP_LOAN_RESTORATION",
                loan_args.clone(),
            );
            let (_gain_explanation, gain_key) = if self.loans[index].kind
                == ScorpLoanKind::FormalNote
            {
                match &self.loans[index].repayment_term {
                    ScorpNoteHolding::Single { term: crate::GainTerm::Short } => (
                        "Short-term personal-investment gain on Form 8949 and Schedule D, separate from corporate operating income and QBI.",
                        "NOTE_SCORP_LOAN_GAIN_SHORT",
                    ),
                    ScorpNoteHolding::Single { term: crate::GainTerm::Long } => (
                        "Long-term personal-investment gain on Form 8949 and Schedule D, separate from corporate operating income and QBI.",
                        "NOTE_SCORP_LOAN_GAIN_LONG",
                    ),
                    ScorpNoteHolding::Dated { .. } => (
                        "Written-note repayment gain; dated payments allocate the supported capital gain between short- and long-term Form 8949/Schedule D amounts.",
                        "NOTE_SCORP_LOAN_GAIN_DATED",
                    ),
                    ScorpNoteHolding::NoRepayment => (
                        "No written-note principal was repaid this year, so the note has no repayment gain.",
                        "NOTE_SCORP_LOAN_GAIN_NO_REPAYMENT",
                    ),
                }
            } else {
                (
                    "Open-account repayment gain on Form 4797 and Schedule 1 line 4; personal-investment income, not corporate operating income or QBI.",
                    "NOTE_SCORP_LOAN_GAIN_OPEN",
                )
            };
            #[cfg(test)]
            let gain_explanation = _gain_explanation;

            report(
                #[cfg(test)]
                &alloc::format!("loan_{}_gain", index + 1),
                loan.capital_gain.get() + loan.ordinary_gain.get(),
                #[cfg(test)]
                gain_explanation,
                gain_key,
                loan_args,
            );
        }
        // A current loss with a statement reports its tracking with the
        // statement's QBI (federal_rules).
        if self.loss_reported_with_basis() {
            let source = self.qbi_projection();
            source.loss_qbi.report(&source, &basis, id, result);
        }
        self.risk.report(self, &basis, id, result);
    }
}

pub(crate) fn inconsistent() -> (&'static str, &'static str) {
    (
        "ISSUE_SCORP_THE_STOCK_DEBT_BASIS_RECORDS_ARE_INCONSISTENT_9799BBBB",
        crate::fixture_english!("The stock/debt basis records are inconsistent."),
    )
}

// Community-property allocation of this source record; facts retain their meaning.
crate::community::divide_record!(ScorpK1 {
    amounts: [
        ordinary,
        interest,
        dividends,
        qualified_dividends,
        royalties,
        capital,
        section1231,
        section179_sales,
        rentals,
        rental_facts,
        withholding,
        opening,
        additions,
        exempt_interest,
        private_activity,
        exempt,
        distributions,
        nondeductible,
        prior_loss,
        prior_nondeductible,
        loans,
        risk,
        passive,
        loss_qbi,
        property_qbi,
        qbi_statement,
        amt,
        cash_charity,
        section179,
        section179_carryover,
        section179_carryover_sold,
        amt_section179_income_carry,
        foreign_taxes,
        carried,
        schedule_k3,
        credits,
        investment_property,
        income,
        deductions
    ],
    gains: [disposition],
    facts: [
        reporting_period,
        qbi_aggregation_group,
        health_insurance,
        trade_kind,
        losses_first,
        participation,
        stock_term,
        investment_confirmed,
        housing_confirmed,
        unentered,
        final_k1,
        special
    ],
});

// Community-property allocation of this source record; facts retain their meaning.
crate::community::divide_record!(ScorpLoan {
    amounts: [principal, basis, advances, repayments],
    gains: [],
    facts: [kind, repayment_term],
});

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(ScorpPassiveHistory {
    amounts: [loss],
    gains: [],
    facts: [complete],
});

// One native invocation owns basis allowance and before179 remaining risk.
impl ScorpK1 {
    pub(crate) fn risk_in(
        &self,
        events: Option<&ScorpEvents>,
        capacity: crate::scorp_amt::ScorpSourceCapacity,
    ) -> Result<crate::scorp_risk::ScorpRiskResult, (&'static str, &'static str)> {
        let basis = self.basis_result_in(capacity)?;
        self.risk.calculate_in(self, &basis, events, capacity).map_err(|error| error.message())
    }
}
