use crate::kernel::carry::{Amount, Cents};
// Coverage inv-419 (2026-10-18): a qualified retained section 111 comparison
//   limits prior partner fuel recovery through the shared cash-fuel worksheet.
//   Evidence: partial_partner_fuel_recovery_* regressions; 26 CFR 1.111-1(b)(2).
// Coverage update B9-7 R8 (2026-10-12): box 14 code A is independent of
//   box 1 when no ordinary loss/carry is held back. A binding ordinary basis,
//   at-risk or passive limit still requires a compatible SE loss share;
//   limited-partner and trading checks remain. Farm code A uses Schedule SE
//   line 1a after regular limits, in the same owner computation as nonfarm
//   earnings; optional elections retain their normal skipped lines.
//   Evidence: tests/passthrough/k1_items.rs and tests/passthrough/interview.rs k1_box14_*;
//   i1065sk1/i1040sse (2025 final), f1040sse/i1040sse (2026 drafts).
//   No change to retained SE character, AMT ownership or full B9-7 acceptance.
// Coverage update B9-7 business-property checkpoint (2026-10-12): Partner box 10/9c business record and separate section 1231 carries join the shared basis, at-risk and passive character owners; ordinary SE/QBI fields remain ordinary-only.
//   Evidence: tests/passthrough/k1_items.rs and tests/passthrough/interview.rs k1_partner_property_*; i1065sk1/i4797/i8582/i8995 (2025 final), f4797 2026 draft. Full B9-7 acceptance remains open.
// Coverage update B9-7 R11/R12 (2026-10-11): uncomputed K-1 credits and
//   oil/gas depletion leave only the benefit out, with one scoped notice;
//   code AX credit recapture remains refused. Accepted facts survive the
//   estimate and reopen; no unclaimed depletion reduces basis or SE earnings.
//   Sources: 2025 final K-1 instructions (1065/1120-S), credit code AX and
//   depletion code 20T/17R; tests/k1_items and tests/interview k1_benefit_*.
// Coverage update B9-7 R7 (2026-10-11): box 7 royalties and box 13 code I
//   use the shared basis/at-risk ledger and Schedule E Part I columns through
//   k1_items/k1_totals. Portfolio amounts stay outside passive loss, QBI and SE;
//   regular/AMT allowed expenses remain separate. tests/k1_items k1_royalty_*
//   exercise independent receipts, expense, limitation, AMT and combined columns.
//   Sources: i1065sk1 (2025 final), f1040se (2026 draft, created 2026-05-06).
// Coverage: Schedule K-1 (1065) — Partial (audited 2026-09-24).
//   Lines: Box 1 ordinary (regular and AMT), 4a/4b guaranteed payments, 14A SE earnings with
//   limited-partner classification, 20C fuel credit (Form 4136), 199A statement/QBI, sequential
//   basis, at-risk, passive limits with recapture, partner health insurance and SEP, prior
//   fuel-credit recovery, narrow Form 8978 BBA adjustment. Refused (complete != Yes): boxes
//   2–13 separately stated items, foreign items, unreimbursed expenses, final return/disposition,
//   special allocations, retiring payments; missing 14A, unresolved 1402(a) status, missing QBI
//   statement.
//   Evidence: partnership.rs (PartnerK1), partnership_basis.rs, partnership_qbi.rs, bba_audit.rs,
//   federal_rules.rs, extended_rules.rs; tests/passthrough/passthrough.rs
//   ordinary_partner_profit_reaches_complete_return_and_survives_restart,
//   partner_sequential_limits_keep_carries_separate_and_recapture_negative_risk,
//   partnership_se_source_is_independent_and_unknown_is_not_zero,
//   partnership_qbi_applies_sstb_once_and_wage_property_limits; sources i1065sk1, Pub 925, i1040se.
//   Unit C13b (2026-09-25): publicly_traded is supported for QBI (qualified PTP income to the
//   REIT/PTP component); a PTP with history, suspended balances or QBI wages/UBIA is refused by name.
//   Unit T2-29 (2026-09-26): PartnerK1.separate (PartnerSeparateItems) calculates box 5 interest
//   (1040 line 2b), 6a/6b dividends (3b/3a; 6b <= 6a on both sides, PARTNER_QUALIFIED_ABOVE_DIVIDENDS),
//   8 and 9a capital gain or loss (Schedule D lines 5/12), 9b collectibles (28% worksheet line 4), 9c
//   statement parts from partnership-interest sales and funds (1250 worksheet lines 10/11), 13 code A
//   cash gifts (Schedule A line 11 or the non-itemizer line 12f cap), 15 code O backup withholding
//   (line 25c) and 18 code A tax-exempt interest (line 2a). They enter the same outside basis
//   (worksheet line 4 increases; short loss, long loss and the gift as Part II categories after the
//   ordinary class) and at-risk amount (portfolio income and capital losses as Form 6198 activity
//   items; the gift stays outside). Refused by name: a separately stated loss limited by basis or at
//   risk (character carryforward), box 19A cash above basis (section 731 gain), 13A with a prior
//   cash-charity carryover, and every box listed in PartnerK1.unentered (PartnerK1Item: boxes 2/3,
//   9c business property, 10, 11, 12, 13 other codes, 15 other codes, 16/21, 17 beyond the AMT
//   record, 19B/C, 20 recapture/743(b)/sale codes, 22/23). Entered on the Schedule K-1 topic
//   since T2-29p (below).
//   STORAGE_FORMAT 45. Tests: tests/passthrough/passthrough.rs
//   partner_separately_stated_items_reach_the_return_by_hand,
//   partner_separate_losses_exactly_absorbed_by_basis_are_allowed,
//   partner_cash_distribution_equal_to_basis_has_no_gain_and_one_more_is_refused,
//   partner_box_8_loss_meets_the_3000_capital_loss_limit,
//   partner_cash_gift_above_the_non_itemizer_cap_is_limited_to_1000,
//   partner_qualified_dividends_above_box_6a_are_rejected, partner_boxes_not_calculated_are_refused_by_name.
//   Unit T2-29b (2026-09-26): PartnerK1.section179 (box 12) is one Form 4562 line 6 item of the
//   partner's own Part I (section179.rs pass_throughs/calculate): it joins line 8 without entering
//   the line 2 phaseout (Reg. 1.179-2(b)(3)); line 11 adds the materially participating partner's
//   box 1 net after basis and at-risk limits (Reg. 1.179-2(c)(2)(v)) and excludes guaranteed
//   payments (refused by name only when line 11 binds). The full box 12 amount is the last Part II
//   basis class (Reg. 1.179-3(h)(1), PartnerBasisRecord::result_with) in both tax systems; the line
//   12 share reaches Schedule E line 28 column (j) (extended_rules nonpassive), QBI as a deduction of
//   the business (federal_rules), and a general partner's box 14A (se_after_passive); the line 13
//   carryover is reported per K-1 with the basis note. Refused by name: a share suspended by basis
//   (SECTION179_LIMITED) or above the remaining amount at risk (Form 6198), a partner without
//   established material participation (Form 8582), a PTP, a share above the dollar limit, partner
//   health/retirement deductions, and box 20 codes L/M (dispositions and recapture) stay in
//   PartnerK1Item::OtherInformation. PartnerK1Item::Section179 is removed. STORAGE_FORMAT 55. Tests:
//   tests/passthrough/passthrough.rs partner_section179_reaches_form_4562_schedule_e_basis_and_qbi_by_hand,
//   general_partner_section179_claimed_reduces_box_14a_earnings,
//   partner_section179_meets_the_business_income_limit_and_carries_the_excess,
//   partner_section179_exactly_absorbed_by_basis_is_allowed_and_one_dollar_less_is_refused,
//   partner_section179_exactly_within_the_amount_at_risk_is_allowed_and_one_dollar_less_is_refused,
//   partner_section179_meets_the_dollar_limit_and_one_dollar_more_is_refused,
//   partner_section179_situations_not_calculated_are_refused_by_name; tests/activity/business.rs
//   section179_business_income_limit_includes_materially_participated_k1_income; sources i4562,
//   i1065sk1, Reg. 1.179-2, 1.179-3, i8995.
//   Unit T2-29p (2026-09-27): the interview enters this K-1 on the Schedule K-1 topic
//   (k1_page.rs K1PartnershipPaper); the PassThrough situation covers S corporations only. C9: with an
//   Unknown opening at-risk history the Form 6198 stage is skipped (PartnerRiskRecord::
//   without_history, risk_end None) when the year's losses entering it do not exceed the activity's
//   income, no at-risk carryover or unrecaptured deducted loss exists and box 12 is zero; otherwise
//   HISTORY_NEEDED is refused by name. C13: PartnerK1.section179_carryover is a Form 4562 line 10
//   item (section179.rs: not a line 8 election, allocated after current elections, deducted on
//   Schedule E column (j), not a basis reduction again); an allowed carryover without the at-risk
//   history is refused by name. STORAGE_FORMAT 62. Tests: tests/passthrough/passthrough.rs
//   partner_profit_without_at_risk_history_needs_no_form_6198,
//   partner_loss_equal_to_activity_income_needs_no_history_and_one_dollar_more_does,
//   partner_at_risk_carryover_or_recapture_history_still_needs_form_6198,
//   partner_section179_carryover_joins_form_4562_line_10_by_hand,
//   partner_section179_carryover_meets_the_business_income_limit_and_carries_the_rest.
//   Unit T2-43p (2026-09-27): new PartnerK1.pensions (box 13 code R) and retirement becomes
//   PartnerRetirement {None, Sep{rate, eligible}, Other} (sep.rs; the SEP contribution is
//   pensions, Other and box 13 code R without a plan are refused by name under "Partner
//   retirement"). PartnerK1Item::OtherDeductions narrows to box 13 codes other than A, M and R.
//   Entered on the self-employed benefits page (interview.rs). STORAGE_FORMAT 66.
//   Not-sure audit batch 7 (2026-10-05): se_history, the qualified SE gross and the
//   limited-partner year counts are required. No box 20 code Z statement (qbi None) adds no QBI
//   and no qualified SE gross (Reg. 1.199A-6(b)(3)(iii)) with WARN_K1_QBI_NO_STATEMENT, or
//   leaves a loss out with WARN_K1_QBI_LOSS_NO_STATEMENT. STORAGE_FORMAT 84.
//   Unit k1-rentals (2026-10-09): boxes 2 and 3 are calculated (k1_rental.rs): each tax system's
//   PartnerActivityRecord.rentals (box amounts and 2025 Form 8582 carryovers) enters this outside
//   basis (line 4 income; Part II classes after the separately stated ones) and this one Form 6198
//   activity (result_with, calculate_with, without_history); PartnerK1.rental_facts holds the box 2
//   participation (NotActive, Active, Professional) and the section 199A rental amounts. A limited
//   partner cannot be Active (section 469(i)(6)(C); valid, PARTNER_LIMITED_NOT_ACTIVE). Refused by
//   name: an unsupported rental disposition or unresolved rental-property character.
//   Kept passive PTP box 2/3 amounts share only their own PTP's passive pool;
//   professional, property, disposition and rental-QBI combinations retain located boundaries.
//   A rental disposition
//   (PartnerK1Item::Rentals renamed RentalDisposition). MultipleActivities narrows to box 22 or a
//   box 23 statement with two activities of one kind. The at-risk debt includes the partner's
//   qualified nonrecourse financing (section 465(b)(6)). Unit form-3468 (2026-10-09):
//   PartnerK1.investment_property (box 15 code E; box 20 codes D, E, AV) and investment_confirmed
//   (Form 3468, investment_credit.rs): the rehabilitation credit and the energy credit determined
//   on the statement's amounts reach Form 3800 Part III lines 4k, 4a and 1v (box 15 code E is a
//   Worksheet 2 rental real estate credit, the others follow material participation), and the
//   section 50(c)(5) reduction of the placed-in-service year lowers the outside basis in the
//   partner basis worksheet (PartnerBasis.credit_reduction, never carried); the recapture codes
//   (box 20 code H) and the sections 48A to 48D codes (AS, AT, AU) stay refused. Stored-shape change (STORAGE_FORMAT bumped
//   with the batch). Tests: tests/passthrough/k1_rental.rs; tests/passthrough/interview.rs k1_1065_rentals_*,
//   k1_1065_limited_partner_cannot_actively_participate_on_both_sides,
//   real_estate_professional_is_offered_with_a_k1_box_2_activity.
//   Unit k3 (2026-10-09): box 21 foreign taxes (PartnerSeparateItems.foreign_taxes) reduce the
//   outside basis as a fourth Part II class after the box 13 code A gift (section 705(a)(2)(B);
//   basis_losses, calculate_with) and are never read by Form 1116: the credit is figured from the
//   Schedule K-3 on PartnerK1.schedule_k3 (schedule_k3.rs, which owns the Coverage: Schedule K-3
//   block). A share limited by basis or at risk is refused by name (LIMITED). PartnerK1Item::
//   International is removed; the K-3 items ForeignCorporations and DisallowedDeduction carry its
//   refusal key. Stored-shape change (STORAGE_FORMAT bumped with the batch). Tests:
//   tests/foreign/foreign.rs k3_*, k1_foreign_tax_without_a_k3_is_never_credited.
//   Unit B8 (2026-10-10): the loss-class catalog (k1_loss.rs K1LossClass) replaces every positional
//   class; PartnerActivityRecord::plan builds one class input per K-1 and tax system (this year's
//   share, the basis carry and the at-risk carry of ordinary, box 2 and 3 rentals, box 8 and 9a
//   capital, box 12 section 179, box 13 code A gifts and box 21 foreign taxes), the basis worksheet
//   and Form 6198 share each limit pro rata across them, and PartnerOrdinaryResult.ledger (LossLedger)
//   says what each limit allowed and held back. A held-back amount keeps its character and carries
//   to 2027 (NOTE_K1_LOSS_CARRYFORWARD, the AMT twin NOTE_K1_AMT_CLASS_CARRYFORWARD); the 2025
//   carries are entered on PartnerCarried (PartnerActivityRecord.carried, capital_carried). The
//   basis-allowed amounts reach Schedule D (box 8, 9a), Schedule A / section 170(p) (box 13 A),
//   Form 4562 line 6 (box 12, current plus released) and Form 8582 (boxes 2 and 3); the foreign
//   taxes stay basis only. The retired refusals (a separately stated loss, section 179 or rental
//   loss limited by basis or at risk) and improvement row 241 (without_history) are gone: the basis
//   stage always runs and an unknown Form 6198 history needs the 2025 figures only for an at-risk
//   carry. Refused by name: box 9b or a box 9c part with a limited or released long-term loss
//   (ISSUE_PARTNER_LONG_TERM_PARTS_LIMITED), a section 179 share above the remaining amount at risk
//   (ISSUE_SECTION179_RISK_CARRYFORWARD, narrowed reason), a different AMT allowance of the gift or
//   section 179 (ISSUE_K1_AMT_LIMIT_DIFFERS). Stored-shape change (STORAGE_FORMAT bumped by the
//   dispatcher). Tests: tests/passthrough/k1_loss.rs; tests/passthrough/k1_rental.rs fixture_f_*, fixture_3b_*;
//   tests/passthrough/interview.rs k1_1065_loss_class_carries_round_trip_and_the_ledger_survives_reopen.
//   Unit B9-1 (2026-10-10): the box 11 and box 13 codes (k1_items.rs; PartnerK1.income and
//   deductions), box 20 codes AB, AC, AD, AI and AQ, the section 199A statement's dividends and PTP
//   items, the sale or cash liquidation of the whole interest (PartnerK1.disposition,
//   PartnerPlan::disposal_of) and box 19 cash above basis (section 731 gain, the refusal is gone).
//   Codes F and V adjust box 1 (PartnerItems), K is basis only; the activity items (E, R, S) join
//   the passive result (PartnerPoolResult.combined); a sale releases the activity's passive
//   losses (PooledNets, section 469(g)). `complete` becomes complete(): PartnerK1.special lists the
//   remaining situations. Stored-shape change (STORAGE_FORMAT bumped by the dispatcher). Tests:
//   tests/passthrough/k1_items.rs.

