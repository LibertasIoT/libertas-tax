// Coverage: Schedule 8812 / Form 8812 — Supported (audited 2026-09-24).
//   Lines: CTC $2,200 per SSN child under 17, ODC $500, $50/$1,000 MAGI phaseout over $200k/$400k
//   with 2555 add-back; nonrefundable limit ordered per Credit Limit Worksheet B; ACTC =
//   min($1,700×children, 15%×(earned−$2,500)); three-child payroll alternative; zero ACTC with Form
//   2555. No named exclusions (Puerto Rico Part II-B not modeled).
//   Evidence: forms/schedule_8812.rs (compute, Schedule8812::limit, lines 14 and 27),
//   federal_rules.rs (family; the Worksheet B ordering of the homeowner credits), income_tax.rs
//   (PersonalCreditPotentials::limit); tests: child_and_other_dependent_credit,
//   head_of_household_eitc_and_additional_child_credit, three_child_payroll_refund_alternative,
//   exclusion_magi_is_restored_for_niit_and_child_phaseout (family.rs); i1040s8
//   C-fixture (2026-09-25): mfj-three-children-magi-phaseout (returns.rs; lines 1-14, 16a-27 with the
//   three-child payroll alternative, MAGI inside the joint phase-out, by hand) with family.rs
//   child_credit_phaseout_starts_at_the_first_dollar_over_the_threshold,
//   itin_child_earns_the_other_dependent_credit,
//   foreign_exclusion_election_zeroes_the_additional_child_credit,
//   additional_child_credit_needs_earned_income_over_2500. The loader now carries a W-2 owner.
//   Unit family credits F8 (2026-10-10): every printed line except 2a, 2c (Puerto Rico and
//   American Samoa income, not modeled) and the reserved line 15 is a return row (lines 1–14,
//   16a–27; Part II-B lines 21–26 only when the payroll alternative applies; a zero amount is
//   dropped by the review), a child attains 17 on the birthday (Rev. Rul. 2003-72), a child
//   born and died in 2026 without an SSN counts on line 4 (FederalDependent::born_and_died), a
//   U.S.-citizen child who lived abroad is not excluded, and line 18a deducts the joint
//   special-needs deemed excess (the negative Form 1040 line 1f). Tests:
//   line_audit_family_credits.rs d1_*, d2_*, d3_*, d6_*, d12_*.
//   Forms 8332 and 2120 (unit form-8332, 2026-10-09): family() counts a child released to the filer
//   (IRC 152(e)(1)) under 17 with an SSN in the CTC and an older or SSN-less one in the ODC, never
//   the custodial parent who released the child; a qualifying relative under a multiple support
//   agreement (IRC 152(d)(3)) is ODC only. Tests: noncustodial_parent_with_a_release_gets_the_child_tax_credit_only,
//   custodial_parent_who_released_keeps_head_of_household_eic_and_care,
//   multiple_support_agreement_gives_the_credit_for_other_dependents_not_head_of_household,
//   released_child_aged_17_gets_the_credit_for_other_dependents (family.rs).

//   Wage-special coordination (2026-10-12): combat pay is always ACTC earned income;
//   excluded Medicaid follows the EIC choice when EIC is claimed and the Earned Income
//   Worksheet otherwise. Payroll includes RRTA tier 1/ordinary Medicare and actual
//   additional tax, excluding the separate actual RRTA withholding. The shared
//   homeowner-credit owner also scores election candidates without duplicating limits.
//   Evidence: src/tests/income/wage_specials.rs B, D, E and three-child RRTA payroll fixture.

// Schedule 8812 credits for qualifying children and other dependents.
// Amounts are whole dollars.
use crate::kernel::phase;
use crate::line_emission::{form_count, form_line};
use crate::{
    FederalResult, FormLine, TaxForm,
    kernel::ratio::{fraction, positive},
};

const SOURCE: &str = "https://www.irs.gov/instructions/i1040s8";
const INPUTS: &[&str] = &["agi", "income_tax", "credits", "income", "payments"];

