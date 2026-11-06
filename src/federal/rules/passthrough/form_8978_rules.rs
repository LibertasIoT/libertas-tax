// form_8978 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) fn bba_audit_reporting_phase(
    r: &mut FederalResult,
    bba_audit: &Option<crate::bba_audit::BbaAuditAdjustment>,
) {
    if let Some(audit) = bba_audit {
        line(
            r,
            "form_8978_prior_income_tax",
            audit.prior_tax,
            &["credits.bba_audit_interest.answers.prior_taxable_income"],
            "https://www.irs.gov/instructions/i8978",
            msg!("EXPLAIN_FORM_N_PRIOR_INCOME_TAX"),
        );
        line(
            r,
            "form_8978_corrected_income_tax",
            audit.corrected_tax,
            &["credits.bba_audit_interest.document.interest_adjustment"],
            "https://www.irs.gov/instructions/i8978",
            msg!("EXPLAIN_FORM_N_CORRECTED_INCOME_TAX"),
        );
        line(
            r,
            "form_8978_line_14",
            audit.line_14(),
            &["form_8978_prior_income_tax", "form_8978_corrected_income_tax"],
            "https://www.irs.gov/instructions/i8978",
            msg!("EXPLAIN_FORM_N_LINE_N"),
        );
    }
}