use crate::k1_loss::{ClassInput, K1LossClass, LossLedger, limit_classes};
use crate::kernel::phase;
use crate::kernel::ratio::dollars;
use crate::partnership_basis::{PartnerBasis, PartnerBasisResult};
use crate::{FederalFuelLines, ScorpParticipation};
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Ordinary partnership outside-basis records
/// Retain tax basis, not the capital account in K-1 item L. This source record covers cash contributions and distributions for a continuing ordinary business; property, ownership-transfer and special transaction histories need their own records. Amount at risk and passive loss history are separate.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PartnerBasisRecord {
    /// Tax basis at the beginning of the partnership year
    /// [NullText]
    /// Opening basis is not established yet
    /// [SetText]
    /// Established opening basis
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub opening: Option<i64>,
    /// Cash capital contributions during the year
    /// Exclude guaranteed payments, loans and liability shares entered separately.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub contributions: i64,
    /// Beginning allocated partnership liabilities, K-1 item K1
    /// Include recourse and nonrecourse shares for outside basis. This does not establish that any debt is at risk.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub opening_liabilities: i64,
    /// Ending allocated partnership liabilities, K-1 item K1
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub ending_liabilities: i64,
    /// Partnership liabilities personally assumed during the year
    /// From the assumption records; the App combines this with the change in allocated liabilities.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub assumed: i64,
    /// Personal liabilities assumed by the partnership
    /// Exclude amounts already included in the allocated-liability change.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub transferred: i64,
    /// Cash distributions during the year
    /// Include insurance premiums charged against distributions, before subtracting them from the cash actually delivered to you. Exclude guaranteed payments and debt relief entered separately. Property and marketable-security distributions need their own treatment.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub distributions: i64,
    /// Ordinary loss suspended only by outside basis through 2025
    /// Exclude at-risk and passive carryovers; those already passed the basis limit.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub prior_loss: i64,
    /// Nondeductible expense carry from the retained partner-basis worksheet
    /// Exclude any at-risk-only expense carry. This reduces basis when absorbed; it is not a tax deduction.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub prior_nondeductible: i64,
    /// Complete ordinary cash and liability records
    /// Confirm annual worksheet ordering applies, no property contributions or distributions, sales, acquisitions, liquidations, section 743(b), special allocations, foreign basis adjustments or unentered basis adjustments. Business interest needs its own loss class, which is not entered here. No leaves this K-1 out of the estimate with a notice; that situation is not supported.
    pub complete: bool,
}

impl PartnerBasisRecord {
    fn valid_amounts(&self, ordinary: i64, exempt: i64, nondeductible: i64) -> bool {
        let amounts = [
            self.contributions,
            self.opening_liabilities,
            self.ending_liabilities,
            self.assumed,
            self.transferred,
            self.distributions,
            self.prior_loss,
            self.prior_nondeductible,
            exempt,
            nondeductible,
        ];
        // Arithmetic safety: result_with and disposal_of retain native basis,
        // gain and liability arithmetic before k1_totals cents transfers.
        self.opening.is_none_or(|a| (0..=100000000000).contains(&a))
            && amounts.into_iter().all(|a| (0..=100000000000).contains(&a))
            && (-100000000000..=100000000000).contains(&ordinary)
    }
    // The ordinary class alone (tests): the box 1 result and the retained
    // basis loss, with no separately stated item, rental or section 179.
    #[cfg(test)]
    pub(crate) fn ordinary_result(
        &self,
        ordinary: i64,
        exempt: i64,
        nondeductible: i64,
    ) -> Result<PartnerBasisResult, (&'static str, &'static str)> {
        if !self.valid_amounts(ordinary, exempt, nondeductible) {
            return Err(PartnerPlan::INCOMPLETE);
        }
        let mut plan = PartnerPlan::default();
        plan.input.current[K1LossClass::Ordinary] =
            Amount::<Cents>::new((-ordinary).max(0)).expect("validated source class cents");
        plan.input.basis_carry[K1LossClass::Ordinary] =
            Amount::<Cents>::new(self.prior_loss).expect("validated source class cents");
        self.result_with(ordinary, exempt, nondeductible, &plan)
    }

    // Part I of the partner basis worksheet with the separately stated items
    // (T2-29): portfolio income, gains and tax-exempt interest are line 4
    // increases; every loss and deduction is a Part II class (the plan's
    // ClassInput) allocated pro rata, current plus carried (Reg.
    // 1.704-1(d)(2); section 704(d) covers charitable gifts and foreign taxes
    // for partnership years beginning after 2017). The box 12 section 179
    // amount (T2-29b) is a class too: the full allocated amount reduces
    // basis this year whether or not Form 4562 lets the partner deduct it
    // (Reg. 1.179-3(h)(1)). Boxes 2 and 3 (unit k1-rentals): rental income is
    // a line 4 increase and each rental loss is its own Part II class.
    pub(crate) fn result_with(
        &self,
        ordinary: i64,
        exempt: i64,
        nondeductible: i64,
        plan: &PartnerPlan,
    ) -> Result<PartnerBasisResult, (&'static str, &'static str)> {
        let opening = self.opening.ok_or((
            "DOCUMENT_K1_OPENING_BASIS",
            crate::fixture_english!(
                "Enter the actual opening basis, including zero when it is zero."
            ),
        ))?;
        if !self.complete || !self.valid_amounts(0, exempt, nondeductible) {
            return Err(PartnerPlan::INCOMPLETE);
        }
        let losses = plan.input.checked_basis().ok_or(PartnerPlan::RANGE)?;
        PartnerBasis {
            opening: Amount::<Cents>::new(opening).expect("bounded basis result"),
            contributions: Amount::<Cents>::new(self.contributions).expect("bounded basis result"),
            opening_liabilities: Amount::<Cents>::new(self.opening_liabilities)
                .expect("bounded basis result"),
            ending_liabilities: Amount::<Cents>::new(self.ending_liabilities)
                .expect("bounded basis result"),
            partnership_liabilities_assumed: Amount::<Cents>::new(self.assumed)
                .expect("bounded basis result"),
            personal_liabilities_transferred: Amount::<Cents>::new(self.transferred)
                .expect("bounded basis result"),
            income: Amount::<Cents>::new(
                phase::floor(ordinary, 0)
                    .checked_add(exempt)
                    .and_then(|income| income.checked_add(plan.basis_income))
                    .ok_or(PartnerPlan::RANGE)?,
            )
            .expect("nonnegative basis income"),
            cash_distributions: Amount::<Cents>::new(self.distributions)
                .expect("bounded basis result"),
            distributed_property_basis: Amount::<Cents>::new(0).expect("bounded basis result"),
            nondeductible: Amount::<Cents>::new(nondeductible + self.prior_nondeductible)
                .expect("bounded basis result"),
            credit_reduction: Amount::<Cents>::new(plan.credit_reduction)
                .expect("bounded basis result"),
            depletion: Amount::<Cents>::new(0).expect("bounded basis result"),
            losses: &losses,
            business_interest: Amount::<Cents>::new(0).expect("bounded basis result"),
            excess_business_interest: Amount::<Cents>::new(plan.ebie)
                .expect("bounded basis result"),
            deemed_money: Amount::<Cents>::new(plan.deemed_money).expect("bounded basis result"),
        }
        .calculate()
        .ok_or((
            "ISSUE_PARTNERSHIP_THE_PARTNER_BASIS_SOURCE_AMOUNTS_CANNOT_BE_56598BD1",
            crate::fixture_english!("The partner-basis source amounts cannot be reconciled."),
        ))
    }
}

// The K-1 boxes, shared by both tax systems, that join one tax system's
// record in the basis and at-risk limits.
#[derive(Clone, Copy, Default)]
pub(crate) struct PartnerItems<'a> {
    pub separate: Option<&'a PartnerSeparateTotals>,
    // Box 12, in cents.
    pub section179: i64,
    pub section179_carryover: i64,
    pub section179_carryover_sold: i64,
    pub section179_sales: &'a [crate::K1Section179Sale],
    pub alternative_sales: bool,
    // The section 50(c)(5) basis reduction, in cents.
    pub credit_reduction: i64,
    // Unit B9-1: the box 11 and box 13 codes, and what happened to the
    // interest (the sale or liquidation of all of it).
    pub income_codes: &'a [crate::PartnerIncomeTotal],
    pub deduction_codes: &'a [crate::PartnerDeduction],
    pub disposition: Option<&'a crate::K1Disposition>,
    pub sale: Option<&'a crate::PartnerSaleParts>,
    // A community-property allocation can give source income and external
    // property gains different fractions. This computed disposal owns only
    // the property event; source answers and the income loss ledger stay intact.
    pub property_disposal: Option<crate::k1_items::Disposal>,
}

impl PartnerItems<'_> {
    pub(crate) fn k1(&self) -> crate::k1_items::K1Items {
        crate::k1_items::K1Items::partner(self.income_codes, self.deduction_codes)
    }
    // Whether the whole interest left the partner's hands this year.
    pub(crate) fn ended(&self) -> bool {
        self.disposition.is_some_and(|v| v.ended())
    }
}

// One class input per K-1 and tax system (unit B8): what the record brings
// to each limit by class, and the income the worksheet and Form 6198 add.
// Whole dollars except the section 179 box.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PartnerPlan {
    pub input: ClassInput,
    // Partner basis worksheet line 4: the separately stated and rental income.
    pub basis_income: i64,
    // Form 6198 income of the K-1's one activity: portfolio income, gains and
    // rental income.
    pub risk_income: i64,
    // Tax-exempt interest added to the amount at risk.
    pub risk_exempt: i64,
    pub credit_reduction: i64,
    pub section179_cents: i64,
    // Box 13 code K, actual source cents: it reduces basis, whether or not it is
    // deductible (worksheet line 15n).
    pub ebie: i64,
    // Money the liquidation relieves, the item K1 share of liabilities
    // immediately before it (section 752(b)), actual source cents.
    pub deemed_money: i64,
    // The sale or liquidation of the whole interest this year.
    pub disposal: Option<DisposalInput>,
    pub property_disposal: Option<crate::k1_items::Disposal>,
}

// The facts of a sale or liquidation of the whole interest that the basis
// result completes (actual source cents).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DisposalInput {
    // Proceeds of a sale; None for a liquidation for cash.
    pub proceeds: Option<i64>,
    // The item K1 share of liabilities immediately before the disposition.
    pub relief: i64,
    // Box 20 code AB, the section 751 gain (loss).
    pub ordinary: i64,
}

impl PartnerPlan {
    // The gain of this plan's year-end events from the basis result.
    pub(crate) fn disposal_of(&self, basis: &PartnerBasisResult) -> crate::k1_items::Disposal {
        if let Some(disposal) = self.property_disposal {
            return disposal;
        }
        let mut disposal = crate::k1_items::Disposal {
            distribution_gain: basis.distribution_gain.get(),
            ..Default::default()
        };
        if let Some(input) = self.disposal {
            disposal.sold = true;
            disposal.ordinary = input.ordinary;
            disposal.basis = basis.ending.get();
            if let Some(proceeds) = input.proceeds {
                disposal.proceeds = proceeds + input.relief - input.ordinary;
            }
        }
        disposal
    }

    pub(crate) const RANGE: (&'static str, &'static str) = (
        crate::k1_activity::ordinary::RANGE_KEY,
        crate::fixture_english!(
            "Review the document amounts; they must be within the supported input range."
        ),
    );
    const INCOMPLETE: (&'static str, &'static str) = (
        "ISSUE_PARTNERSHIP_COMPLETE_THE_ORDINARY_PARTNER_BASIS_SOURCE_RECORDS_FCA2848B",
        crate::fixture_english!(
            "Complete the ordinary partner-basis source records and identify additional adjustments."
        ),
    );
}

/// Partner's material participation under the passive-loss rules
/// This classification is for section 469, not the separate limited-partner self-employment classification under section 1402. Rental, farming exceptions, unresolved entity classification and grouping need separate analysis.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum PartnerParticipation {
    /// Ordinary individual tests apply
    /// Includes a partner who owned a general-partner interest throughout the year. Use only when the section 469 classification is established.
    General { facts: ScorpParticipation },
    /// Limited-partner restrictions apply
    /// Only the more-than-500-hour, five-of-ten-year or personal-service-history tests can establish material participation.
    Limited {
        /// Combined qualifying work by the partner and spouse
        /// Exclude investor-only work and work performed to evade the passive-loss rules.
        #[libertas_number(min = 0)]
        #[libertas_physical_unit("minute")]
        minutes: u32,
        /// Material participation among 2016 through 2025
        /// Count the returns from 2016 through 2025 on which Schedule E Part II line 28 reported this activity as nonpassive (column (i) or (k)) because you materially participated. Do not establish those solely through the historical five-of-ten test. Enter zero when there were none.
        // Tax structure: the 2016–2025 lookback contains ten tax years.
        #[libertas_number(min = 0, max = 10)]
        #[libertas_physical_unit("year")]
        prior_ten: u32,
        /// This is a personal-service activity
        /// Health, law, engineering, architecture, accounting, actuarial science, performing arts, consulting or another field where capital is not a material income-producing factor (Reg. §1.469-5T(d)).
        personal_service: bool,
        /// Earlier material participation in this personal-service activity
        /// Count them from your earlier returns: every return before 2026 that Schedule E Part II reported this activity as nonpassive (line 28 column (i) or (k)) because you materially participated. Enter zero when there were none or when it is not a personal-service activity.
        #[libertas_number(min = 0)]
        #[libertas_physical_unit("year")]
        service_years: u32,
    },
}
impl PartnerParticipation {
    pub(crate) fn material(&self) -> crate::participation::Participation {
        match self {
            Self::General { facts } => facts.material(),
            Self::Limited { minutes, prior_ten, personal_service, service_years } => {
                // Tax structure: at most two spouses' minutes in the 2026 calendar year,
                // and ten years in the material-participation lookback.
                if *minutes > 1_051_200
                    || *prior_ten > 10
                    || (!*personal_service && *service_years != 0)
                {
                    return crate::participation::Participation::Unavailable;
                }
                if *minutes > 30_000
                    || *prior_ten >= 5
                    || (*personal_service && *service_years >= 3)
                {
                    crate::participation::Participation::Yes
                } else {
                    crate::participation::Participation::No
                }
            }
        }
    }
}

/// Partner's ordinary-business at-risk records
/// Separate from outside basis and passive losses. Use one continuing cash-funded activity with no property transactions, debt assumptions/transfers, outside collateral, pre-1979 balances or unentered adjustments. Ordinary nonrecourse liabilities may increase outside basis without increasing these at-risk amounts; your share of qualified nonrecourse financing secured by real property is at risk.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PartnerRiskRecord {
    /// Opening detailed-method at-risk history
    pub start: crate::ScorpRiskStart,
    /// Beginning allocated liabilities for which this partner is at risk
    /// The recourse part from financing and recourse records, plus the share of qualified nonrecourse financing secured by real property (K-1 item K1), excluding related-party and protected borrowing. Do not copy the full K-1 liability total automatically.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub opening_debt: i64,
    /// Ending allocated liabilities for which this partner is at risk
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub ending_debt: i64,
    /// Cash contributions financed with amounts not at risk
    /// Included in the basis record's contributions. Exclude amounts already identified in the debt allocation.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub excluded_contributions: i64,
    /// Cash distributions returning previously excluded investment
    /// Included in the basis record's distributions; never count the same funds twice.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub excluded_distributions: i64,
    /// Previously at-risk cash investment newly protected against loss
    /// Exclude casualty or tort insurance and debt changes already entered.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub protected: i64,
    /// Previously excluded cash investment no longer protected
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub released: i64,
    /// Ordinary loss suspended only by at-risk limits through 2025
    /// Already passed outside basis; do not enter it again as a basis carryover.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub prior_loss: i64,
    /// Nondeductible expenses suspended only by at-risk limits
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub prior_nondeductible: i64,
    /// Cumulative previously deducted losses for recapture
    /// After 1978, excluding this year and losses never deducted.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub deducted: i64,
    /// At-risk recapture already reported in prior years
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub recaptured: i64,
    /// Complete ordinary financing records
    /// Tax-exempt income belongs to this activity. No unentered prior recapture deduction, guaranteed payments other than separately entered current service and capital payments, separately stated activity items, distributions producing gain or other special financing. These records determine the at-risk stage only; passive and return-level limits still apply. No leaves this K-1 out of the estimate with a notice; that situation is not supported.
    pub complete: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct PartnerOrdinaryResult {
    pub basis: PartnerBasisResult,
    // None when the at-risk stage limited nothing and the opening history
    // is unknown (question C9): the amount at risk is then not determined.
    pub risk_investment: Option<i64>,
    pub income: Amount<Cents>,
    // What the basis and at-risk limits allowed and held back of each class.
    pub ledger: LossLedger,
    pub nondeductible_used: Amount<Cents>,
    pub nondeductible_suspended: Amount<Cents>,
    pub risk_end: Option<i64>,
    pub recapture: crate::scorp_basis::AtRiskRecapture,
    // The gain on the sale or liquidation of the whole interest and on cash
    // above the basis (unit B9-1).
    pub disposal: crate::k1_items::Disposal,
}

pub(crate) struct PartnerRiskFinancing {
    pub contributions: i64,
    pub distributions: i64,
    pub opening_liabilities: i64,
    pub ending_liabilities: i64,
    pub assumed: i64,
    pub transferred: i64,
    pub has_prior: bool,
}

impl PartnerRiskRecord {
    // Arithmetic safety: calculate_with/without_history combine native risk
    // components and their regular/AMT form-dollar loss classes.
    fn source_amounts_valid(&self) -> bool {
        [
            self.opening_debt,
            self.ending_debt,
            self.excluded_contributions,
            self.excluded_distributions,
            self.protected,
            self.released,
            self.prior_loss,
            self.prior_nondeductible,
            self.deducted,
            self.recaptured,
        ]
        .into_iter()
        .all(|amount| (0..=100000000000).contains(&amount))
            && self.recaptured <= self.deducted
    }

