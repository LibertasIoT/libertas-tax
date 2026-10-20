// Coverage: Form 8978 — Partial (2026-10-10).
// Single identified prior2025 ordinary-interest refigure; actual negative6l
// shares existing personal/3800/8801/8912 credit capacity once. Eligible PartII
// spillover retains explicit chapter1 classification limits. No general audit
// history reconstruction. Evidence: tests/federal/minimum_tax.rs,credits.rs,passthrough.rs;
// IRS2025 i1040 negative8978 worksheet;2026 draft i3800/i8801 and Schedule2.
use crate::kernel::phase;
use crate::{FederalBbaAuditInterest, FederalStatus, income_tax::Rates};

pub(crate) struct BbaAuditAdjustment {
    pub(crate) prior_tax: i64,
    pub(crate) corrected_tax: i64,
}

impl BbaAuditAdjustment {
    pub(crate) fn line_14(&self) -> i64 {
        self.corrected_tax - self.prior_tax
    }

    pub(crate) fn credit(&self) -> i64 {
        phase::floor(-self.line_14(), 0)
    }

    pub(crate) fn additional_tax(&self) -> i64 {
        phase::floor(self.line_14(), 0)
    }
}

impl FederalBbaAuditInterest {
    pub(crate) fn refigure(&self) -> Result<BbaAuditAdjustment, (&'static str, &'static str)> {
        if self.source_id.trim().is_empty() {
            return Err((
                "ISSUE_BBA_AUDIT_SOURCE_ID",
                crate::fixture_english!(
                    "Identify the single Form 8986 by its tracking number, audit control number, or issuing entity tax ID."
                ),
            ));
        }
        if !(20260101..=20261231).contains(&self.furnished_on)
            || !crate::federal_rules::valid_date(self.furnished_on)
        {
            return Err((
                "ISSUE_BBA_AUDIT_FURNISHED_YEAR",
                crate::fixture_english!(
                    "Only a Form 8986 actually furnished during the 2026 reporting year belongs on this return."
                ),
            ));
        }
        if !self.single_audit_adjustment || !self.simple_prior_return || !self.no_other_effects {
            return Err((
                "ISSUE_BBA_AUDIT_SCOPE",
                crate::fixture_english!(
                    "Confirm one BBA-audit portfolio-interest adjustment, a simple ordinary-tax 2025 return, and no other tax-attribute effects. Other Form 8978 cases need a complete prior-year refigure."
                ),
            ));
        }
        // Arithmetic safety: prior-income reconciliation adds native wages/interest
        // cents; line 17 enters the federal payment aggregate. The Tax Table bounds
        // below are actual supported prior-return scope.
        if self.interest_adjustment == 0
            || self.interest_adjustment < -100_000_000_000
            || self.interest_adjustment > 100_000_000_000
            || self.interest_adjustment % 100 != 0
            || self.original_partnership_interest + self.interest_adjustment < 0
            || self.original_partnership_interest > 100_000_000_000
            || self.original_partnership_interest + self.interest_adjustment > 100_000_000_000
            || self.original_partnership_interest % 100 != 0
            || self.prior_wages < 0
            || self.prior_wages > 100_000_000_000
            || self.prior_wages % 100 != 0
            || self.prior_taxable_income < 0
            || self.prior_taxable_income >= 10_000_000
            || self.prior_taxable_income % 100 != 0
            || self.prior_income_tax < 0
            || self.prior_income_tax % 100 != 0
            || !(0..=100_000_000_000).contains(&self.line_17_interest)
            || self.line_17_interest % 100 != 0
            || (self.interest_adjustment < 0 && self.line_17_interest != 0)
            || self.prior_taxable_income + self.interest_adjustment < 0
            || self.prior_taxable_income + self.interest_adjustment >= 10_000_000
        {
            return Err((
                "ISSUE_BBA_AUDIT_WHOLE_DOLLAR_VALUES",
                crate::fixture_english!(
                    "Use whole-dollar 2025 Form 1040 and partnership-interest amounts. The signed Form 8986 adjustment cannot make the corrected K-1 interest or taxable income negative; both original and corrected income must use the 2025 Tax Table."
                ),
            ));
        }
        // 2025 Form 1040 basic standard deduction; age, blindness and dependent
        // adjustments are excluded by the explicit source confirmation.
        // https://www.irs.gov/pub/irs-prior/f1040--2025.pdf
        let basic_deduction = match self.prior_status {
            FederalStatus::Single | FederalStatus::Separate => {
                crate::year_constants::STANDARD_DEDUCTION_2025_SINGLE_OR_SEPARATE_DOLLARS
            }
            FederalStatus::Joint | FederalStatus::Surviving => {
                crate::year_constants::STANDARD_DEDUCTION_2025_JOINT_DOLLARS
            }
            FederalStatus::Head => crate::year_constants::STANDARD_DEDUCTION_2025_HEAD_DOLLARS,
        };
        if phase::floor(
            self.prior_wages + self.original_partnership_interest - basic_deduction * 100,
            0,
        ) != self.prior_taxable_income
        {
            return Err((
                "ISSUE_BBA_AUDIT_PRIOR_INCOME_RECONCILIATION",
                crate::fixture_english!(
                    "The source wages and original K-1 interest do not reconcile to 2025 taxable income after the basic standard deduction. Other income or deductions require a full Form 8978 refigure."
                ),
            ));
        }
        let Some(rates) = Rates::with_method(2025, self.prior_status, true) else {
            return Err((
                "ISSUE_BBA_AUDIT_TAX_TABLE_STATUS",
                crate::fixture_english!(
                    "The 2025 filing status cannot be calculated from the published Tax Table."
                ),
            ));
        };
        let original = rates.schedule_d(
            crate::kernel::ratio::trunc_i64(self.prior_taxable_income, 100),
            0,
            0,
            0,
            false,
        );
        if crate::kernel::ratio::trunc_i64(self.prior_income_tax, 100) != original {
            return Err((
                "ISSUE_BBA_AUDIT_PRIOR_TAX_RECONCILIATION",
                crate::fixture_english!(
                    "The original 2025 Form 1040 income tax does not match the stated ordinary taxable income and filing status; refigure that return before using Form 8978."
                ),
            ));
        }
        let corrected = rates.schedule_d(
            crate::kernel::ratio::trunc_i64(
                self.prior_taxable_income + self.interest_adjustment,
                100,
            ),
            0,
            0,
            0,
            false,
        );
        if self.interest_adjustment > 0 {
            if corrected <= original {
                return Err((
                    "ISSUE_BBA_AUDIT_TAX_INCREASE_REQUIRED",
                    crate::fixture_english!(
                        "This narrow positive Form 8978 route requires an increase in the refigured 2025 income tax."
                    ),
                ));
            }
            if !self.interest_calculation_prepared {
                return Err((
                    "ISSUE_BBA_AUDIT_INTEREST_DOCUMENTATION",
                    crate::fixture_english!(
                        "Compute and document Form 8978 line 17 interest on the additional 2025 income tax before using this route; that interest is part of the reporting-year net payment, outside Form 1040 tax."
                    ),
                ));
            }
        }
        Ok(BbaAuditAdjustment { prior_tax: original, corrected_tax: corrected })
    }
}