// What the return supplies to Schedule 8812 before the credit-limit ordering.
pub(crate) struct Schedule8812Context {
    // Line 4: qualifying children under 17 with the required SSN.
    pub(crate) children: i64,
    // Line 6: other dependents.
    pub(crate) other_dependents: i64,
    // Line 1: Form 1040 line 11b.
    pub(crate) agi: i64,
    // Line 3: MAGI including the Form 2555 exclusion.
    pub(crate) magi: i64,
    pub(crate) joint: bool,
    // Form 2555 elected: Part II is skipped and the ACTC is zero.
    pub(crate) foreign_election: bool,
    // Line 18a: earned income including the optional-method adjustment.
    pub(crate) earned: i64,
    // Line 21: Social Security and Medicare tax withheld (W-2 boxes 4 and 6)
    // less the Medicare tax Form 8959 treats as Additional Medicare Tax, plus
    // the Additional Medicare Tax itself.
    pub(crate) withheld: i64,
    // Line 22 without Schedule 1 line 15: Schedule 2 lines 16c and 17c (the
    // Form 4137 and 8919 tax and the W-2 box 12 A, B, M and N tax).
    pub(crate) other_payroll: i64,
    // Schedule 1 line 15, the deduction for half of self-employment tax.
    pub(crate) half_se: i64,
    // Line 24: Form 1040 line 27a, and Schedule 3 line 11.
    pub(crate) eic: i64,
    pub(crate) excess_social: i64,
}

// Line 12 and the refundable ceiling from lines 16b–26, before Credit Limit
// Worksheet B places the homeowner credits between them. The printed lines
// computed from the context are kept for the return rows.
pub(crate) struct Schedule8812 {
    pub(crate) potential: i64,
    pub(crate) actc_limit: i64,
    agi: i64,
    magi: i64,
    children: i64,
    other_dependents: i64,
    // Line 9, and line 10 in units of $1,000.
    threshold: i64,
    excess_thousands: i64,
    foreign_election: bool,
    earned: i64,
    // Line 20.
    earned_refund: i64,
    // Lines 21 and 22, and the amounts line 24 subtracts.
    withheld: i64,
    other_payroll: i64,
    eic: i64,
    excess_social: i64,
}

// Lines 13, 14 and 27, with the Credit Limit Worksheet A amount.
pub(crate) struct ChildCredits {
    pub(crate) limit: i64,
    pub(crate) nonrefundable: i64,
    pub(crate) additional: i64,
}

pub(crate) fn compute(ctx: &Schedule8812Context) -> Schedule8812 {
    // Lines 5–12: Form 8880 follows care/education; Schedule 8812 also
    // subtracts the allowed saver credit.
    let threshold = if ctx.joint {
        crate::year_constants::CHILD_TAX_CREDIT_PHASEOUT_START_JOINT_DOLLARS
    } else {
        crate::year_constants::CHILD_TAX_CREDIT_PHASEOUT_START_OTHER_DOLLARS
    };
    let excess_thousands = phase::steps(ctx.magi - threshold, 1000, phase::StepRounding::Up);
    let potential = positive(
        ctx.children * crate::year_constants::CHILD_TAX_CREDIT_PER_CHILD_DOLLARS
            + ctx.other_dependents * crate::year_constants::OTHER_DEPENDENT_CREDIT_DOLLARS
            - excess_thousands
                * crate::year_constants::CHILD_TAX_CREDIT_REDUCTION_PER_THOUSAND_DOLLARS,
    );
    // Lines 18a–20.
    let earned_refund = fraction(
        positive(ctx.earned - crate::year_constants::CHILD_TAX_CREDIT_EARNED_INCOME_FLOOR_DOLLARS),
        15,
        100,
    );
    let mut schedule = Schedule8812 {
        potential,
        actc_limit: 0,
        agi: ctx.agi,
        magi: ctx.magi,
        children: ctx.children,
        other_dependents: ctx.other_dependents,
        threshold,
        excess_thousands,
        foreign_election: ctx.foreign_election,
        earned: ctx.earned,
        earned_refund,
        withheld: ctx.withheld,
        other_payroll: ctx.other_payroll + ctx.half_se,
        eic: ctx.eic,
        excess_social: ctx.excess_social,
    };
    // Lines 16b and 21–26: the three-or-more-children alternative uses the
    // actual Additional Medicare tax, replacing its withholding.
    schedule.actc_limit = if ctx.foreign_election {
        0
    } else {
        phase::cap(
            schedule.children_limit(),
            [if ctx.children >= 3 {
                phase::floor(earned_refund, schedule.payroll_net())
            } else {
                earned_refund
            }],
        )
    };
    schedule
}

// Shared Worksheet B coordination for the current return and each earned-income election.
// Inputs are the credits ordered before the child credit, in their existing order.
pub(crate) struct HomeownerCredits {
    pub mortgage: i64,
    pub adoption: i64,
    pub dc: i64,
    pub energy: i64,
}
impl HomeownerCredits {
    pub(crate) fn total(&self) -> i64 {
        self.mortgage + self.adoption + self.dc + self.energy
    }
}

impl Schedule8812 {
    // Line 8.
    fn credit_before_phaseout(&self) -> i64 {
        self.children * crate::year_constants::CHILD_TAX_CREDIT_PER_CHILD_DOLLARS
            + self.other_dependents * crate::year_constants::OTHER_DEPENDENT_CREDIT_DOLLARS
    }