    // Same native risk-history arithmetic exclusion as source_amounts_valid.
    fn history_amounts_valid(&self) -> bool {
        match self.start {
            crate::ScorpRiskStart::Detailed { capacity, profit, loss } => [capacity, profit, loss]
                .into_iter()
                .all(|amount| (0..=100000000000).contains(&amount)),
            _ => true,
        }
    }

    // Reuse the native history owner with exact source cents (MoneyPrecision,
    // 2026-10-19). Complete=false remains an independent source fact.
    pub fn source_valid(&self) -> bool {
        self.source_amounts_valid()
            && self.history_amounts_valid()
            && match self.start {
                crate::ScorpRiskStart::Detailed { capacity, profit, loss } => {
                    crate::scorp_basis::prior_detailed_risk(
                        Amount::<Cents>::new(capacity).expect("validated risk history"),
                        Amount::<Cents>::new(profit).expect("validated risk history"),
                        Amount::<Cents>::new(loss).expect("validated risk history"),
                    )
                    .map(|value| value.get())
                    .is_ok()
                }
                _ => true,
            }
    }
    // The ordinary class alone (tests): the box 1 result, the retained basis
    // and at-risk losses, with no separately stated item, rental or section
    // 179.
    #[cfg(test)]
    pub(crate) fn calculate(
        &self,
        source: &PartnerBasisRecord,
        ordinary: i64,
        exempt: i64,
        nondeductible: i64,
    ) -> Result<PartnerOrdinaryResult, (&'static str, &'static str)> {
        let mut plan = PartnerPlan::default();
        plan.input.current[K1LossClass::Ordinary] =
            Amount::<Cents>::new((-ordinary).max(0)).expect("validated source class cents");
        plan.input.basis_carry[K1LossClass::Ordinary] =
            Amount::<Cents>::new(source.prior_loss).expect("validated source class cents");
        plan.input.risk_carry[K1LossClass::Ordinary] =
            Amount::<Cents>::new(self.prior_loss).expect("validated source class cents");
        self.calculate_with(source, ordinary, exempt, nondeductible, &plan)
    }

    // Shared source-owned history/financing calculation. Callers supply actual
    // cash/liability facts, never a fabricated single-activity basis worksheet.
    pub(crate) fn investment(
        &self,
        financing: PartnerRiskFinancing,
        exempt: i64,
        risk_exempt: i64,
    ) -> Result<Option<i64>, (&'static str, &'static str)> {
        use crate::scorp_basis::prior_detailed_risk;
        let intrinsic = self.source_valid();
        if !self.complete
            || (!intrinsic && !self.source_amounts_valid())
            || self.opening_debt > financing.opening_liabilities
            || self.ending_debt > financing.ending_liabilities
            || self.excluded_contributions > financing.contributions
            || self.excluded_distributions > financing.distributions
            || financing.assumed != 0
            || financing.transferred != 0
        {
            return Err((
                "ISSUE_PARTNERSHIP_RECONCILE_PARTNER_FINANCING_AND_ADDITIONAL_AT_RISK_B169E3B9",
                crate::fixture_english!(
                    "Reconcile partner financing and additional at-risk adjustments before applying loss limits."
                ),
            ));
        }
        let opening = match self.start {
            crate::ScorpRiskStart::New => {
                if financing.has_prior
                    || self.prior_loss != 0
                    || self.prior_nondeductible != 0
                    || self.deducted != 0
                    || self.recaptured != 0
                    || self.protected != 0
                    || self.released != 0
                {
                    return Err((
                        "ISSUE_PARTNERSHIP_A_NEW_ACTIVITY_CANNOT_CONTAIN_PRIOR_822BC4EF",
                        crate::fixture_english!(
                            "A new activity cannot contain prior investment or loss history."
                        ),
                    ));
                }
                0
            }
            crate::ScorpRiskStart::Detailed { capacity, profit, loss } => {
                // Keep the existing specific history error and financing
                // error priority while reusing the one intrinsic predicate.
                if !intrinsic && !self.history_amounts_valid() {
                    return Err((
                        "ISSUE_PARTNERSHIP_REVIEW_THE_RETAINED_FORM_6198_AMOUNTS_34CD8625",
                        crate::fixture_english!("Review the retained Form 6198 amounts."),
                    ));
                }
                prior_detailed_risk(
                    Amount::<Cents>::new(capacity).expect("validated risk history"),
                    Amount::<Cents>::new(profit).expect("validated risk history"),
                    Amount::<Cents>::new(loss).expect("validated risk history"),
                )
                .map(|value| value.get())
                .map_err(|_| {
                    (
                        "ISSUE_PARTNERSHIP_THE_RETAINED_AT_RISK_HISTORY_IS_INCONSISTENT_0CD4F902",
                        crate::fixture_english!("The retained at-risk history is inconsistent."),
                    )
                })?
            }
            crate::ScorpRiskStart::Unknown => {
                return Ok(None);
            }
        };
        let investment = opening + financing.contributions - self.excluded_contributions
            + self.ending_debt
            - self.opening_debt
            - financing.distributions
            + self.excluded_distributions
            + self.released
            - self.protected;
        let investment = investment
            .checked_add(exempt)
            .and_then(|value| value.checked_add(risk_exempt))
            .ok_or(PartnerPlan::RANGE)?;
        Ok(Some(investment))
    }

    // Form 6198 for the K-1's one activity, after the basis stage. The box
    // 12 section 179 amount reduces basis in full and is a basis class; it
    // is not a Form 6198 loss of this stage (Form 4562 Part I decides the
    // deductible part first and section179.rs tests that part against the
    // remaining amount at risk). The box 2 and 3 rental activities (unit
    // k1-rentals) share this outside basis and, without box 22, this one
    // Form 6198 activity: their income is activity income and their losses
    // are activity losses, like every at-risk class of the ledger.
    pub(crate) fn calculate_with(
        &self,
        source: &PartnerBasisRecord,
        ordinary: i64,
        exempt: i64,
        nondeductible: i64,
        plan: &PartnerPlan,
    ) -> Result<PartnerOrdinaryResult, (&'static str, &'static str)> {
        let basis = source.result_with(ordinary, exempt, nondeductible, plan)?;
        // The gain on cash above the basis (section 731(a)(1)) and on the
        // sale of the whole interest is income of the K-1's one Form 6198
        // activity (i6198 lines 2a to 2c).
        let disposal = plan.disposal_of(&basis);
        let plan = &PartnerPlan {
            risk_income: plan
                .risk_income
                .checked_add(disposal.activity_gain())
                .ok_or(PartnerPlan::RANGE)?,
            ..*plan
        };
        let financing = PartnerRiskFinancing {
            contributions: source.contributions,
            distributions: source.distributions,
            opening_liabilities: source.opening_liabilities,
            ending_liabilities: source.ending_liabilities,
            assumed: source.assumed,
            transferred: source.transferred,
            has_prior: source.opening.is_some_and(|value| value != 0)
                || source.opening_liabilities != 0
                || source.prior_loss != 0
                || source.prior_nondeductible != 0
                || plan.input.carries(),
        };
        let Some(investment) = self.investment(financing, exempt, plan.risk_exempt)? else {
            return self.without_history(basis, ordinary, plan, disposal);
        };
        let income = phase::floor(ordinary, 0);
        let activity_income = income.checked_add(plan.risk_income).ok_or(PartnerPlan::RANGE)?;
        // Form 6198 treats the K-1's portfolio income and capital gains or
        // losses as items of the one partnership activity: income raises the
        // loss allowance and capital losses share it. The box 13 code A gift
        // and the box 21 foreign taxes are not activity deductions and stay
        // outside the at-risk stage. Prior risk carries have already reduced
        // basis. Do not limit them twice.
        let stage = limit_classes(
            Amount::new(activity_income).expect("nonnegative activity income"),
            Amount::new(phase::floor(investment, 0)).expect("nonnegative risk capacity"),
            &basis.allowed,
            &plan.input.risk_carry,
            Amount::new(basis.nondeductible_used.get() + self.prior_nondeductible)
                .expect("nonnegative activity expenses"),
        )
        .map_err(|_| {
            (
                "ISSUE_PARTNERSHIP_THE_PARTNER_S_AT_RISK_LOSS_ALLOCATION_IS_D6B562C7",
                crate::fixture_english!("The partner's at-risk loss allocation is inconsistent."),
            )
        })?;
        let account = (|| {
            use crate::kernel::carry::{Amount, Cents};
            use crate::kernel::loss_limits::risk::{RecaptureHistory, close_economic_account};
            use crate::kernel::source_fact::SourceFact;
            let history = RecaptureHistory::<Cents> {
                prior_deducted: Amount::new(self.deducted)?,
                prior_recaptured: Amount::new(self.recaptured)?,
                threshold_1978: 0,
            };
            close_economic_account(
                investment,
                activity_income - stage.allowed_total.get(),
                SourceFact::Reported(history),
            )
            .map_err(|_| crate::kernel::carry::LedgerError::InvalidAmount)
        })()
        .map_err(|_| {
            (
                "ISSUE_PARTNERSHIP_THE_PARTNER_S_RECAPTURE_HISTORY_IS_INCONSISTENT_4E8989F8",
                crate::fixture_english!("The partner's recapture history is inconsistent."),
            )
        })?;
        let ending = account.ending;
        let crate::kernel::source_fact::SourceFact::Reported(recapture) = account.recapture else {
            unreachable!("reported recapture history")
        };
        let recapture = crate::scorp_basis::AtRiskRecapture {
            income: recapture.income,
            next_year_deduction: recapture.next_year_deduction,
        };
        Ok(PartnerOrdinaryResult {
            ledger: LossLedger::of(basis.allowed, basis.suspended, &stage),
            basis,
            risk_investment: Some(investment),
            income: Amount::new(income).expect("nonnegative activity income"),
            nondeductible_used: stage.nondeductible_allowed,
            nondeductible_suspended: stage.nondeductible_suspended,
            risk_end: Some(ending),
            recapture,
            disposal,
        })
    }

    // Question C9: Form 6198 is filed only for a loss. A continuing partner
    // without 2025 at-risk figures needs none when the stage limits nothing:
    // the year's losses entering it (the basis-allowed amount of every
    // at-risk class, current plus released, and nondeductible expenses) do
    // not exceed the activity's income, no loss or expense is carried at
    // risk from 2025, no box 12 section 179 amount waits for its own at-risk
    // test, and no earlier deducted loss is left to recapture. The amount at
    // risk itself stays unknown; any other case still needs the history. The
    // basis stage always runs, so a basis-limited class carries without it.
    fn without_history(
        &self,
        basis: PartnerBasisResult,
        ordinary: i64,
        plan: &PartnerPlan,
        disposal: crate::k1_items::Disposal,
    ) -> Result<PartnerOrdinaryResult, (&'static str, &'static str)> {
        let income = phase::floor(ordinary, 0);
        let activity_income = income.checked_add(plan.risk_income).ok_or(PartnerPlan::RANGE)?;
        let losses: i64 = K1LossClass::ALL
            .into_iter()
            .filter(|class| class.at_risk())
            .map(|class| basis.allowed[class].get())
            .sum();
        if self.prior_nondeductible != 0
            || plan.input.risk_carry.any()
            || self.deducted != self.recaptured
            || plan.section179_cents != 0
            || basis.allowed[K1LossClass::Section179].get() != 0
            || losses.checked_add(basis.nondeductible_used.get()).ok_or(PartnerPlan::RANGE)?
                > activity_income
        {
            return Err(("ISSUE_PARTNER_AT_RISK_HISTORY_NEEDED", Self::HISTORY_NEEDED));
        }
        Ok(PartnerOrdinaryResult {
            income: Amount::new(income).expect("nonnegative activity income"),
            ledger: LossLedger::without_risk_stage(basis.allowed, basis.suspended),
            nondeductible_used: basis.nondeductible_used,
            nondeductible_suspended: Amount::default(),
            risk_investment: None,
            risk_end: None,
            recapture: crate::scorp_basis::AtRiskRecapture {
                income: Amount::default(),
                next_year_deduction: Amount::default(),
            },
            basis,
            disposal,
        })
    }

    pub(crate) const HISTORY_NEEDED: &'static str = crate::fixture_english!(
        "Reconstruct opening at-risk history; outside basis cannot substitute for it. Your 2025 Form 6198 figures are needed when this K-1 has a loss, a loss carried because of the at-risk limits, a section 179 deduction or earlier losses to recapture."
    );
}

/// Ordinary partnership activity in one tax system
/// Keep separate regular and AMT records; neither system's basis or suspended balance substitutes for the other.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PartnerActivityRecord {
    /// Ordinary business income or loss, K-1 box 1
    /// For AMT use the documented amount after that system's activity adjustments. Positive is income and negative is loss. Other separately stated items are not included here.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub ordinary: i64,
    /// Tax-exempt income belonging to this activity
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub exempt: i64,
    /// Current noncapital nondeductible expenses
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub nondeductible: i64,
    pub basis: PartnerBasisRecord,
    pub risk: PartnerRiskRecord,
    /// Prior ordinary loss suspended only by passive limits
    /// These losses already passed basis and at-risk limits. Retain this tax system's balance; exclude those other carryovers. No disposition, inheritance, gift or change in activity grouping.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub passive_loss: i64,
    /// Rental activities — boxes 2 and 3
    pub rentals: crate::K1RentalAmounts,
    /// Losses carried because of your basis or amount at risk — rentals, section 179, gifts and foreign taxes
    pub carried: crate::K1LossCarryforwards,
    /// Capital losses carried because of your basis or amount at risk
    pub capital_carried: crate::PartnerCapitalCarry,
    /// Business property of the box 1 activity
    /// The business parts of boxes 10 and 9c and their separate loss carryovers. Rental property belongs with its rental activity, not here.
    /// [NullText]
    /// This activity has no section 1231 business-property records
    /// [SetText]
    /// The activity's section 1231 business-property records are
    #[libertas_default($null)]
    pub section1231: Option<crate::K1Section1231>,
}

/// Self-employment character of retained ordinary partnership losses
/// Copy the self-employment portions of the separate retained loss worksheets, not additional losses. These amounts are subsets of the regular-tax basis, at-risk and passive balances. Current limited-partner classification does not establish the character of earlier losses.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PartnerSeHistory {
    /// Self-employment portion of the basis-only loss carry
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub basis: i64,
    /// Self-employment portion of the at-risk-only loss carry
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub risk: i64,
    /// Self-employment portion of the passive-only loss carry
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub passive: i64,
}

/// Partnership gross income for self-employment
/// Use the K-1 statement's share of gross income for self-employment purposes, before operating deductions. Code B identifies a farming or fishing partnership for Schedule SE line 1a; these gross amounts also support the optional methods and the QBI deduction allocation. This does not add taxable income. Limited partners include only guaranteed payments for services; include them in the service-payment field and in the reported SE gross amount.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum PartnerSeGross {
    /// Farming — K-1 box 14 code B
    Farm {
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        gross: i64,
    },
    /// Nonfarming — K-1 box 14 code C
    Nonfarm {
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        gross: i64,
    },
}
impl PartnerSeGross {
    pub(crate) fn source(&self) -> (usize, i64) {
        match *self {
            Self::Farm { gross } => (0, gross),
            Self::Nonfarm { gross } => (1, gross),
        }
    }
}

/// Partner's Form 4136 fuel lines from K-1 box 20 code C
/// The partnership reports the partner's distributive share of qualifying fuel gallons, use and applicable rate. This is a partner-level payment credit, not a second partnership expense or a new K-1 ordinary-income item.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PartnerFuelTaxCredit {
    /// Partner-share gallon and cost records for the supported Form 4136 lines, including any registered issuer's government-card sales
    // Boxed: inline, these lines would set the size of every income source and record value.
    pub fuels: alloc::boxed::Box<FederalFuelLines>,
    /// K-1 statement and underlying fuel, cost, use and no-duplicate-claim records are complete
    /// Check the attached K-1 fuel statement and underlying use and cost records. No claims no fuel tax credit from this K-1.
    pub statement_complete: bool,
    /// Farm-operator activity for farm gasoline, diesel, kerosene or emulsion claims
    /// A custom aerial applicator's aviation gasoline or kerosene qualifies from its applicator and purchaser facts even if the partnership does not operate the serviced farm. Kerosene also requires an unwaived claim. Whether you operate the farm is a known fact. With farm-operator fuel lines, No claims no fuel tax credit from this K-1.
    pub farm_activity: bool,
}

impl PartnerFuelTaxCredit {
    pub(crate) fn valid(&self) -> bool {
        self.fuels.valid()
    }

    pub(crate) fn credit_cents(&self) -> i64 {
        self.fuels.credit_cents()
    }
}

