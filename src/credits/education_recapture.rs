// 2026 recapture of a 2025 education credit after a post-filing refund or
// tax-free education assistance. Source: Publication 970 (2025), chapters 2–3.
use crate::EducationMethod;
pub use crate::education_recapture_source::PriorYearEducationCreditRecapture;
use crate::kernel::phase;
use crate::kernel::ratio::{dollars, fraction};

impl PriorYearEducationCreditRecapture {
    /// Additional 2026 tax in whole dollars for this ordinary prior-return history.
    pub(crate) fn additional_tax(&self) -> Result<i64, (&'static str, &'static str)> {
        if !self.valid() {
            return Err((
                "ISSUE_EDUCATION_RECAPTURE_AMOUNTS",
                crate::fixture_english!(
                    "Reconcile the filed 2025 education expenses, credit and later refund or assistance amounts."
                ),
            ));
        }
        if !self.ordinary_prior_return {
            return Err((
                "ISSUE_EDUCATION_RECAPTURE_COMPLEX_PRIOR_RETURN",
                crate::fixture_english!(
                    "A limited or competing 2025 education credit needs a complete prior-return refigure before calculating 2026 recapture."
                ),
            ));
        }

        let before = self.education_credit(dollars(self.filed_qualified_expenses));
        if self.filed_credit_used != before * 100 {
            return Err((
                "ISSUE_EDUCATION_RECAPTURE_PRIOR_CREDIT_MISMATCH",
                crate::fixture_english!(
                    "The filed 2025 Form 8863 credit does not match the ordinary single-credit calculation; reconcile the retained prior return."
                ),
            ));
        }
        let after = self
            .education_credit(dollars(self.filed_qualified_expenses - self.after_filing_reduction));
        Ok(before - after)
    }

    fn education_credit(&self, expense: i64) -> i64 {
        let cap = if self.filed_joint {
            crate::year_constants::EDUCATION_CREDIT_PHASEOUT_END_JOINT_DOLLARS
        } else {
            crate::year_constants::EDUCATION_CREDIT_PHASEOUT_END_OTHER_DOLLARS
        };
        let range = if self.filed_joint {
            crate::year_constants::EDUCATION_CREDIT_PHASEOUT_RANGE_JOINT_DOLLARS
        } else {
            crate::year_constants::EDUCATION_CREDIT_PHASEOUT_RANGE_OTHER_DOLLARS
        };
        let factor = phase::bound(cap - dollars(self.filed_education_magi), 0, range);
        match self.method {
            EducationMethod::AmericanOpportunity => phase::remaining_fraction(
                phase::cap(expense, [2_000])
                    + fraction(phase::cap(phase::floor(expense - 2_000, 0), [2_000]), 1, 4),
                factor,
                range,
            ),
            EducationMethod::LifetimeLearning => {
                lifetime_learning_credit(phase::cap(expense, [10_000]), factor, range)
            }
        }
    }
}

/// Form 8863 lifetime learning credit: 20% of the expenses (already limited to $10,000 per
/// return) phased out by `factor / range`, rounded once.
pub(crate) fn lifetime_learning_credit(expenses: i64, factor: i64, range: i64) -> i64 {
    phase::remaining_fraction(expenses * 20, factor, range * 100)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn history(
        method: EducationMethod,
        expense: i64,
        reduction: i64,
        credit: i64,
    ) -> PriorYearEducationCreditRecapture {
        PriorYearEducationCreditRecapture {
            method,
            filed_qualified_expenses: expense * 100,
            after_filing_reduction: reduction * 100,
            filed_education_magi: 50_000 * 100,
            filed_joint: false,
            filed_credit_used: credit * 100,
            ordinary_prior_return: true,
        }
    }

    #[test]
    fn publication_970_american_opportunity_refund_example() {
        // $7,000 paid in 2025, $4,000 refunded after filing in 2026.
        let record = history(EducationMethod::AmericanOpportunity, 7_000, 4_000, 2_500);
        assert_eq!(record.additional_tax(), Ok(250));
    }

    #[test]
    fn publication_970_lifetime_learning_refund_example() {
        // $9,300 paid in 2025, $2,900 refunded after filing in 2026.
        let record = history(EducationMethod::LifetimeLearning, 9_300, 2_900, 1_860);
        assert_eq!(record.additional_tax(), Ok(580));
    }

    #[test]
    fn prior_year_phaseout_applies_before_and_after_refigure() {
        let mut record = history(EducationMethod::AmericanOpportunity, 4_000, 1_000, 1_250);
        record.filed_education_magi = 85_000 * 100;
        assert_eq!(record.additional_tax(), Ok(125));
    }

    #[test]
    fn lifetime_learning_refigure_rounds_once_like_the_return() {
        // $3 of expenses at a half phase-out is $0.30 of credit, which the return rounds to 0.
        let mut record = history(EducationMethod::LifetimeLearning, 3, 1, 0);
        record.filed_education_magi = 85_000 * 100;
        assert_eq!(record.additional_tax(), Ok(0));
    }

    #[test]
    fn no_prior_tax_benefit_means_no_recapture() {
        let mut record = history(EducationMethod::LifetimeLearning, 9_300, 2_900, 0);
        record.filed_education_magi = 90_000 * 100;
        assert_eq!(record.additional_tax(), Ok(0));
    }

    #[test]
    fn unverified_tax_capacity_or_prior_credit_is_not_silently_accepted() {
        let mut record = history(EducationMethod::AmericanOpportunity, 7_000, 4_000, 2_500);
        record.ordinary_prior_return = false;
        assert_eq!(
            record.additional_tax().unwrap_err().0,
            "ISSUE_EDUCATION_RECAPTURE_COMPLEX_PRIOR_RETURN"
        );
        record.ordinary_prior_return = true;
        record.filed_credit_used = 2_000 * 100;
        assert_eq!(
            record.additional_tax().unwrap_err().0,
            "ISSUE_EDUCATION_RECAPTURE_PRIOR_CREDIT_MISMATCH"
        );
        record.filed_credit_used = 2_500 * 100;
        record.after_filing_reduction = 8_000 * 100;
        assert_eq!(record.additional_tax().unwrap_err().0, "ISSUE_EDUCATION_RECAPTURE_AMOUNTS");
    }
}
