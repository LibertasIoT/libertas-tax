// Coverage: Form 8978 — Partial (2026-10-10).
// Single identified prior2025 ordinary-interest refigure; actual negative6l
// shares existing personal/3800/8801/8912 credit capacity once. Eligible PartII
// spillover retains explicit chapter1 classification limits. No general audit
// history reconstruction. Evidence: tests/federal/minimum_tax.rs,credits.rs,passthrough.rs;
// IRS2025 i1040 negative8978 worksheet;2026 draft i3800/i8801 and Schedule2.
use crate::kernel::phase;
use crate::{
    BbaAuditAnswers, FederalBbaAuditInterest, FederalStatus, Form8986Paper, income_tax::Rates,
};

pub(crate) struct BbaAuditAdjustment {
    pub(crate) prior_tax: i64,
    pub(crate) corrected_tax: i64,
    pub(crate) line_17_interest: i64,
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

const WHOLE_DOLLAR_VALUES_ISSUE: (&str, &str) = (
    "ISSUE_BBA_AUDIT_WHOLE_DOLLAR_VALUES",
    crate::fixture_english!(
        "Use whole-dollar 2025 Form 1040 and partnership-interest amounts. The signed Form 8986 adjustment cannot make the corrected K-1 interest or taxable income negative; both original and corrected income must use the 2025 Tax Table."
    ),
);

impl Form8986Paper {
    pub(crate) fn issue(&self) -> Option<(&'static str, &'static str)> {
        // The supported adjustment is Part V's one Schedule K-1 line 5 interest line.
        if self
            .interest_item()
            .is_none_or(|item| item.as_reported.is_none() || item.adjustment.is_none())
        {
            return Some((
                "ISSUE_BBA_AUDIT_SCOPE",
                crate::fixture_english!(
                    "This calculation supports a Form 8986 whose Part V has one line: Schedule K-1 line 5 interest, with columns (d) and (g) printed."
                ),
            ));
        }
        form_8986_issue(
            &self.source_id(),
            self.interest_adjustment(),
            self.original_partnership_interest(),
        )
    }
}

fn form_8986_issue(
    source_id: &str,
    interest_adjustment: i64,
    original_partnership_interest: i64,
) -> Option<(&'static str, &'static str)> {
    if source_id.trim().is_empty() {
        return Some((
            "ISSUE_BBA_AUDIT_SOURCE_ID",
            crate::fixture_english!(
                "Identify the single Form 8986 by its tracking number, audit control number, or issuing entity tax ID."
            ),
        ));
    }
    // Arithmetic safety: the printed amounts and their corrected total must
    // remain within the supported native-cent bounds.
    if interest_adjustment == 0
        || !(-100_000_000_000..=100_000_000_000).contains(&interest_adjustment)
        || interest_adjustment % 100 != 0
        || original_partnership_interest > 100_000_000_000
        || original_partnership_interest + interest_adjustment < 0
        || original_partnership_interest + interest_adjustment > 100_000_000_000
        || original_partnership_interest % 100 != 0
    {
        return Some(WHOLE_DOLLAR_VALUES_ISSUE);
    }
    None
}

impl FederalBbaAuditInterest {
    pub(crate) fn refigure(&self) -> Result<BbaAuditAdjustment, (&'static str, &'static str)> {
        if let Some(issue) = form_8986_issue(
            &self.source_id,
            self.interest_adjustment,
            self.original_partnership_interest,
        ) {
            return Err(issue);
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
        if self.prior_wages < 0
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
            return Err(WHOLE_DOLLAR_VALUES_ISSUE);
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
        // An increase that stays within its Tax Table row changes no tax: line 14
        // is zero and no line 17 interest accrues (i8978).
        let increased = corrected > original;
        if increased && !self.interest_calculation_prepared {
            return Err((
                "ISSUE_BBA_AUDIT_INTEREST_DOCUMENTATION",
                crate::fixture_english!(
                    "Compute and document Form 8978 line 17 interest on the additional 2025 income tax before using this route; that interest is part of the reporting-year net payment, outside Form 1040 tax."
                ),
            ));
        }
        Ok(BbaAuditAdjustment {
            prior_tax: original,
            corrected_tax: corrected,
            line_17_interest: if increased { self.line_17_interest } else { 0 },
        })
    }
}

impl BbaAuditAnswers {
    pub(crate) fn model(
        &self,
        id: i64,
        document: &Form8986Paper,
    ) -> Result<FederalBbaAuditInterest, (&'static str, &'static str)> {
        let Some(prior_status) = self.prior_status else {
            return Err((
                "ISSUE_BBA_AUDIT_TAX_TABLE_STATUS",
                crate::fixture_english!(
                    "Supply the filing status on the filed or IRS-adjusted 2025 return."
                ),
            ));
        };
        let (Some(prior_wages), Some(prior_taxable_income), Some(prior_income_tax)) =
            (self.prior_wages, self.prior_taxable_income, self.prior_income_tax)
        else {
            return Err((
                "ISSUE_BBA_AUDIT_WHOLE_DOLLAR_VALUES",
                crate::fixture_english!(
                    "Supply the wages, taxable income and income tax from the filed or IRS-adjusted 2025 return. Missing records are not zero."
                ),
            ));
        };
        let (Some(single_audit_adjustment), Some(simple_prior_return), Some(no_other_effects)) =
            (self.single_audit_adjustment, self.simple_prior_return, self.no_other_effects)
        else {
            return Err((
                "ISSUE_BBA_AUDIT_SCOPE",
                crate::fixture_english!(
                    "Confirm the adjustment's scope, the prior-return facts and whether other tax effects apply."
                ),
            ));
        };
        Ok(FederalBbaAuditInterest {
            partnership: document.partnership.clone(),
            reviewed_year_partner: document.partner.clone(),
            id,
            source_id: document.source_id(),
            furnished_on: document.furnished_on,
            interest_adjustment: document.interest_adjustment(),
            original_partnership_interest: document.original_partnership_interest(),
            prior_wages,
            prior_taxable_income,
            prior_income_tax,
            prior_status,
            single_audit_adjustment,
            simple_prior_return,
            no_other_effects,
            // Form 8978 line 17 applies only to additional tax; the refigure below
            // decides whether the Tax Table row moved.
            line_17_interest: self.line_17_interest.unwrap_or(0),
            interest_calculation_prepared: self.interest_calculation_prepared.unwrap_or(false),
        })
        .and_then(|model| {
            // A confirmed calculation still needs its actual amount, even zero.
            if self.line_17_interest.is_none()
                && model.interest_calculation_prepared
                && model.refigure().is_ok_and(|adjustment| adjustment.additional_tax() > 0)
            {
                return Err((
                    "ISSUE_BBA_AUDIT_INTEREST_DOCUMENTATION",
                    crate::fixture_english!(
                        "Supply and confirm the documented Form 8978 line 17 interest calculation, including its actual amount when zero."
                    ),
                ));
            }
            Ok(model)
        })
    }
}