/// Prior partnership fuel credit that becomes 2026 Schedule E income
/// This is the cash-method partner's credit claimed on a 2025 Form 1040 filed in 2026, not 2026 K-1 box 1 income or another 2026 Form 4136 payment. Ordinary continuing nonpassive U.S. business only; passive histories need separate worksheets.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PartnerPriorFuelCredit {
    /// Fuel credit claimed on the 2025 Form 1040, in dollars and cents
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub claimed_credit: i64,
    /// The 2025 Form 4136 credit was claimed on a 2025 return filed in 2026
    /// The 2025 return's filing date shows it. No leaves this recovery out of the estimate with a notice; that situation is not supported.
    // The filing year is relative to the return; the source return is its prior year.
    pub filed_in_return_year: bool,
    /// This activity uses the cash method for the fuel-credit income
    /// Accrual-method activities recognize the credit in the fuel-use year and need a different recovery path. The partnership's K-1 or statement shows its accounting method. No leaves this recovery out of the estimate with a notice.
    pub cash_method: bool,
    /// Full fuel cost including this credited tax was deducted and reduced 2025 income tax
    /// Confirm the same taxable recovery for regular and AMT purposes. For the narrow partial-benefit case below, choose No and supply the retained comparison. Other loss, separate AMT or undeducted-tax histories need another recovery worksheet. Check the retained 2025 return and fuel expense records. No without a qualified comparison includes the full credited fuel tax as recovered income.
    pub full_prior_tax_benefit: bool,
    /// Documented partial 2025 regular-income-tax benefit from deducted fuel tax
    /// With No above, this retained comparison of the filed prior return with and without the fuel-tax expense limits the recovered income. No without a qualified comparison includes the whole credit with a notice.
    /// [NullText]
    /// Use the full recovery; no partial-benefit comparison is supplied
    /// [SetText]
    /// The documented partial prior-year tax benefit is
    #[libertas_default($null)]
    pub partial_prior_year_benefit: Option<crate::FederalPartialCashFuelTaxBenefit>,
    /// The earlier fuel-cost deduction did not reduce net investment income tax
    /// If it reduced NIIT, Form 8960 requires a separate deduction-recovery worksheet even when this activity is nonpassive in 2026. Compare the 2025 Form 8960 with and without that deduction. No includes the recovery for regular tax and leaves its net investment income recovery out with a notice.
    pub prior_niit_recovery_excluded: bool,
    /// This 2026 Schedule E recovery is absent from current K-1 box 1 and other income entries
    /// Compare the K-1 statement with your other entries. No means it is already included, so it is not added again.
    pub not_reported_elsewhere: bool,
}

impl PartnerPriorFuelCredit {
    pub(crate) fn valid(&self) -> bool {
        // Arithmetic safety: extended_rules adds this recovery to native K-1
        // income; partnership_qbi and federal_rules transfer the resulting
        // form dollars back to cents. Widen with those actual receivers.
        self.validation_issue("").is_none()
    }

    pub(crate) fn validation_issue(
        &self,
        key: &'static str,
    ) -> Option<crate::validation::ValidationIssue> {
        let field = if !(1..=100_000_000_000).contains(&self.claimed_credit) {
            "claimed_credit"
        } else if self
            .partial_prior_year_benefit
            .as_ref()
            .is_some_and(|p| !p.valid(self.claimed_credit))
        {
            "partial_prior_year_benefit"
        } else {
            return None;
        };
        Some(crate::validation::ValidationIssue::new(key).at_field(field))
    }

    pub(crate) fn partial_benefit_applies(&self) -> bool {
        !self.full_prior_tax_benefit
            && self
                .partial_prior_year_benefit
                .as_ref()
                .is_some_and(|p| p.valid(self.claimed_credit) && p.qualified())
    }

    pub(crate) fn taxable_amount(&self) -> i64 {
        if self.partial_benefit_applies() {
            self.partial_prior_year_benefit.as_ref().unwrap().taxable_amount(self.claimed_credit)
        } else {
            self.claimed_credit
        }
    }

    // A recovery this route reports: claimed on a 2025 return filed in 2026
    // by a cash-method activity, and not already in another entry. Others
    // are left out before the rules run.
    pub(crate) fn in_scope(&self) -> bool {
        self.filed_in_return_year && self.cash_method && self.not_reported_elsewhere
    }
}

/// Partnership interest, dividends, capital gains and cash gifts
/// Enter each box from your K-1 as shown; enter zero for a blank box. These amounts also change your outside basis and amount at risk in the partnership.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct PartnerSeparateItems {
    /// Interest income — box 5
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub interest: i64,
    /// Ordinary dividends — box 6a
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub dividends: i64,
    /// Qualified dividends — box 6b
    /// The part of box 6a taxed at capital-gain rates.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub qualified_dividends: i64,
    /// Box 7 royalties
    /// Reported on Schedule E line 4. Box 13 code I deductions belong in the deductions list.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub royalties: i64,
    /// Net short-term capital gain or loss — box 8
    /// Enter a loss as a negative amount.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub short: i64,
    /// Net long-term capital gain or loss — box 9a
    /// Enter a loss as a negative amount. Include the collectibles and unrecaptured section 1250 amounts below; they are parts of this box, not additional gains.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub long: i64,
    /// Collectibles (28%) gain or loss — box 9b
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub collectibles: i64,
    /// Unrecaptured section 1250 gain from selling a partnership interest — box 9c statement
    /// The part of box 9c the partnership's statement shows as coming from the sale of an interest in another partnership.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub unrecaptured_partnership: i64,
    /// Unrecaptured section 1250 gain from estates, trusts, mutual funds or REITs — box 9c statement
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub unrecaptured_fund: i64,
    /// Cash contributions (60%) — box 13 code A
    /// The partnership's cash gifts to charities, deducted on Schedule A like your own cash gifts.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub cash_charity: i64,
    /// Backup withholding — box 15 code O
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub backup_withholding: i64,
    /// Tax-exempt interest income — box 18 code A
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub exempt_interest: i64,
    /// Foreign taxes paid or accrued — box 21
    /// They reduce your basis. The credit is figured from the Schedule K-3.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub foreign_taxes: i64,
}

impl PartnerSeparateItems {
    pub(crate) const INVALID: &'static str = crate::fixture_english!(
        "Review the separately stated Schedule K-1 (1065) amounts: box 6b cannot exceed box 6a."
    );
}

/// Separately stated partnership amounts projected from accepted source records.
/// Coded-box totals retain every admitted source cent before partner-level limits.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PartnerSeparateTotals {
    /// Interest income — box 5
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub interest: i64,
    /// Ordinary dividends — box 6a
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub dividends: i64,
    /// Qualified dividends — box 6b
    /// The part of box 6a taxed at capital-gain rates.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub qualified_dividends: i64,
    /// Box 7 royalties
    /// Reported on Schedule E line 4. Box 13 code I deductions belong in the deductions list.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub royalties: i64,
    /// Net short-term capital gain or loss — box 8
    /// Enter a loss as a negative amount.
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    pub short: i64,
    /// Net long-term capital gain or loss — box 9a
    /// Enter a loss as a negative amount. Include the collectibles and unrecaptured section 1250 amounts below; they are parts of this box, not additional gains.
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    pub long: i64,
    /// Collectibles (28%) gain or loss — box 9b
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    pub collectibles: i64,
    /// Unrecaptured section 1250 gain from selling a partnership interest — box 9c statement
    /// The part of box 9c the partnership's statement shows as coming from the sale of an interest in another partnership.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub unrecaptured_partnership: i64,
    /// Unrecaptured section 1250 gain from estates, trusts, mutual funds or REITs — box 9c statement
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub unrecaptured_fund: i64,
    /// Cash contributions (60%) — box 13 code A
    /// The partnership's cash gifts to charities, deducted on Schedule A like your own cash gifts.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub cash_charity: i64,
    /// Backup withholding — box 15 code O
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub backup_withholding: i64,
    /// Tax-exempt interest income — box 18 code A
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub exempt_interest: i64,
    /// Foreign taxes paid or accrued — box 21
    /// They reduce your basis. The credit is figured from the Schedule K-3.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub foreign_taxes: i64,
}

impl PartnerSeparateTotals {
    pub(crate) fn valid(&self) -> bool {
        [
            self.interest,
            self.dividends,
            self.qualified_dividends,
            self.royalties,
            self.unrecaptured_partnership,
            self.unrecaptured_fund,
            self.cash_charity,
            self.backup_withholding,
            self.exempt_interest,
            self.foreign_taxes,
        ]
        .into_iter()
        .all(|amount| amount >= 0)
            && self.qualified_dividends <= self.dividends
    }

    // Basis worksheet line 4: the positive separately stated items.
    fn basis_income(&self) -> Option<i64> {
        [
            self.interest,
            self.dividends,
            self.royalties,
            phase::floor(self.short, 0),
            phase::floor(self.long, 0),
            self.exempt_interest,
        ]
        .into_iter()
        .try_fold(0i64, i64::checked_add)
    }

    // Taxable portfolio income and capital gains of the activity (Form 6198).
    fn risk_income(&self) -> Option<i64> {
        [
            self.interest,
            self.dividends,
            self.royalties,
            phase::floor(self.short, 0),
            phase::floor(self.long, 0),
        ]
        .into_iter()
        .try_fold(0i64, i64::checked_add)
    }
}

/// Other K-1 box with an amount
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum PartnerK1Item {
    /// Rental property disposed of by installment sale, exchange, casualty or theft, or to a related buyer
    /// Reported with box 2 or 3 on the partnership's statement, in box 10 or in box 20.
    RentalDisposition,
    /// Dividend equivalents or other income — box 6c or box 11 codes not calculated
    OtherIncome,
    /// Deductions — box 13 code O
    OtherDeductions,
    /// Credits — box 15 codes not entered above
    /// Your share of credits this App does not figure. They are left out of the estimate; the rest of the K-1 is calculated.
    Credits,
    /// Carbon oxide sequestration credit recapture — box 15 code AX
    CreditRecapture,
    /// Oil and gas depletion information — box 20 code T
    /// Your depletion deduction is left out of the estimate; the rest of the K-1 is calculated.
    Depletion,
    /// Alternative minimum tax items — box 17
    /// Only items the separate AMT records do not already include.
    AmtItems,
    /// Property distributions — box 19 code B or C
    PropertyDistribution,
    /// Other information — box 20 codes that report an amount the App does not calculate
    OtherInformation,
    /// Cooperative items on the section 199A statement — box 20 code Z
    CooperativeQbi,
    /// Separate activity allocations — box 22 or box 23
    /// A checked box needs the distinct-activity statement document. This single-paper document also retains statements identifying more than one activity of the same kind.
    MultipleActivities,
}

impl PartnerK1Item {
    pub(crate) fn left_out(self) -> bool {
        self.item().is_some()
    }
    pub(crate) fn item(self) -> Option<&'static str> {
        match self {
            Self::Credits => Some("K1_ITEM_PARTNER_CREDITS"),
            Self::Depletion => Some("K1_ITEM_PARTNER_DEPLETION"),
            _ => None,
        }
    }
    pub(crate) fn refusal(self) -> &'static str {
        self.refusal_info().1
    }
    pub(crate) fn refusal_key(self) -> &'static str {
        self.refusal_info().0
    }
    pub(crate) fn refusal_info(self) -> (&'static str, &'static str) {
        match self {
            Self::RentalDisposition => (
                "ISSUE_PARTNER_K1_RENTAL_DISPOSITION",
                crate::fixture_english!(
                    "A partnership rental activity's installment sale, like-kind exchange, casualty or theft, or disposition to a related buyer needs separate property and passive-loss worksheets that this App does not calculate."
                ),
            ),
            Self::OtherIncome => (
                "ISSUE_PARTNER_K1_OTHER_INCOME",
                crate::fixture_english!(
                    "Schedule K-1 (1065) box 6c dividend equivalents, box 11 codes B, D, I, L, M, N, O, P and ZZ, a code A portfolio loss or REMIC residual interest, a code C straddle, a code F section 743(b) adjustment to another item and gambling of a gambling business are not calculated."
                ),
            ),
            Self::OtherDeductions => (
                "ISSUE_PARTNER_K1_OTHER_DEDUCTIONS",
                crate::fixture_english!(
                    "Schedule K-1 (1065) box 13 code O dependent care benefits (Form 2441 Part III) and a code V section 743(b) adjustment to another item are not calculated."
                ),
            ),
            Self::Credits | Self::Depletion => {
                unreachable!("left-out K-1 benefits are removed before calculation")
            }
            Self::CreditRecapture => (
                "ISSUE_K1_CREDIT_RECAPTURE",
                crate::fixture_english!(
                    "Carbon oxide sequestration credit recapture (K-1 code AX, Form 8933 Part III) is not calculated."
                ),
            ),
            Self::AmtItems => (
                "ISSUE_PARTNER_K1_AMT_ITEMS",
                crate::fixture_english!(
                    "Schedule K-1 (1065) box 17 alternative minimum tax items beyond the separate AMT activity records (Form 6251) are not calculated."
                ),
            ),
            Self::PropertyDistribution => (
                "ISSUE_PARTNER_K1_PROPERTY_DISTRIBUTION",
                crate::fixture_english!(
                    "Schedule K-1 (1065) box 19 codes B and C property distributions (sections 732 and 737) are not calculated."
                ),
            ),
            Self::OtherInformation => (
                "ISSUE_PARTNER_K1_OTHER_INFORMATION",
                crate::fixture_english!(
                    "Schedule K-1 (1065) box 20 investment income or expense totals requiring separate Form 4952 reconciliation (codes A and B), recapture, look-back interest, section 179 dispositions other than fully taxable sales, installment and section 1260 interest, precontribution gain, section 1061, inversion gain and section 48A–48D items (codes F through M, O through S, W, AL for a U.S. transferor, AM, AP, AS through AU, and ZZ with an amount) are not calculated."
                ),
            ),
            Self::CooperativeQbi => (
                "ISSUE_K1_COOPERATIVE_QBI",
                crate::fixture_english!(
                    "A section 199A statement with items from a specified agricultural or horticultural cooperative (Form 8995-A Schedule D patron reduction and the section 199A(g) deduction) is not calculated."
                ),
            ),
            Self::MultipleActivities => (
                "ISSUE_PARTNER_K1_MULTIPLE_ACTIVITIES",
                crate::fixture_english!(
                    "Schedule K-1 (1065) box 22, more than one activity for at-risk purposes, or a box 23 statement with more than one trade or business, rental real estate or other rental activity (Form 6198 and Form 8582 by activity) is not calculated."
                ),
            ),
        }
    }
}

// Box 13 code A cash gifts of every partnership K-1 on the return that the
// outside basis allows (this year's gift plus a released carry), in cents.
pub(crate) fn cash_charity(d: &crate::FederalDraft) -> i64 {
    d.income
        .iter()
        .filter_map(|e| match &e.income {
            crate::FederalIncome::Partnership { data } => Some(data.allowed_cash_charity()),
            _ => None,
        })
        .sum()
}

