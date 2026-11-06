use crate::FederalResult;
pub use crate::alimony_model::{AlimonyNewRules, AlimonyPaid, AlimonyPayment, AlimonyRecapture};
use crate::kernel::ratio::{dollars, fraction, positive};

impl AlimonyPayment {
    // `side` names the page the refusal opens: "Alimony paid" or "Alimony
    // received". An unconfirmed payment (No) takes each side's
    // no-benefit path: the payer deducts nothing, the recipient is taxed.
    pub(crate) fn checked(
        &self,
        r: &mut FederalResult,
        context: &crate::IssueContext,
        payer: bool,
        #[cfg(test)] side: &str,
    ) -> i64 {
        let pre_2019 = self.agreement < 20190101;
        crate::require_localized!(
            r,
            !pre_2019 || self.new_rules != AlimonyNewRules::DuringReturnYear,
            &context.clone().manual(),
            side,
            "ISSUE_AN_AGREEMENT_CHANGED_DURING_RETURN_YEAR_TO_THE_POST_2018_RULES_OR_REPLACED_F80A1653",
            "An agreement changed during 2026 to the post-2018 rules, or replaced during 2026 by a divorce decree signed after 2018, splits this year's payments at the change (Pub. 504): Schedule 1 line 2a or 19a for such a year is not calculated yet.",
        );
        let ordinary = pre_2019 && self.new_rules == AlimonyNewRules::No;
        crate::require_localized!(
            r,
            !ordinary || self.agreement >= 19850101,
            &context.clone().manual(),
            side,
            "ISSUE_CONFIRM_ORDINARY_QUALIFYING_ALIMONY_AND_RESOLVE_RECAPTURE_PRE_1985_DABB0528",
            "Alimony under an instrument executed before 1985 follows special rules that are not calculated.",
        );
        if ordinary && !self.eligible && payer {
            return 0;
        }
        self.amount()
    }
}
impl AlimonyPayment {
    // `side` names the page the refusal opens: "Alimony recapture (paid)"
    // or "Alimony recapture (received)". For an unconfirmed payment the payer
    // still includes recapture and the recipient deducts none.
    pub(crate) fn recaptured(
        &self,
        r: &mut FederalResult,
        context: &crate::IssueContext,
        payer: bool,
        #[cfg(test)] side: &str,
    ) -> i64 {
        let AlimonyRecapture::ThirdYear {
            first,
            second,
            first_excluded,
            second_excluded,
            current_excluded,
            death_or_remarriage,
        } = self.recapture
        else {
            return 0;
        };
        crate::require_localized!(
            r,
            (19870101..20190101).contains(&self.agreement) && self.new_rules == AlimonyNewRules::No,
            &context.clone().manual(),
            side,
            "ISSUE_CONFIRM_THE_THIRD_YEAR_PAYMENT_HISTORY_AND_EXCEPTION_PRE_1987_INSTR_F7A45E5F",
            "Third-year recapture for a pre-1987 instrument, or one changed to the post-2018 rules during the recapture period, needs separate review.",
        );
        if death_or_remarriage || (!payer && !self.eligible) {
            return 0;
        }
        let first = dollars(first - first_excluded);
        let second = dollars(second - second_excluded);
        let current = dollars(self.amount() - current_excluded);
        let second_excess = positive(second - current - 15000);
        let first_excess =
            positive(first - fraction(second - second_excess + current, 1, 2) - 15000);
        (second_excess + first_excess) * 100
    }
}