    // Line 16b.
    fn children_limit(&self) -> i64 {
        self.children * crate::year_constants::ADDITIONAL_CHILD_TAX_CREDIT_PER_CHILD_DOLLARS
    }

    // Line 23.
    fn payroll_total(&self) -> i64 {
        self.withheld + self.other_payroll
    }

    // Line 25.
    fn payroll_net(&self) -> i64 {
        positive(self.payroll_total() - self.eic - self.excess_social)
    }

    // Credit Limit Worksheet B reserves only the child credit that cannot
    // become ACTC while it limits the homeowner credits.
    pub(crate) fn reserved(&self) -> i64 {
        positive(self.potential - self.actc_limit)
    }

    // Lines 13, 14, 16a, 17 and 27, after every credit ordered before the
    // nonrefundable child credit.
    pub(crate) fn limit(&self, creditable_tax: i64, preceding_credits: i64) -> ChildCredits {
        let limit = positive(creditable_tax - preceding_credits);
        let nonrefundable = phase::cap(self.potential, [limit]);
        ChildCredits {
            limit,
            nonrefundable,
            additional: phase::cap(positive(self.potential - nonrefundable), [self.actc_limit]),
        }
    }

    pub(crate) fn homeowner_credits(
        &self,
        tax: i64,
        preceding: i64,
        potentials: [i64; 4],
    ) -> HomeownerCredits {
        let mut available = positive(tax - preceding - self.reserved());
        let mut take = |potential| {
            crate::kernel::carry::native_consume_step(positive(potential), &mut available)
        };
        HomeownerCredits {
            mortgage: take(potentials[0]),
            adoption: take(potentials[1]),
            dc: take(potentials[2]),
            energy: take(potentials[3]),
        }
    }

    // Every printed line of the schedule except the identification entries
    // and the reserved line 15; the return rows drop a zero amount. Lines 2a
    // and 2c (Puerto Rico and American Samoa income) are not modeled.
    pub(crate) fn emit_lines(&self, credits: &ChildCredits, r: &mut FederalResult) {
        let excluded = self.magi - self.agi;
        row(r, "1", "child_credit_agi", self.agi, "EXPLAIN_CHILD_CREDIT_AGI");
        row(
            r,
            "2b",
            "child_credit_foreign_exclusions",
            excluded,
            "EXPLAIN_CHILD_CREDIT_FOREIGN_EXCLUSIONS",
        );
        row(
            r,
            "2d",
            "child_credit_exclusions_total",
            excluded,
            "EXPLAIN_CHILD_CREDIT_EXCLUSIONS_TOTAL",
        );
        row(r, "3", "child_credit_magi", self.magi, "EXPLAIN_CHILD_CREDIT_MAGI");
        count(r, "4", "child_credit_children", self.children, "EXPLAIN_CHILD_CREDIT_CHILDREN");
        row(
            r,
            "5",
            "child_credit_children_amount",
            self.children * crate::year_constants::CHILD_TAX_CREDIT_PER_CHILD_DOLLARS,
            "EXPLAIN_CHILD_CREDIT_CHILDREN_AMOUNT",
        );
        count(
            r,
            "6",
            "child_credit_other_dependents",
            self.other_dependents,
            "EXPLAIN_CHILD_CREDIT_OTHER_DEPENDENTS",
        );
        row(
            r,
            "7",
            "child_credit_other_dependents_amount",
            self.other_dependents * crate::year_constants::OTHER_DEPENDENT_CREDIT_DOLLARS,
            "EXPLAIN_CHILD_CREDIT_OTHER_DEPENDENTS_AMOUNT",
        );
        row(
            r,
            "8",
            "child_credit_before_phaseout",
            self.credit_before_phaseout(),
            "EXPLAIN_CHILD_CREDIT_BEFORE_PHASEOUT",
        );
        row(
            r,
            "9",
            "child_credit_phaseout_threshold",
            self.threshold,
            "EXPLAIN_CHILD_CREDIT_PHASEOUT_THRESHOLD",
        );
        row(
            r,
            "10",
            "child_credit_excess_income",
            self.excess_thousands * 1000,
            "EXPLAIN_CHILD_CREDIT_EXCESS_INCOME",
        );
        row(
            r,
            "11",
            "child_credit_phaseout_reduction",
            self.excess_thousands
                * crate::year_constants::CHILD_TAX_CREDIT_REDUCTION_PER_THOUSAND_DOLLARS,
            "EXPLAIN_CHILD_CREDIT_PHASEOUT_REDUCTION",
        );
        row(
            r,
            "12",
            "child_credit_after_phaseout",
            self.potential,
            "EXPLAIN_CHILD_CREDIT_AFTER_PHASEOUT",
        );
        row(r, "13", "child_credit_limit", credits.limit, "EXPLAIN_CHILD_CREDIT_LIMIT");
        // Line 14, Form 1040 line 19.
        form_line(
            r,
            FormLine::new(TaxForm::Schedule8812, "14"),
            "child_credit",
            credits.nonrefundable,
            INPUTS,
            SOURCE,
            msg!("EXPLAIN_CHILD_CREDIT"),
        );
        if !self.foreign_election {
            self.emit_part_ii(credits, r);
        }
        // Line 27, Form 1040 line 28.
        form_line(
            r,
            FormLine::new(TaxForm::Schedule8812, "27"),
            "additional_child_credit",
            credits.additional,
            INPUTS,
            SOURCE,
            msg!("EXPLAIN_ADDITIONAL_CHILD_CREDIT"),
        );
    }