/// Partnership K-1 ordinary-business records
/// Form 1065, not Form 1120-S. Preserve the payer's documents and separate tax-system histories. Additional items need their own calculation before a complete return can be produced.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PartnerK1 {
    /// Actual paper source, when this view came from an entered document.
    /// Source records are stored through the ordinary income-record projection.
    #[libertas_default($null)]
    pub source: Option<alloc::boxed::Box<crate::PartnershipSource>>,
    /// Your eligible-small-business treatment for the partnership research credit.
    #[libertas_default($null)]
    pub research_credit_line: Option<crate::BusinessCredit>,
    /// The accepted AMT-record choice in a transient source projection.
    /// The source record stores the personal AMT enum; this tag retains its
    /// choice when separate records happen to equal regular records.
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default($null)]
    pub amt_record_choice: Option<bool>,
    /// Entity's reporting tax period
    /// The actual beginning and ending dates of the entity year reported by this K-1. Leave unknown when its source records do not establish the period.
    pub reporting_period: Option<crate::K1EntityTaxPeriod>,
    pub regular: PartnerActivityRecord,
    /// What produces your partnership income
    /// Needed for young-filer personal-service earnings. Use your own services, not other partners' work. Independent of passive participation and K-1 box 14. Absent or unresolved is taken as capital (only the guaranteed service payment is earned income), with a note. Current cash guaranteed service and capital payments are coordinated; historical-loss and special-payment combinations need further work.
    pub income_source: Option<crate::BusinessIncomeSource>,
    /// Your gross income from a services-only partnership business
    /// For Form 8615 and the dependent standard deduction when your personal services produced the income and capital was not material. Use your allocable gross income before ordinary business expenses, supported by partnership records. Exclude the separately entered guaranteed service payment; the App adds that payment once. K-1 box 1 net income or box 14 SE earnings is not a substitute. Absent, the income is taken as capital-produced, with a note. Other income-source classes do not use this field.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub gross_personal_services: Option<i64>,
    /// Guaranteed payments for personal services — K-1 box 4a
    /// Current-year compensation for services actually rendered, including partnership-paid or reimbursed health premiums reported as service payments, also included in box 14 code A. Keep separate from box 1, basis contributions and distributions. Enter zero when none. Enter capital payments separately. Deferred compensation and retirement/liquidation payments require additional records and are outside this source.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub guaranteed_services: i64,
    /// Guaranteed payments for use of capital — K-1 box 4b
    /// Current-year cash payments for capital used by this continuing U.S. trade or business. Keep separate from box 1, loans, contributions and distributions. Enter zero when none. Exclude deferred compensation, retiring-partner payments, property payments and amounts from nonbusiness investment activities; those require additional records. General-partner SE treatment includes this amount in box 14 code A; limited-partner SE treatment excludes it.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub guaranteed_capital: i64,
    /// Separate AMT activity and retained records
    /// Leave absent if not established. Absence never means the AMT adjustment is zero.
    pub alternative: Option<PartnerActivityRecord>,
    /// [NullText]
    /// Participation is not established yet
    /// [SetText]
    /// Participation is
    pub participation: Option<PartnerParticipation>,
    /// Business classification for passive losses and investment tax
    pub trade_kind: crate::ScorpTradeKind,
    /// Limited-partner treatment under section 1402(a)(13) is established
    /// This is a separate self-employment classification, not the section 469 passive classification above or an automatic result of an LLC label. The K-1's general or limited partner checkbox and the partnership agreement show it.
    /// [NullText]
    /// Self-employment classification is not established yet
    /// [SetText]
    /// Limited-partner treatment for self-employment tax applies
    pub limited_for_se: Option<bool>,
    /// Net earnings or loss from self-employment, K-1 box 14 code A
    /// Enter the reported amount before partner-level adjustments and the Schedule SE percentage. Enter zero only when established from the K-1 and accompanying statements; leave absent when unresolved. This is not box 1 income, a cash distribution or the final Schedule SE result. A reported loss remains subject to deductible-loss limits.
    #[libertas_money("USD", 2)]
    pub reported_se: Option<i64>,
    /// Retained losses that can affect self-employment earnings
    /// From your 2025 basis, at-risk and passive loss worksheets for this partnership: the self-employment part of each ordinary loss carried to 2026. Zero for each part when nothing was carried or none of it was self-employment loss.
    pub se_history: PartnerSeHistory,
    /// Gross self-employment income and business category
    /// Used for optional-method elections and gross-income allocation of the QBI half-SE deduction, including activities left on the actual method. Missing information is not zero gross income.
    /// [NullText]
    /// No gross self-employment source is available
    /// [SetText]
    /// The actual gross self-employment source is
    pub se_gross: Option<PartnerSeGross>,
    /// Ordinary-business section 199A statement
    /// The K-1's box 20 code Z statement. Absent only when the K-1 came with no such statement; current-year qualified business income or loss is then left out of the QBI deduction with a notice. Retained suspended losses keep their original QBI status and count when allowed.
    pub qbi: Option<crate::PartnerQbiStatement>,
    /// Retained QBI loss and gross-income allocation records
    /// Preserved even when no section 199A statement is present.
    pub qbi_records: crate::partnership_qbi::PartnerQbiRecords,
    /// Partner health insurance
    /// Enter premiums once, whether personally paid, paid from distributions under partnership-named policies, or paid/reimbursed by the partnership and taxed as service payments. Personally paid premiums without reimbursement require a policy in the partnership's name. A policy in your own name requires partnership reimbursement and guaranteed-payment reporting. Confirm these establishment rules in the plan's eligibility answer. Do not repeat these premiums in itemized expenses.
    pub health_insurance: crate::BusinessHealthInsurance,
    /// Included taxable health premiums paid or reimbursed by the partnership
    /// The part of the entered health and qualified long-term-care premiums already included in guaranteed service payments and box 14 code A. This is a subset, not additional income or an additional premium. Include the full taxable payment before employer-coverage exclusions, age caps and income limits. Enter zero for personally paid premiums not reimbursed. Enter premiums charged against distributions separately below.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub health_guaranteed: i64,
    /// Included health premiums charged against cash distributions
    /// Premiums under partnership-named policies paid on your behalf and charged against your distributions, not deducted by the partnership or reported as guaranteed income. Include them in the coverage records and in cash distributions in both regular and AMT basis records. This is a subset of those amounts, not another distribution or premium. Do not include these premiums in the taxable health subset above. Enter zero when none; partner-named policies require reimbursement and guaranteed-payment reporting instead.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub health_distributions: i64,
    /// Pensions and IRAs — K-1 box 13 code R
    /// Payments the partnership made for you to an IRA, a qualified plan, a SEP or a SIMPLE IRA. The plan is answered below. Enter zero when the box is empty.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub pensions: i64,
    /// Plan the box 13 code R payment went to
    pub retirement: crate::PartnerRetirement,
    /// Fuel tax credit statement — K-1 box 20 code C
    /// This App calculates the partner's Form 4136 payment from the reported gallons. An earlier credit creating current Schedule E income belongs in the separate prior-credit field below. Do not add this current credit to box 1 income.
    pub fuel_tax_credit: Option<PartnerFuelTaxCredit>,
    /// Prior-year partner fuel credit recognized on 2026 Schedule E
    /// Enter only for a cash-method 2025 Form 4136 credit claimed on a return filed in 2026. The qualified retained partial-benefit comparison limits recovery; otherwise the whole credit is included. Keep it separate from current K-1 gallons and box 1 income.
    pub prior_fuel_credit: Option<PartnerPriorFuelCredit>,
    /// Interest, dividends, capital gains, cash gifts and tax-exempt interest
    /// Leave absent when the K-1 has none of these boxes.
    pub separate: Option<PartnerSeparateTotals>,
    /// Schedule K-3 — K-1 box 16
    /// Leave absent when box 16 is not checked.
    pub schedule_k3: Option<crate::ScheduleK3>,
    /// Your facts about the rental activities — boxes 2 and 3
    /// The amounts and carryovers are in each tax system's activity record.
    pub rental_facts: crate::K1RentalFacts,
    /// Section 179 deduction — box 12
    /// Your share of the partnership's section 179 expense. It joins your own elections on Form 4562 Part I under your dollar and business-income limits, reduces your outside basis in full this year even when part is carried to 2027, and reduces your box 14 code A earnings as a general partner. Enter zero when the box is empty.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub section179: i64,
    /// Section 179 deduction from this K-1 you could not deduct in 2025
    /// The unused part of this partnership's earlier section 179 amounts carried from your 2025 return because of the business income limit. It already reduced your basis in the partnership; it joins this year's section 179 under this year's limits.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub section179_carryover: i64,
    /// Part of this section 179 carryover belonging to property the entity sold
    /// From your own Form 4562 carryover records, not the K-1. This amount restores the sold property's basis and is no longer available as a deduction.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub section179_carryover_sold: i64,
    /// Other boxes on this K-1 with an amount
    /// ----
    /// Box with an amount
    /// General business credits — box 15
    /// Your share of the low-income housing, work opportunity, tip, small employer retirement plan, health insurance and paid family and medical leave credits. They reach Form 3800 Part III with this partnership's EIN; other box 15 codes belong among the other boxes.
    /// ----
    /// Credit
    pub credits: alloc::vec::Vec<crate::K1Credit>,
    /// Investment credit property (Form 3468)
    pub investment_property: alloc::vec::Vec<crate::K1InvestmentProperty>,
    /// Investment credit property confirmed
    pub investment_confirmed: Option<bool>,
    /// Low-income housing interest acquired after 1989 (Some(false) selects the pre-1990 rules)
    pub housing_confirmed: Option<bool>,
    pub unentered: alloc::vec::Vec<PartnerK1Item>,
    /// Final K-1
    pub final_k1: bool,
    /// Other income — box 11
    // Tax structure: one row per distinct Form 1065 income code in the fixed catalog.
    #[libertas_size(max = 12)]
    /// ----
    /// Box 11 amount
    pub income: alloc::vec::Vec<crate::PartnerIncomeTotal>,
    /// Other deductions — box 13 codes other than A, M, O and R
    // Tax structure: one row per distinct Form 1065 deduction code in the fixed catalog.
    #[libertas_size(max = 25)]
    /// ----
    /// Box 13 amount
    pub deductions: alloc::vec::Vec<crate::PartnerDeduction>,
    /// Interest and additional tax on deferred compensation — box 20 code AI
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub deferred_compensation_tax: i64,
    /// Conservation reserve program payments — box 20 code AQ
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub crp_payments: i64,
    /// How long you have held your interest
    /// Needed only when box 19 cash exceeds your basis or the interest was sold. Leave absent otherwise.
    #[libertas_default($null)]
    pub interest_term: Option<crate::GainTerm>,
    /// What happened to the interest in 2026
    /// [NullText]
    /// What happened to this interest is not answered yet
    /// [SetText]
    /// What happened to this interest is
    pub disposition: Option<crate::K1Disposition>,
    /// Parts of the gain on the sale — box 20 codes AB, AC and AD
    pub sale: crate::PartnerSaleParts,
    /// Situations this App does not calculate
    // Tax structure: one row per distinct partnership special situation in the fixed catalog.
    #[libertas_size(max = 5)]
    /// ----
    /// Situation
    pub special: alloc::vec::Vec<crate::PartnerSpecial>,
    /// This is a publicly traded partnership under section 7704(b)
    /// Its section 199A statement reports current-year qualified income or loss and the actual specified-service classification, with no wages or unadjusted property basis. Apply current specified-service income limits once before eligible income or loss joins qualified REIT dividends (Form 8995 line 6 / Form 8995-A line 28), without a wage/UBIA limit or aggregation, and carries forward with them. Historical qualified-loss statements and suspended basis/at-risk/passive balances still need their actual source worksheets. Separately reported PTP items need their actual business and specified-service source classification. Leave false for an ordinary (non-PTP) partnership interest.
    pub publicly_traded: bool,
    /// Identified section179 sale properties with regular and AMT monetary views
    #[libertas_unordered]
    pub section179_sales: alloc::vec::Vec<crate::K1Section179Sale>,
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

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct PartnerPoolResult {
    pub activity: PartnerOrdinaryResult,
    /// Original annual-Dollar operands after basis/risk, before passive netting.
    /// Ordinary includes the same allowed activity items as `combined`.
    pub current_ordinary_with_items: i64,
    pub current_section1231: i64,
    pub characters: Option<crate::scorp::ActivityCharacters>,
    pub nonpassive: i64,
    pub passive: i64,
    pub prior_released: i64,
    // The prior unallowed loss that remains in the passive pool (Form 8582
    // Part V column (c)).
    pub prior_pool: i64,
    // Unit B9-1: the activity items (code E, R and S) after the basis and
    // at-risk limits, and the same four results of the activity with them
    // (Reg. 1.469-2T(c)(2)); equal to the fields above without any.
    pub items: crate::k1_items::ActivityItems,
    pub combined: PooledNets,
}

// The passive result of an activity: the part that is nonpassive (income
// that releases the old loss, or all of it once the whole interest is
// disposed of, section 469(g)(1)(A)), the part still passive, the old loss
// released and the old loss left in the pool.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PooledNets {
    pub nonpassive: i64,
    pub passive: i64,
    pub prior_released: i64,
    pub prior_pool: i64,
    pub characters: Option<crate::scorp::ActivityCharacters>,
}

impl PooledNets {
    pub(crate) fn of(net: i64, prior: i64, nonpassive: bool, disposed: bool) -> Self {
        let released = if disposed {
            prior
        } else if nonpassive {
            phase::cap(prior, [phase::floor(net, 0)])
        } else {
            0
        };
        let (nonpassive, passive) = if nonpassive || disposed {
            (net - released, -(prior - released))
        } else {
            (0, net - prior)
        };
        Self {
            nonpassive,
            passive,
            prior_released: released,
            prior_pool: prior - released,
            characters: None,
        }
    }
}
impl PartnerActivityRecord {
    pub(crate) fn sale_results(
        &self,
        items: &PartnerItems,
    ) -> Result<Vec<crate::k1_items::Section179SaleResult>, (&'static str, &'static str)> {
        let sales =
            crate::K1Section179Sale::views(items.section179_sales, items.alternative_sales)?;
        crate::K1Section179Sale::worksheets(
            &sales,
            items.section179_carryover,
            items.section179_carryover_sold,
        )
    }

    pub(crate) fn business_property(
        &self,
        items: &PartnerItems,
    ) -> Result<Option<crate::k1_items::BusinessProperty>, (&'static str, &'static str)> {
        Ok(crate::k1_items::BusinessProperty::of(
            self.section1231.as_ref(),
            &self.sale_results(items)?,
        ))
    }

    pub(crate) fn source_k1_items(
        &self,
        items: &PartnerItems,
    ) -> Result<crate::k1_items::K1Items, (&'static str, &'static str)> {
        items.k1().checked_with_sales(&self.sale_results(items)?).ok_or(PartnerPlan::RANGE)
    }

    // The class input of this record in its tax system (unit B8): this
    // year's share of every loss and deduction, and what the record carries
    // because of the basis and at-risk limits (partner basis worksheet Part
    // II columns A and B; Form 6198 lines 1, 2a and 4).
    pub(crate) fn plan(
        &self,
        items: &PartnerItems,
    ) -> Result<PartnerPlan, (&'static str, &'static str)> {
        if !self.basis.complete
            || !self.basis.valid_amounts(self.ordinary, self.exempt, self.nondeductible)
            // Arithmetic safety: the plan's native class totals and downstream
            // k1_totals dollar-to-cent transfers retain these source bounds.
            || !(0..=100000000000).contains(&items.section179)
            || !(0..=100000000000).contains(&items.credit_reduction)
            || !self.rentals.valid()
            || !self.carried.valid()
            || !self.capital_carried.valid()
            || self.section1231.as_ref().is_some_and(|v| !v.valid())
        {
            return Err(PartnerPlan::INCOMPLETE);
        }
        use K1LossClass::*;
        let mut plan = PartnerPlan {
            property_disposal: items.property_disposal,
            credit_reduction: items.credit_reduction,
            section179_cents: items.section179,
            ..PartnerPlan::default()
        };
        // Unit B9-1: code F and code V adjust the box 1 business, so they are
        // part of box 1 for every purpose here.
        let k1 = self.source_k1_items(items)?;
        let ordinary = self
            .ordinary
            .checked_add(k1.checked_box_one().ok_or(PartnerPlan::RANGE)?)
            .ok_or(PartnerPlan::RANGE)?;
        let input = &mut plan.input;
        input.current[Ordinary] = Amount::<Cents>::new(if ordinary < 0 {
            ordinary.checked_neg().ok_or(PartnerPlan::RANGE)?
        } else {
            0
        })
        .expect("nonnegative ordinary loss");
        input.basis_carry[Ordinary] =
            Amount::<Cents>::new(self.basis.prior_loss).expect("validated source class cents");
        input.risk_carry[Ordinary] =
            Amount::<Cents>::new(self.risk.prior_loss).expect("validated source class cents");
        input.current[Section179] =
            Amount::<Cents>::new(items.section179).expect("validated source class cents");
        let [real_estate, other] = self.rentals.losses();
        input.current[RentalRealEstate] =
            Amount::<Cents>::new(real_estate).expect("validated source class cents");
        input.current[OtherRental] =
            Amount::<Cents>::new(other).expect("validated source class cents");
        plan.basis_income = self
            .rentals
            .income()
            .checked_add(k1.checked_income().ok_or(PartnerPlan::RANGE)?)
            .ok_or(PartnerPlan::RANGE)?;
        plan.risk_income = plan.basis_income;
        if let Some(property) = self.business_property(items)? {
            input.current[Section1231] =
                Amount::<Cents>::new(property.loss).expect("validated source class cents");
            input.basis_carry[Section1231] =
                Amount::<Cents>::new(property.basis_loss).expect("validated source class cents");
            input.risk_carry[Section1231] =
                Amount::<Cents>::new(property.risk_loss).expect("validated source class cents");
            plan.basis_income =
                plan.basis_income.checked_add(property.gain).ok_or(PartnerPlan::RANGE)?;
            plan.risk_income =
                plan.risk_income.checked_add(property.gain).ok_or(PartnerPlan::RANGE)?;
        }
        input.current[Section1231] = Amount::<Cents>::new(
            input.current[Section1231].get() + (self.rentals.property_losses().iter().sum::<i64>()),
        )
        .expect("validated source class cents");
        let code_losses = k1.checked_current().ok_or(PartnerPlan::RANGE)?;
        input.current = input.current.checked_plus(&code_losses).ok_or(PartnerPlan::RANGE)?;
        plan.ebie = k1.checked_ebie().ok_or(PartnerPlan::RANGE)?;
        match items.disposition {
            Some(crate::K1Disposition::Sold { proceeds }) => {
                plan.disposal = Some(DisposalInput {
                    proceeds: Some(*proceeds),
                    relief: self.basis.ending_liabilities,
                    ordinary: items.sale.map_or(0, |v| v.ordinary),
                });
            }
            Some(crate::K1Disposition::LiquidatedForCash) => {
                plan.deemed_money = self.basis.ending_liabilities;
                plan.disposal =
                    Some(DisposalInput { proceeds: None, relief: plan.deemed_money, ordinary: 0 });
            }
            _ => {}
        }
        if let Some(separate) = items.separate {
            if !separate.valid() {
                return Err((
                    "ISSUE_PARTNER_SEPARATE_ITEMS_RECONCILIATION",
                    PartnerSeparateItems::INVALID,
                ));
            }
            let short_loss = separate.short.checked_neg().ok_or(PartnerPlan::RANGE)?;
            let long_loss = separate.long.checked_neg().ok_or(PartnerPlan::RANGE)?;
            input.current[ShortTerm] = Amount::<Cents>::new(phase::floor(short_loss, 0))
                .expect("validated source class cents");
            input.current[LongTerm] = Amount::<Cents>::new(phase::floor(long_loss, 0))
                .expect("validated source class cents");
            input.current[Charity] =
                Amount::<Cents>::new(separate.cash_charity).expect("validated source class cents");
            input.current[ForeignTaxes] =
                Amount::<Cents>::new(separate.foreign_taxes).expect("validated source class cents");
            plan.basis_income = separate
                .basis_income()
                .and_then(|income| plan.basis_income.checked_add(income))
                .ok_or(PartnerPlan::RANGE)?;
            plan.risk_income = separate
                .risk_income()
                .and_then(|income| plan.risk_income.checked_add(income))
                .ok_or(PartnerPlan::RANGE)?;
            plan.risk_exempt = separate.exempt_interest;
        }
        input.basis_carry =
            input.basis_carry.plus(&self.carried.basis()).plus(&self.capital_carried.basis());
        input.risk_carry =
            input.risk_carry.plus(&self.carried.risk()).plus(&self.capital_carried.risk());
        Ok(plan)
    }

    // The basis and at-risk stages of this record with the K-1's shared
    // boxes: what each limit allows and holds back, by class.
    pub(crate) fn activity_result(
        &self,
        items: &PartnerItems,
    ) -> Result<PartnerOrdinaryResult, (&'static str, &'static str)> {
        let plan = self.plan(items)?;
        self.risk.calculate_with(
            &self.basis,
            self.ordinary
                .checked_add(items.k1().checked_box_one().ok_or(PartnerPlan::RANGE)?)
                .ok_or(PartnerPlan::RANGE)?,
            self.exempt,
            self.nondeductible,
            &plan,
        )
    }

    #[cfg(test)]
    pub(crate) fn pool_result(
        &self,
        material: crate::participation::Participation,
    ) -> Result<PartnerPoolResult, (&'static str, &'static str)> {
        self.pool_result_with(material, &PartnerItems::default())
    }

    pub(crate) fn pool_result_with(
        &self,
        material: crate::participation::Participation,
        items: &PartnerItems,
    ) -> Result<PartnerPoolResult, (&'static str, &'static str)> {
        // Arithmetic safety: pool_result_with combines this history with
        // native current classes before the cents transfer.
        if !(0..=100000000000).contains(&self.passive_loss) {
            return Err((
                "ISSUE_PARTNERSHIP_REVIEW_THE_RETAINED_ORDINARY_PASSIVE_LOSS_BALANCE_1389FA9F",
                crate::fixture_english!("Review the retained ordinary passive-loss balance."),
            ));
        }
        let activity = self.activity_result(items)?;
        if activity.recapture.income.get() != 0 {
            return Err((
                "ISSUE_PARTNERSHIP_PARTNER_AT_RISK_RECAPTURE_REQUIRES_SEPARATE_RETURN_B3B090A9",
                crate::fixture_english!(
                    "Partner at-risk recapture requires separate return and character coordination."
                ),
            ));
        }
        let net = activity.income.get() - activity.ledger.risk_allowed[K1LossClass::Ordinary].get();
        let prior = self.passive_loss;
        let nonpassive = match material {
            crate::participation::Participation::Yes => true,
            crate::participation::Participation::No => false,
            crate::participation::Participation::Unavailable => {
                return Err((
                    "ISSUE_PARTNERSHIP_ESTABLISH_THE_PARTNERSHIP_BUSINESS_CLASSIFICATION_AND_2561E41A",
                    crate::fixture_english!(
                        "Establish the partnership business classification and material participation before applying passive limits."
                    ),
                ));
            }
        };
        let disposed = items.ended();
        let rental_property_losses =
            self.rentals.property_allowed(&activity.ledger)?.iter().sum::<i64>();
        let plain = PooledNets::of(net, prior, nonpassive, disposed);
        let activity_items = self
            .source_k1_items(items)?
            .checked_flows(&activity.ledger)
            .ok_or(PartnerPlan::RANGE)?
            .activity();
        let current_ordinary_with_items =
            activity_items.checked_net_with(net).ok_or(PartnerPlan::RANGE)?;
        let combined = PooledNets::of(current_ordinary_with_items, prior, nonpassive, disposed);
        if let Some(property) = self.business_property(items)? {
            let current = property.gain
                - (activity.ledger.risk_allowed[K1LossClass::Section1231].get()
                    - rental_property_losses);
            let property_prior = property.passive_loss;
            let characters = crate::scorp::ActivityCharacters::from_current_and_prior(
                [net, current],
                [prior, property_prior],
                nonpassive,
                disposed,
            )?;
            let with_items = crate::scorp::ActivityCharacters::from_current_and_prior(
                [current_ordinary_with_items, current],
                [prior, property_prior],
                nonpassive,
                disposed,
            )?;
            // These four fields remain ordinary-only for SE and ordinary QBI.
            // The combined pool carries both forms; Form 4797 removes its own
            // character after the shared passive allocation.
            return Ok(PartnerPoolResult {
                activity,
                current_ordinary_with_items,
                current_section1231: current,
                characters: Some(characters),
                nonpassive: characters.nonpassive[0],
                passive: characters.passive[0],
                prior_released: characters.prior_released[0],
                prior_pool: prior - characters.prior_released[0],
                items: activity_items,
                combined: PooledNets {
                    nonpassive: with_items.nonpassive.iter().sum(),
                    passive: with_items.passive.iter().sum(),
                    prior_released: with_items.prior_released.iter().sum(),
                    prior_pool: prior + property_prior
                        - with_items.prior_released.iter().sum::<i64>(),
                    characters: Some(with_items),
                },
            });
        }
        Ok(PartnerPoolResult {
            activity,
            current_ordinary_with_items,
            current_section1231: 0,
            characters: None,
            nonpassive: plain.nonpassive,
            passive: plain.passive,
            prior_released: plain.prior_released,
            prior_pool: plain.prior_pool,
            items: activity_items,
            combined,
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HealthPaymentError {
    TaxablePremiums,
    DistributionPremiums,
}

impl HealthPaymentError {
    pub(crate) fn message(self) -> (&'static str, &'static str) {
        match self {
            Self::TaxablePremiums => (
                "ISSUE_PARTNER_TAXABLE_HEALTH_PREMIUMS",
                crate::fixture_english!(
                    "Taxable partnership-paid health premiums must be included in both the entered premium records and guaranteed service payments, without adding either amount twice."
                ),
            ),
            Self::DistributionPremiums => (
                "ISSUE_PARTNER_DISTRIBUTION_HEALTH_PREMIUMS",
                crate::fixture_english!(
                    "Distribution-paid health premiums must fit the coverage total after taxable premiums and be included once in both regular and AMT cash-distribution records."
                ),
            ),
        }
    }
}

impl PartnerK1 {
    const PERSONAL_NEEDED: (&'static str, &'static str) =
        ("DOCUMENT_K1_PARTNER_RECORDS", crate::fixture_english!("Your partnership records"));
    // One tax system's ordinary activity with this K-1's separately stated
    // items in the same outside basis and at-risk amount.
    pub(crate) fn pool(
        &self,
        source: &PartnerActivityRecord,
    ) -> Result<PartnerPoolResult, (&'static str, &'static str)> {
        self.pool_with_items(source, &self.items_for_record(source)?)
    }

    pub(crate) fn pool_with_items(
        &self,
        source: &PartnerActivityRecord,
        items: &PartnerItems,
    ) -> Result<PartnerPoolResult, (&'static str, &'static str)> {
        if self.participation.is_none() || self.limited_for_se.is_none() {
            return Err(Self::PERSONAL_NEEDED);
        }
        if self.disposition.is_none() {
            return Err(crate::k1_totals::DISPOSITION_NEEDED);
        }
        if self.rental_facts.participation.is_none() && self.rental_participation_needed() {
            return Err(crate::k1_rental::PARTICIPATION_NEEDED);
        }
        if self.publicly_traded && source.business_property(items)?.is_some() {
            return Err((
                "ISSUE_K1_PTP_PROPERTY",
                crate::fixture_english!(
                    "Business-property gains, losses and their carryovers in a publicly traded partnership need its separate property and passive-loss worksheets; they are not calculated."
                ),
            ));
        }
        source.pool_result_with(self.material_for_losses(), items)
    }

    // The shared boxes that join each tax system's basis and at-risk limits.
    // This selects the already bound regular/AMT record of this parent. It
    // never establishes physical-property correspondence: canonical sale IDs
    // own that, and both monetary views are on each actual source row.
    pub(crate) fn items_for_record(
        &self,
        source: &PartnerActivityRecord,
    ) -> Result<PartnerItems<'_>, (&'static str, &'static str)> {
        if core::ptr::eq(source, &self.regular) {
            Ok(self.items())
        } else if self.alternative.as_ref().is_some_and(|record| core::ptr::eq(source, record)) {
            Ok(self.amt_items())
        } else {
            Err((
                "K1_SALE_SOURCE_OWNER",
                "Use the regular or AMT record of this same accepted K-1 owner.",
            ))
        }
    }

    pub(crate) fn items(&self) -> PartnerItems<'_> {
        PartnerItems {
            separate: self.separate.as_ref(),
            section179: self.section179,
            section179_carryover: self.section179_carryover,
            section179_carryover_sold: self.section179_carryover_sold,
            section179_sales: &self.section179_sales,
            alternative_sales: false,
            credit_reduction: crate::investment_credit::k1_basis_decrease(
                &self.investment_property,
            ),
            income_codes: &self.income,
            deduction_codes: &self.deductions,
            disposition: self.disposition.as_ref(),
            sale: Some(&self.sale),
            property_disposal: None,
        }
    }

    pub(crate) fn amt_items(&self) -> PartnerItems<'_> {
        PartnerItems { alternative_sales: true, ..self.items() }
    }

    // Whether the activity items are in the passive pool: the activity is
    // passive and the whole interest was not disposed of (section 469(g)).
    pub(crate) fn items_in_pool(&self) -> bool {
        self.material_for_losses() != crate::participation::Participation::Yes
            && !self.disposition.as_ref().is_some_and(crate::K1Disposition::ended)
    }

    // The box 11 and box 13 codes of this K-1.
    pub(crate) fn k1_items(&self) -> crate::k1_items::K1Items {
        self.regular.source_k1_items(&self.items()).unwrap_or_else(|_| self.items().k1())
    }

    // What the codes and the year-end events of one tax system amount to
    // after the basis and at-risk limits. A K-1 whose limits cannot be
    // figured is refused elsewhere and counts its boxes whole.
    pub(crate) fn outcome(
        &self,
        source: &PartnerActivityRecord,
    ) -> Result<
        (crate::k1_items::K1ItemFlows, crate::k1_items::Disposal),
        (&'static str, &'static str),
    > {
        let items = self.items_for_record(source)?;
        self.outcome_with_items(source, &items)
    }

    pub(crate) fn outcome_with_items(
        &self,
        source: &PartnerActivityRecord,
        items: &PartnerItems<'_>,
    ) -> Result<
        (crate::k1_items::K1ItemFlows, crate::k1_items::Disposal),
        (&'static str, &'static str),
    > {
        match source.activity_result(items) {
            Ok(result) => Ok((
                source
                    .source_k1_items(items)?
                    .checked_flows(&result.ledger)
                    .ok_or(PartnerPlan::RANGE)?,
                result.disposal,
            )),
            Err(error) if error == PartnerPlan::RANGE => Err(error),
            Err(_) => {
                let codes = match source.source_k1_items(items) {
                    Ok(codes) => codes,
                    Err(error) if error == PartnerPlan::RANGE => return Err(error),
                    Err(_) => items.k1(),
                };
                Ok((
                    codes.checked_flows_whole().ok_or(PartnerPlan::RANGE)?,
                    crate::k1_items::Disposal::default(),
                ))
            }
        }
    }

    // The Schedule K-1 facts of the codes in the return's totals.
    pub(crate) fn totals_facts<'a>(
        &self,
        entry: &'a crate::FederalIncomeEntry,
    ) -> Result<crate::k1_totals::Facts<'a>, (&'static str, &'static str)> {
        let (flows, disposal) = self.outcome(&self.regular)?;
        let (amt_flows, amt_disposal) = match self.alternative.as_ref() {
            Some(record) => self.outcome(record)?,
            None => (flows.clone(), disposal),
        };
        let (nii_activity, investment_activity) = crate::k1_totals::classify(
            self.trade_kind,
            self.participation.as_ref().map_or(
                crate::participation::Participation::Unavailable,
                PartnerParticipation::material,
            ),
        );
        Ok(crate::k1_totals::Facts {
            entry,
            partner: true,
            flows,
            amt_flows,
            disposal,
            amt_disposal,
            nii_activity,
            investment_activity,
            term: self.interest_term,
            collectibles: self.sale.collectibles,
            unrecaptured: self.sale.unrecaptured,
            reit_dividends: self.qbi.as_ref().map_or(0, |v| v.reit_dividends),
            deferred_tax: self.deferred_compensation_tax,
            source: "https://www.irs.gov/instructions/i1065sk1",
            k3: self.schedule_k3.is_some(),
        })
    }

    // The section 751 gain (loss) of a sold interest, box 20 code AB, whole
    // cents: qualified business income with the statement's (unit B9-1).
    pub(crate) fn sale_qbi(&self) -> i64 {
        if matches!(self.disposition, Some(crate::K1Disposition::Sold { .. })) {
            self.sale.ordinary
        } else {
            0
        }
    }

    // No situation that leaves the whole K-1 out of the estimate (unit B9-1;
    // answers and unknowns, decision 2).
    pub(crate) fn complete(&self) -> bool {
        !self.special.iter().any(|situation| situation.whole())
            && self
                .source
                .as_deref()
                .is_none_or(|source| source.paper.contributed_built_in_property != Some(true))
    }

    // What one tax system's basis and at-risk limits allow and hold back of
    // this K-1, without the passive stage.
    pub(crate) fn ledger(
        &self,
        source: &PartnerActivityRecord,
    ) -> Result<LossLedger, (&'static str, &'static str)> {
        source.activity_result(&self.items_for_record(source)?).map(|v| v.ledger)
    }

    // The box 13 code A gift the outside basis allows, in cents: this year's
    // gift plus a released carry, less what the limit holds back. A K-1 whose
    // limits cannot be figured is refused elsewhere and counts its box.
    pub(crate) fn allowed_cash_charity(&self) -> i64 {
        let gift = self.separate.as_ref().map_or(0, |items| items.cash_charity);
        if gift == 0 && self.regular.carried.charity == 0 {
            return 0;
        }
        match self.ledger(&self.regular) {
            Ok(ledger) => ledger.basis_allowed[K1LossClass::Charity].get(),
            Err(_) => gift,
        }
    }

    pub(crate) const LONG_TERM_PARTS_LIMITED: &'static str = crate::fixture_english!(
        "Schedule K-1 (1065) box 9b collectibles or box 9c amounts with a box 9a long-term loss that your basis or amount at risk limits, or with a long-term loss carried from 2025, need the parts of the allowed loss, which the partner basis worksheet does not give."
    );

    // Whether a limit holds back part of the long-term loss, or the record
    // carries one into the year, while box 9b or a box 9c part is reported:
    // the instructions do not say how the allowed part divides among them.
    pub(crate) fn long_term_parts_affected(&self) -> bool {
        let Some(items) = &self.separate else {
            return false;
        };
        if items.collectibles == 0
            && items.unrecaptured_partnership == 0
            && items.unrecaptured_fund == 0
        {
            return false;
        }
        core::iter::once(&self.regular).chain(self.alternative.as_ref()).any(|record| {
            record.capital_carried.long != crate::K1LossCarry::default()
                || self.ledger(record).is_ok_and(|ledger| {
                    ledger.basis_suspended[K1LossClass::LongTerm].get() != 0
                        || ledger.risk_suspended[K1LossClass::LongTerm].get() != 0
                })
        })
    }

    pub(crate) fn capital_gains_by_term(&self) -> [i64; 2] {
        self.separate
            .as_ref()
            .map_or([0; 2], |items| [phase::floor(items.short, 0), phase::floor(items.long, 0)])
    }

    // K-1 box 14: all trade/business guarantees for general partners;
    // only services actually rendered for section 1402 limited partners.
    pub(crate) fn guaranteed_se(&self) -> i64 {
        self.guaranteed_services
            + if self.limited_for_se == Some(false) { self.guaranteed_capital } else { 0 }
    }

    pub(crate) fn health_payment_check(&self) -> Result<(), HealthPaymentError> {
        let premiums = match &self.health_insurance {
            crate::BusinessHealthInsurance::None => 0,
            crate::BusinessHealthInsurance::Plan { months, long_term_care, .. } => {
                months.iter().map(|m| m.premiums).sum::<i64>()
                    + long_term_care.iter().flat_map(|p| &p.months).map(|m| m.premiums).sum::<i64>()
            }
        };
        if self.health_guaranteed > premiums || self.health_guaranteed > self.guaranteed_services {
            return Err(HealthPaymentError::TaxablePremiums);
        }
        if self.health_guaranteed + self.health_distributions > premiums
            || self.health_distributions > self.regular.basis.distributions
            || self
                .alternative
                .as_ref()
                .is_some_and(|v| self.health_distributions > v.basis.distributions)
        {
            return Err(HealthPaymentError::DistributionPremiums);
        }
        Ok(())
    }

    pub(crate) fn qualified_se_gross_fits(&self, gross: i64, qualified: i64) -> bool {
        gross >= self.guaranteed_se()
            && (0..=gross - self.guaranteed_se()).contains(&qualified)
            && (self.limited_for_se == Some(false) || qualified == 0)
    }

    pub(crate) fn rental_participation_needed(&self) -> bool {
        self.regular.rentals.participation_needed(&self.credits, &self.investment_property)
    }

    pub(crate) fn limited_partner(&self) -> bool {
        matches!(self.participation, Some(PartnerParticipation::Limited { .. }))
    }

    // Box 2/3 activity classification follows the normal rental owner after
    // basis and at-risk limits. Section 469(k) changes the receiving PAL pool.
    pub(crate) fn rental_activities(
        &self,
        source: &PartnerActivityRecord,
        ledger: &LossLedger,
        professional_status: bool,
        group_disposed: bool,
    ) -> Result<alloc::vec::Vec<crate::k1_rental::RentalActivity>, (&'static str, &'static str)>
    {
        if self.publicly_traded && source.rentals.any() {
            if source.rentals.properties().iter().any(|property| property.has_items()) {
                return Err(crate::k1_rental::PTP_PROPERTY);
            }
            if self
                .rental_facts
                .participation
                .is_some_and(crate::K1RentalParticipation::professional)
            {
                return Err(crate::k1_rental::PTP_PARTICIPATION);
            }
            if self.disposition != Some(crate::K1Disposition::Kept)
                || !self.rental_facts.disposed.is_empty()
            {
                return Err(crate::k1_rental::PTP_DISPOSITION);
            }
            if self.rental_facts.real_estate_qbi.is_some() || self.rental_facts.other_qbi.is_some()
            {
                return Err(crate::k1_rental::PTP_QBI);
            }
        }
        crate::k1_rental::activities(
            &source.rentals,
            &self.rental_facts,
            ledger,
            self.limited_partner(),
            professional_status,
            self.disposition.as_ref().is_some_and(crate::K1Disposition::ended),
            group_disposed,
        )
    }

    // Forward the activity owner's actual post-risk cents; prior PAL has
    // already passed basis/risk and stays distinct from current amounts.
    pub(crate) fn ptp_rental_components(
        &self,
        source: &PartnerActivityRecord,
        ledger: &LossLedger,
    ) -> Result<alloc::vec::Vec<crate::ptp_passive::PtpPalComponent>, (&'static str, &'static str)>
    {
        self.rental_activities(source, ledger, false, false)?
            .into_iter()
            .map(|activity| {
                let current = activity
                    .passive
                    .checked_add(activity.prior)
                    .ok_or(crate::k1_rental::PTP_SOURCE)?;
                Ok(crate::ptp_passive::PtpPalComponent {
                    character: if activity.real_estate {
                        K1LossClass::RentalRealEstate
                    } else {
                        K1LossClass::OtherRental
                    },
                    passive_current_cents: current,
                    prior_passive_loss_cents: activity.prior,
                    nonpassive_cents: activity.nonpassive,
                })
            })
            .collect()
    }

    pub(crate) fn ptp_passive_activity(
        &self,
        id: i64,
        source: &PartnerActivityRecord,
        pool: &PartnerPoolResult,
    ) -> Result<crate::ptp_passive::PtpPalActivity, (&'static str, &'static str)> {
        let items = self.items_for_record(source)?;
        let nonpassive =
            self.material_for_losses() == crate::participation::Participation::Yes || items.ended();
        let mut components = alloc::vec![crate::ptp_passive::PtpPalComponent {
            character: K1LossClass::Ordinary,
            passive_current_cents: if nonpassive { 0 } else { pool.current_ordinary_with_items },
            prior_passive_loss_cents: pool.combined.prior_pool,
            nonpassive_cents: pool.combined.nonpassive,
        }];
        components.extend(self.ptp_rental_components(source, &pool.activity.ledger)?);
        Ok(crate::ptp_passive::PtpPalActivity { k1: id, source: id, components })
    }

    pub(crate) fn material_for_losses(&self) -> crate::participation::Participation {
        match self.trade_kind {
            crate::ScorpTradeKind::Ordinary => self.participation.as_ref().map_or(
                crate::participation::Participation::Unavailable,
                PartnerParticipation::material,
            ),
            crate::ScorpTradeKind::FinancialTrading => crate::participation::Participation::Yes,
        }
    }

    pub(crate) fn young_earned(
        &self,
        net: i64,
        half_se: i64,
    ) -> Result<(i64, bool), (&'static str, &'static str)> {
        if self.regular.basis.prior_loss != 0
            || self.regular.risk.prior_loss != 0
            || self.regular.passive_loss != 0
        {
            return Err((
                "ISSUE_YOUNG_PARTNER_HISTORICAL_LOSSES",
                crate::fixture_english!(
                    "Young-partner historical losses require separate earned-income coordination."
                ),
            ));
        }
        let ordinary = net - dollars(self.guaranteed_services) - dollars(self.guaranteed_capital);
        let service_payment = dollars(self.guaranteed_services);
        // A source that is not known (absent, Not sure, a services-only
        // gross not entered, or the 30% answer without net profit) is taken
        // as Capital: only the guaranteed service payment is earned income.
        // The flag asks the caller for its warning.
        let assumed = Ok((service_payment, true));
        match self.income_source.as_ref() {
            Some(crate::BusinessIncomeSource::Services) => match self.gross_personal_services {
                Some(gross) => Ok((dollars(gross) + service_payment, false)),
                None => assumed,
            },
            Some(crate::BusinessIncomeSource::Capital) => Ok((service_payment, false)),
            Some(crate::BusinessIncomeSource::MixedThirtyPercent) if ordinary <= 0 => assumed,
            Some(
                source @ (crate::BusinessIncomeSource::Mixed { .. }
                | crate::BusinessIncomeSource::MixedThirtyPercent),
            ) => {
                let guarantee_se = self.guaranteed_se();
                let ordinary_half = if guarantee_se == 0 {
                    half_se
                } else if half_se == 0 {
                    0
                } else {
                    let gross = self.se_gross.as_ref()
                        .ok_or(("ISSUE_YOUNG_PARTNER_GROSS_SE_INCOME", crate::fixture_english!("Supply K-1 gross SE income to allocate the mixed partner's half-SE deduction between profit and guaranteed payments.")))?
                        .source().1;
                    if gross < guarantee_se || gross == 0 {
                        return Err((
                            "ISSUE_YOUNG_PARTNER_GROSS_SE_RECONCILIATION",
                            crate::fixture_english!(
                                "Reconcile K-1 gross SE income with guaranteed payments before allocating the young partner's half-SE deduction."
                            ),
                        ));
                    }
                    crate::kernel::ratio::allocate_remaining(
                        half_se,
                        &[
                            gross - guarantee_se,
                            self.guaranteed_services,
                            guarantee_se - self.guaranteed_services,
                        ],
                    )[0]
                };
                Ok((source.earned(ordinary, ordinary_half).unwrap_or(0) + service_payment, false))
            }
            None => assumed,
        }
    }

    pub(crate) fn valid(&self) -> bool {
        self.validation_issue("").is_none()
    }
    pub(crate) fn validation_issue(
        &self,
        key: &'static str,
    ) -> Option<crate::validation::ValidationIssue> {
        self.validation_issue_with_mode(key, true)
    }
    pub(crate) fn validation_issue_with_mode(
        &self,
        key: &'static str,
        source_alignment: bool,
    ) -> Option<crate::validation::ValidationIssue> {
        let mut issue = None;
        if self.valid_details(key, &mut issue, source_alignment) {
            None
        } else {
            Some(issue.unwrap_or_else(|| crate::validation::ValidationIssue::new(key)))
        }
    }
    // Arithmetic safety: plan, se_after_passive_with_limits and k1_totals
    // retain native income/loss/basis arithmetic and dollar-to-cent transfers.
    fn valid_details(
        &self,
        key: &'static str,
        issue: &mut Option<crate::validation::ValidationIssue>,
        source_alignment: bool,
    ) -> bool {
        // Saving a structurally valid record does not require every tax fact
        // to be established. Unknown facts remain visible calculation issues.
        let valid = |v: &PartnerActivityRecord| {
            v.basis.valid_amounts(v.ordinary, v.exempt, v.nondeductible)
                && (0..=100000000000).contains(&v.passive_loss)
                && v.rentals.valid()
                && v.carried.valid()
                && v.capital_carried.valid()
                && v.section1231.as_ref().is_none_or(|v| v.valid())
        };
        valid(&self.regular)
            && crate::K1Section179Sale::valid_source_list(&self.section179_sales)
            && (!source_alignment
                || (crate::K1Section179Sale::carryover_rule(
                    &self.section179_sales,
                    self.section179_carryover,
                    self.section179_carryover_sold,
                )
                .is_none()
                    && self.alternative.as_ref().is_none_or(|_| {
                        crate::K1Section179Sale::views(&self.section179_sales, true).map_or(
                            true,
                            |sales| {
                                crate::K1Section179Sale::carryover_rule(
                                    &sales,
                                    self.section179_carryover,
                                    self.section179_carryover_sold,
                                )
                                .is_none()
                            },
                        )
                    })
                    && !(self.limited_partner()
                        && self.rental_facts.participation.is_some_and(|value| value.active()))
                    && crate::K1Credit::issue_list_with_mode(&self.credits, true).is_none()
                    && (self
                        .investment_property
                        .iter()
                        .all(|property| property.own_form().is_some())
                        || self.investment_confirmed.is_some())
                    && (!self.credits.iter().any(|credit| credit.code.housing())
                        || self.housing_confirmed.is_some())))
            && self.rental_facts.valid()
            && self.qbi_records.valid()
            && crate::K1Credit::source_valid_list(&self.credits)
            && self
                .research_credit_line
                .is_none_or(|line| crate::K1CreditCode::Research.line_allows(line))
            && crate::K1InvestmentProperty::valid_list(&self.investment_property)
            && self.income_source.as_ref().is_none_or(|v| v.valid())
            && self.gross_personal_services.is_none_or(|v| {
                (0..=100000000000).contains(&v)
                    && (!source_alignment
                        || self.regular.ordinary <= 0
                        || v >= self.regular.ordinary)
            })
            && {
                let valid = (0..=100000000000).contains(&self.guaranteed_services);
                if !valid {
                    *issue =
                        Some(crate::validation::ValidationIssue::new(key).at_field("services"));
                }
                valid
            }
            && [self.guaranteed_capital, self.health_guaranteed, self.health_distributions]
                .into_iter()
                .all(|v| (0..=100000000000).contains(&v))
            && self.health_insurance.valid()
            && (0..=100000000000).contains(&self.pensions)
            && self.retirement.valid()
            && self.fuel_tax_credit.as_ref().is_none_or(|v| v.valid())
            && self.prior_fuel_credit.as_ref().is_none_or(|v| {
                *issue = v.validation_issue(key).map(|i| i.in_field("prior_fuel_credit"));
                issue.is_none()
            })
            && self.qbi.as_ref().is_none_or(|v| {
                *issue = v.validation_issue(self, key).map(|i| i.in_field("qbi"));
                issue.is_none()
            })
            && self.se_gross.as_ref().is_none_or(|v| (0..=100000000000).contains(&v.source().1))
            && self.alternative.as_ref().is_none_or(valid)
            && self.reported_se.is_none_or(|v| (-100000000000..=100000000000).contains(&v))
            && self.separate.as_ref().is_none_or(PartnerSeparateTotals::valid)
            && self.schedule_k3.as_ref().is_none_or(|k3| k3.valid_with_mode(source_alignment))
            && (0..=100000000000).contains(&self.section179)
            && (0..=100000000000).contains(&self.section179_carryover)
            && (0..=100000000000).contains(&self.section179_carryover_sold)
            && self
                .amt_section179_income_carry
                .as_ref()
                .is_none_or(|history| history.rule().is_none())
            && crate::PartnerIncomeTotal::rule(&self.income).is_none()
            && crate::PartnerDeduction::valid_list(&self.deductions)
            && crate::PartnerSpecial::valid_list(&self.special)
            && self.disposition.as_ref().is_none_or(crate::K1Disposition::valid)
            && self.sale.valid()
            && (0..=100000000000).contains(&self.deferred_compensation_tax)
            && (0..=100000000000).contains(&self.crp_payments)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PartnerSeResult {
    pub earnings: i64,
    // Actual positive box14/guaranteed source cents after the same admitted
    // annual loss/section179 deductions. Those filed deductions stay fixed;
    // this does not reconstruct cents discarded by their owning ledgers.
    pub earnings_cents: i64,
    // The portion still classified as passive after former-activity own-income
    // releases. Exclude it once from EIC investment income, not all SE earnings.
    pub passive_earnings: i64,
    pub basis_suspended: i64,
    pub risk_suspended: i64,
    pub passive_suspended: i64,
}

impl PartnerK1 {
    // Ordinary and current guaranteed-payment source adapter. The caller supplies this activity's final
    // regular passive disallowance from the shared pool, never an AMT amount.
    // K-1 box 14 instructions, Pub. 925, and 1.469-1T(d)(3): an unallowed
    // loss does not reduce SE earnings. Character survives the limitation.
    // `section179` is this K-1's box 12 amount allowed on Form 4562 line 12
    // (actual cents from the existing section179 allocation): a general
    // partner reduces box 14 code A by the section 179 deduction claimed.
    pub(crate) fn se_after_passive(
        &self,
        suspended: Amount<Cents>,
        section179: Amount<Cents>,
    ) -> Result<PartnerSeResult, (&'static str, &'static str)> {
        let pool = self.pool(&self.regular)?;
        self.se_after_passive_with_limits(self, pool, suspended, section179)
    }

    // The SE source remains whole, while an allocated income ledger supplies
    // the proportions admitted by basis, risk and passive limits. No second
    // undivided limitation ledger is run for a community separate return.
    pub(crate) fn se_after_passive_with_limits(
        &self,
        limited: &Self,
        pool: PartnerPoolResult,
        suspended: Amount<Cents>,
        section179: Amount<Cents>,
    ) -> Result<PartnerSeResult, (&'static str, &'static str)> {
        self.earned_after_passive_in(limited, pool, suspended, section179, false)
    }

    // Form 6251 Related Adjustments refigures earned-income limits, while
    // Schedule SE and its actual half-tax deduction remain regular-tax amounts.
    // Reuse the source-character allocator with the AMT activity's final PAL.
    pub(crate) fn amt_earned_after_passive(
        &self,
        suspended: Amount<Cents>,
        section179: Amount<Cents>,
    ) -> Result<PartnerSeResult, (&'static str, &'static str)> {
        let source = self.alternative.as_ref().ok_or(Self::PERSONAL_NEEDED)?;
        self.earned_after_passive_in(self, self.pool(source)?, suspended, section179, true)
    }

    fn earned_after_passive_in(
        &self,
        limited: &Self,
        pool: PartnerPoolResult,
        suspended: Amount<Cents>,
        section179: Amount<Cents>,
        alternative: bool,
    ) -> Result<PartnerSeResult, (&'static str, &'static str)> {
        let limited_for_se = self.limited_for_se.ok_or(Self::PERSONAL_NEEDED)?;
        let source = if alternative {
            self.alternative.as_ref().ok_or(Self::PERSONAL_NEEDED)?
        } else {
            &self.regular
        };
        let v = if alternative {
            limited.alternative.as_ref().ok_or(Self::PERSONAL_NEEDED)?
        } else {
            &limited.regular
        };
        let suspended = suspended.get();
        let section179 = section179.get();
        let mut reported = self.reported_se.ok_or((
            "ISSUE_PARTNERSHIP_ESTABLISH_K_1_BOX_14_CODE_A_2F03EA0B",
            crate::fixture_english!("Establish K-1 box 14 code A."),
        ))? - self.guaranteed_se();
        if alternative {
            let delta =
                source.ordinary.checked_sub(self.regular.ordinary).ok_or(PartnerPlan::RANGE)?;
            // An excluded distributive share stays excluded. A mixed box 1 /14A
            // source does not identify which character an AMT adjustment changes.
            if !limited_for_se && self.trade_kind == crate::ScorpTradeKind::Ordinary {
                if delta != 0 && reported != self.regular.ordinary {
                    return Err(Self::PERSONAL_NEEDED);
                }
                reported = reported.checked_add(delta).ok_or(PartnerPlan::RANGE)?;
            }
            // Separate AMT loss balances do not supply their SE-character
            // subsets. The same-record choice preserves those source facts;
            // otherwise only explicit zero retained balances establish them.
            if self.amt_record_choice != Some(false)
                && [
                    source.basis.prior_loss,
                    source.risk.prior_loss,
                    source.passive_loss,
                    self.regular.basis.prior_loss,
                    self.regular.risk.prior_loss,
                    self.regular.passive_loss,
                ]
                .into_iter()
                .any(|value| value != 0)
            {
                return Err(Self::PERSONAL_NEEDED);
            }
        }
        if !self.valid()
            || (self.limited_for_se == Some(true) && reported != 0)
            || (self.trade_kind == crate::ScorpTradeKind::FinancialTrading && reported != 0)
        {
            return Err((
                "ISSUE_PARTNERSHIP_RECONCILE_BOX_14_LESS_GUARANTEED_PAYMENTS_F10E8824",
                crate::fixture_english!(
                    "Reconcile box 14 code A after guaranteed payments: a limited partner includes only service payments, trading earnings need separate treatment, and an ordinary loss held back by basis, at-risk or passive limits needs a compatible self-employment share."
                ),
            ));
        }
        let totals = [source.basis.prior_loss, source.risk.prior_loss, source.passive_loss];
        let h = &self.se_history;
        let history = [h.basis, h.risk, h.passive];
        if history.iter().zip(totals).any(|(part, total)| *part < 0 || *part > total) {
            return Err((
                "ISSUE_PARTNERSHIP_SELF_EMPLOYMENT_LOSS_PORTIONS_CANNOT_EXCEED_THEIR_5D13E5AC",
                crate::fixture_english!(
                    "Self-employment loss portions cannot exceed their separate retained total balances."
                ),
            ));
        }

        if suspended < 0 || suspended > phase::floor(-pool.passive, 0) {
            return Err((
                "ISSUE_PARTNERSHIP_THE_REGULAR_TAX_PASSIVE_ALLOCATION_DOES_NOT_4C176626",
                crate::fixture_english!(
                    "The regular-tax passive allocation does not reconcile with this partnership activity."
                ),
            ));
        }
        let activity = pool.activity;
        // Box 14 is independently computed under section 1402(a)(3): a
        // property item inside box 1 may be excluded from SE earnings.
        // Only a binding ordinary loss limit needs an allocation within
        // box 1. Never assign an out-of-range SE loss to that limited pool.
        let ordinary_held = activity.ledger.basis_suspended[K1LossClass::Ordinary].get() > 0
            || activity.ledger.risk_suspended[K1LossClass::Ordinary].get() > 0
            || suspended > 0;
        if ordinary_held
            && (reported < phase::cap(v.ordinary, [0]) || (v.ordinary < 0 && reported > 0))
        {
            return Err((
                "ISSUE_PARTNERSHIP_RECONCILE_BOX_14_LESS_GUARANTEED_PAYMENTS_F10E8824",
                crate::fixture_english!(
                    "Reconcile box 14 code A after guaranteed payments: a limited partner includes only service payments, trading earnings need separate treatment, and an ordinary loss held back by basis, at-risk or passive limits needs a compatible self-employment share."
                ),
            ));
        }
        let share = |allowed: i64, total: i64, se: i64| {
            if core::ptr::eq(self, limited) {
                crate::kernel::ratio::allocate_remaining(allowed, &[se, total - se])[0]
            } else if total == 0 {
                0
            } else {
                crate::kernel::ratio::fraction(se, allowed, total)
            }
        };
        let current_se_loss = phase::floor(-reported, 0);
        let current_ordinary_loss = phase::floor(-source.ordinary, 0);
        // With no ordinary suspension the unmatched part is deductible in
        // full; it is not a negative residual capacity in the allocator.
        let unmatched_loss = phase::floor(current_se_loss - current_ordinary_loss, 0);
        let basis_se = phase::cap(current_se_loss, [current_ordinary_loss]) + history[0];
        let basis_total = phase::floor(-v.ordinary, 0) + v.basis.prior_loss;
        let after_basis =
            share(activity.basis.allowed[K1LossClass::Ordinary].get(), basis_total, basis_se);
        let risk_se = after_basis + history[1];
        let risk_total = activity.basis.allowed[K1LossClass::Ordinary].get() + v.risk.prior_loss;
        let ordinary_allowed = activity.ledger.risk_allowed[K1LossClass::Ordinary].get();
        let after_risk = share(ordinary_allowed, risk_total, risk_se);
        let (allowed_se, passive_suspended, passive_earnings) =
            if self.material_for_losses() == crate::participation::Participation::Yes {
                let prior_allowed = v.passive_loss - suspended;
                let prior_se = share(prior_allowed, v.passive_loss, history[2]);
                // Preserve rounding of the total released SE portion. The own-income
                // release is nonpassive; only additional release against other
                // passive income belongs in the passive investment-income adjustment.
                let own_se = share(pool.prior_released, v.passive_loss, history[2]);
                (after_risk + prior_se, history[2] - prior_se, own_se - prior_se)
            } else {
                let total = ordinary_allowed + v.passive_loss;
                let se = after_risk + history[2];
                let allowed = share(total - suspended, total, se);
                (allowed, se - allowed, phase::floor(reported, 0) - allowed - unmatched_loss)
            };
        // The Form 4562 line 12 amount comes from the box 12 share the basis
        // allowed (a released carry included) and the 2025 line 13 carryover.
        if section179 < 0
            || section179
                > activity.ledger.basis_allowed[K1LossClass::Section179].get()
                    + self.section179_carryover
        {
            return Err((
                "ISSUE_PARTNERSHIP_THE_SECTION_179_DEDUCTION_ALLOWED_FOR_251BF7AD",
                crate::fixture_english!(
                    "The section 179 deduction allowed for this partnership does not reconcile with its K-1 box 12 amount."
                ),
            ));
        }
        Ok(PartnerSeResult::from_resolved(ResolvedPartnerSe {
            reported,
            guaranteed_services: self.guaranteed_services,
            guaranteed_capital: self.guaranteed_capital,
            limited_for_se,
            section179,
            allowed_se,
            unmatched_loss,
            passive_earnings,
            basis_suspended: basis_se - after_basis,
            risk_suspended: risk_se - after_risk,
            passive_suspended,
        }))
    }
}

// Resolved source cents and the actual annual deductions admitted by the
// activity owners. Both source models use this one final SE projection.
pub(crate) struct ResolvedPartnerSe {
    pub reported: i64,
    pub guaranteed_services: i64,
    pub guaranteed_capital: i64,
    pub limited_for_se: bool,
    pub section179: i64,
    pub allowed_se: i64,
    pub unmatched_loss: i64,
    pub passive_earnings: i64,
    pub basis_suspended: i64,
    pub risk_suspended: i64,
    pub passive_suspended: i64,
}
impl PartnerSeResult {
    pub(crate) fn from_resolved(source: ResolvedPartnerSe) -> Self {
        let earnings_cents = phase::floor(source.reported, 0)
            + source.guaranteed_services
            + if !source.limited_for_se {
                source.guaranteed_capital - source.section179
            } else {
                0
            }
            - source.allowed_se
            - source.unmatched_loss;
        Self {
            earnings: dollars(earnings_cents),
            earnings_cents,
            passive_earnings: dollars(source.passive_earnings),
            basis_suspended: dollars(source.basis_suspended),
            risk_suspended: dollars(source.risk_suspended),
            passive_suspended: dollars(source.passive_suspended),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct PartnerOwnerSe {
    pub profit: i64,
    pub net: i64,
    pub tax: i64,
    pub half: i64,
    pub attribution: alloc::collections::BTreeMap<i64, i64>,
    // WorksheetW copies the filed Schedule1line15 total, then attributes it
    // using source profit ratios without whole-dollar interim rounding.
    pub attribution_cents: alloc::collections::BTreeMap<i64, i64>,
}

// Actual-method projection after regular activity limits. Apply the shared
// wage base once per owner, not once per K-1. Optional-method selection and
// downstream deduction reconciliation belong to the return coordinator.
#[derive(Debug)]
pub(crate) enum OwnerSeIssue {
    Rule((&'static str, &'static str)),
    Prepared(crate::Section179PreparedIssue),
}

impl From<(&'static str, &'static str)> for OwnerSeIssue {
    fn from(issue: (&'static str, &'static str)) -> Self {
        Self::Rule(issue)
    }
}

impl From<crate::Section179PreparedIssue> for OwnerSeIssue {
    fn from(issue: crate::Section179PreparedIssue) -> Self {
        Self::Prepared(issue)
    }
}

#[cfg(test)]
pub(crate) fn owner_actual_se(
    d: &crate::FederalDraft,
    owner: crate::Owner,
    social_wages_cents: i64,
    church_wages_cents: i64,
    partners: &[(i64, PartnerSeResult)],
) -> Result<PartnerOwnerSe, (&'static str, &'static str)> {
    let source_claims = crate::related_forms::effective_claimants(d);
    owner_actual_se_with_claims(
        d,
        owner,
        social_wages_cents,
        church_wages_cents,
        partners,
        &source_claims,
    )
}

#[cfg(test)]
pub(crate) fn owner_actual_se_with_claims(
    d: &crate::FederalDraft,
    owner: crate::Owner,
    social_wages_cents: i64,
    church_wages_cents: i64,
    partners: &[(i64, PartnerSeResult)],
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<PartnerOwnerSe, (&'static str, &'static str)> {
    match owner_actual_se_with_section179_with_claims(
        crate::related_forms::RelatedFormContext::original(d, source_claims),
        owner,
        social_wages_cents,
        church_wages_cents,
        partners,
        None,
        None,
    ) {
        Ok(value) => Ok(value),
        Err(OwnerSeIssue::Rule(issue)) => Err(issue),
        Err(OwnerSeIssue::Prepared(_)) => {
            unreachable!("legacy owner SE calculation has no selected allocation")
        }
    }
}

pub(crate) fn owner_actual_se_with_section179_with_claims(
    form_context: crate::related_forms::RelatedFormContext<'_>,
    owner: crate::Owner,
    social_wages_cents: i64,
    church_wages_cents: i64,
    partners: &[(i64, PartnerSeResult)],
    selected: Option<&crate::Section179PreparedAllocation>,
    home: Option<&crate::home_office::HomeContext>,
) -> Result<PartnerOwnerSe, OwnerSeIssue> {
    let crate::related_forms::RelatedFormContext { draft: d, claims: source_claims, .. } =
        form_context;
    let mut business_cents = 0;
    let mut partner_profit = 0;
    let mut weights = alloc::vec::Vec::new();
    let mut weights_cents = alloc::vec::Vec::new();
    for entry in d.income.iter().filter(|e| e.owner == crate::DocumentOwner::from(owner)) {
        match &entry.income {
            crate::FederalIncome::Business { data }
                if !crate::statutory_wages::business(d, entry.id) =>
            {
                // A farm's profit includes the Form 1099-G box 7 and 1099-PATR
                // income routed to it (Schedule F line 9).
                let profit = data.earned_receipts()
                    + crate::federal_rules::farm_routed_income_with_claims(
                        d,
                        usize::from(owner == crate::Owner::Spouse),
                        entry.id,
                        source_claims,
                    )?
                    - crate::section179_return::expenses_with_claims(
                        d,
                        entry.id,
                        data,
                        selected,
                        home,
                        source_claims,
                    )
                    .map(|expense| {
                        expense
                            - selected.map_or(0, |allocation| {
                                allocation.ordinary_limit_adjustment(entry.id)
                            })
                    })?
                    - crate::clergy::business_se_exclusion_with_section179_with_claims(
                        d,
                        entry.id,
                        data,
                        selected,
                        home,
                        source_claims,
                    )?;
                business_cents += profit;
                weights.push((entry.id, dollars(profit)));
                weights_cents.push((entry.id, profit));
            }
            crate::FederalIncome::PartnershipActivities { data }
                if matches!(data.ordinary_self_employment(), Ok(None)) => {}
            crate::FederalIncome::Partnership { .. }
            | crate::FederalIncome::PartnershipActivities { .. } => {
                let mut found = partners.iter().filter(|(id, _)| *id == entry.id);
                let source = found.next().ok_or(("ISSUE_PARTNERSHIP_RESOLVE_EVERY_PARTNERSHIP_S_REGULAR_SELF_EMPLOYMENT_SOURCE_6800F63B", crate::fixture_english!("Resolve every partnership's regular self-employment source before combining this owner's earnings.")))?;
                if found.next().is_some() {
                    return Err((
                        "ISSUE_PARTNERSHIP_A_PARTNERSHIP_SELF_EMPLOYMENT_SOURCE_WAS_SUPPLIED_4C3D6EAC",
                        crate::fixture_english!(
                            "A partnership self-employment source was supplied more than once."
                        ),
                    ).into());
                }
                partner_profit += if crate::clergy::calculate_with_section179_with_claims(
                    d,
                    owner,
                    selected,
                    home,
                    source_claims,
                )?
                .sect
                {
                    0
                } else {
                    source.1.earnings
                };
                weights.push((entry.id, source.1.earnings));
                weights_cents.push((entry.id, source.1.earnings_cents));
            }
            _ => {}
        }
    }
    let profit = dollars(
        business_cents
            + crate::clergy::calculate_with_section179_with_claims(
                d,
                owner,
                selected,
                home,
                source_claims,
            )?
            .extra_se(),
    ) + partner_profit;
    // Church employee income (Schedule SE lines 5a-5b) joins line 6 beside the
    // combined line 4c (section 1402(j)(2)).
    let (net, tax, half) =
        crate::federal_rules::self_employment(profit * 100, social_wages_cents, church_wages_cents);
    Ok(PartnerOwnerSe {
        profit,
        net,
        tax,
        half,
        attribution: crate::business_rules::half_se_allocation(half, weights),
        attribution_cents: crate::business_rules::half_se_allocation(half * 100, weights_cents),
    })
}

// Community-property allocation of this source record; facts retain their meaning.
crate::community::divide_record!(PartnerK1 {
    amounts: [
        regular,
        gross_personal_services,
        guaranteed_services,
        guaranteed_capital,
        alternative,
        qbi,
        qbi_records,
        prior_fuel_credit,
        separate,
        schedule_k3,
        rental_facts,
        section179,
        section179_carryover,
        section179_carryover_sold,
        amt_section179_income_carry,
        section179_sales,
        credits,
        investment_property,
        income,
        deductions,
        deferred_compensation_tax,
        crp_payments
    ],
    gains: [sale, disposition],
    facts: [
        source,
        research_credit_line,
        amt_record_choice,
        reporting_period,
        qbi_aggregation_group,
        fuel_tax_credit,
        income_source,
        participation,
        trade_kind,
        limited_for_se,
        reported_se,
        se_history,
        se_gross,
        health_insurance,
        health_guaranteed,
        health_distributions,
        pensions,
        retirement,
        investment_confirmed,
        housing_confirmed,
        unentered,
        final_k1,
        interest_term,
        special,
        publicly_traded
    ],
});

// Community-property allocation of this source record; facts retain their meaning.
crate::community::divide_record!(PartnerBasisRecord {
    amounts: [
        opening,
        contributions,
        opening_liabilities,
        ending_liabilities,
        assumed,
        transferred,
        distributions,
        prior_loss,
        prior_nondeductible
    ],
    gains: [],
    facts: [complete],
});

// Community-property allocation of this source record; facts retain their meaning.
crate::community::divide_record!(PartnerRiskRecord {
    amounts: [
        start,
        opening_debt,
        ending_debt,
        excluded_contributions,
        excluded_distributions,
        protected,
        released,
        prior_loss,
        prior_nondeductible,
        deducted,
        recaptured
    ],
    gains: [],
    facts: [complete],
});

// Community-property allocation of this source record; facts retain their meaning.
crate::community::divide_record!(PartnerActivityRecord {
    amounts: [
        ordinary,
        exempt,
        nondeductible,
        basis,
        risk,
        passive_loss,
        rentals,
        carried,
        capital_carried,
        section1231
    ],
    gains: [],
    facts: [],
});

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(PartnerSeparateItems {
    amounts: [
        interest,
        dividends,
        qualified_dividends,
        royalties,
        short,
        long,
        collectibles,
        unrecaptured_partnership,
        unrecaptured_fund,
        cash_charity,
        backup_withholding,
        exempt_interest,
        foreign_taxes
    ],
    gains: [],
    facts: [],
});

crate::community::divide_record!(PartnerSeparateTotals {
    amounts: [
        interest,
        dividends,
        qualified_dividends,
        royalties,
        short,
        long,
        collectibles,
        unrecaptured_partnership,
        unrecaptured_fund,
        cash_charity,
        backup_withholding,
        exempt_interest,
        foreign_taxes
    ],
    gains: [],
    facts: [],
});

// Allocate source amounts; retained character, identity and confirmations stay facts.
impl crate::community::Divide for PartnerPriorFuelCredit {
    fn divided(&self, share: crate::community::Share) -> Self {
        if share.whole() {
            return self.clone();
        }
        // Calculation-only record: determine the source's taxable recovery
        // before projecting it. These derived amounts/confirmations never edit
        // the accepted source or split its retained comparison worksheet facts.
        Self {
            claimed_credit: share.filer(self.taxable_amount()),
            full_prior_tax_benefit: self.full_prior_tax_benefit || self.partial_benefit_applies(),
            partial_prior_year_benefit: None,
            ..self.clone()
        }
    }
}