    // Parts II-A and II-B: lines 16a to 26.
    fn emit_part_ii(&self, credits: &ChildCredits, r: &mut FederalResult) {
        let unused = positive(self.potential - credits.nonrefundable);
        let children_limit = self.children_limit();
        let smaller = phase::cap(unused, [children_limit]);
        let over_floor = positive(
            self.earned - crate::year_constants::CHILD_TAX_CREDIT_EARNED_INCOME_FLOOR_DOLLARS,
        );
        row(
            r,
            "16a",
            "additional_child_credit_unused",
            unused,
            "EXPLAIN_ADDITIONAL_CHILD_CREDIT_UNUSED",
        );
        row(
            r,
            "16b",
            "additional_child_credit_children_limit",
            children_limit,
            "EXPLAIN_ADDITIONAL_CHILD_CREDIT_CHILDREN_LIMIT",
        );
        row(
            r,
            "17",
            "additional_child_credit_smaller",
            smaller,
            "EXPLAIN_ADDITIONAL_CHILD_CREDIT_SMALLER",
        );
        row(
            r,
            "18a",
            "additional_child_credit_earned_income",
            self.earned,
            "EXPLAIN_ADDITIONAL_CHILD_CREDIT_EARNED_INCOME",
        );
        row(
            r,
            "19",
            "additional_child_credit_earned_over_floor",
            over_floor,
            "EXPLAIN_ADDITIONAL_CHILD_CREDIT_EARNED_OVER_FLOOR",
        );
        row(
            r,
            "20",
            "additional_child_credit_earned_share",
            self.earned_refund,
            "EXPLAIN_ADDITIONAL_CHILD_CREDIT_EARNED_SHARE",
        );
        // Part II-B applies when line 16b is $5,100 or more and line 20 is
        // less than line 17.
        if self.children >= 3 && self.earned_refund < smaller {
            row(
                r,
                "21",
                "additional_child_credit_withheld_taxes",
                self.withheld,
                "EXPLAIN_ADDITIONAL_CHILD_CREDIT_WITHHELD_TAXES",
            );
            row(
                r,
                "22",
                "additional_child_credit_other_payroll",
                self.other_payroll,
                "EXPLAIN_ADDITIONAL_CHILD_CREDIT_OTHER_PAYROLL",
            );
            row(
                r,
                "23",
                "additional_child_credit_payroll_total",
                self.payroll_total(),
                "EXPLAIN_ADDITIONAL_CHILD_CREDIT_PAYROLL_TOTAL",
            );
            row(
                r,
                "24",
                "additional_child_credit_eic_offset",
                self.eic + self.excess_social,
                "EXPLAIN_ADDITIONAL_CHILD_CREDIT_EIC_OFFSET",
            );
            row(
                r,
                "25",
                "additional_child_credit_payroll_net",
                self.payroll_net(),
                "EXPLAIN_ADDITIONAL_CHILD_CREDIT_PAYROLL_NET",
            );
            row(
                r,
                "26",
                "additional_child_credit_larger",
                phase::floor(self.earned_refund, self.payroll_net()),
                "EXPLAIN_ADDITIONAL_CHILD_CREDIT_LARGER",
            );
        }
    }
}

// One printed dollar line.
fn row(r: &mut FederalResult, line: &str, id: &str, amount: i64, key: &str) {
    form_line(
        r,
        FormLine::new(TaxForm::Schedule8812, line),
        id,
        amount,
        INPUTS,
        SOURCE,
        crate::Message::new(key, alloc::vec![]),
    );
}

// One printed line that is a number of people, not dollars.
fn count(r: &mut FederalResult, line: &str, id: &str, number: i64, key: &str) {
    form_count(
        r,
        FormLine::new(TaxForm::Schedule8812, line),
        id,
        number,
        INPUTS,
        SOURCE,
        crate::Message::new(key, alloc::vec![]),
    );
}
