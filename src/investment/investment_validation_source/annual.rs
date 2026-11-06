use crate::annual_debt_source::AnnualDebtRecords;
use crate::debt_reporting_source::DebtReporting;
use crate::validation::ValidationIssue;

pub(super) fn int_issue(
    v: &crate::interest_source::FederalInterest,
    records: &AnnualDebtRecords,
    source_alignment: bool,
) -> Option<ValidationIssue> {
    if let Some(issue) = records.issue() {
        return Some(issue.in_field("instrument_records").in_field("adjustments"));
    }
    if v.nominee.is_some()
        || v.after_death.is_some()
        || v.adjustments.as_ref().is_some_and(|a| {
            a.previously_reported != 0
                || a.reporting != DebtReporting::Gross
                || a.net_premium_offset.is_some()
        })
    {
        return Some(
            ValidationIssue::new("DEBT_PAYER_ALLOCATION_OVERLAP")
                .at_field("instrument_records")
                .in_field("adjustments"),
        );
    }
    if source_alignment
        && records.records.iter().map(|r| r.amounts.taxable).sum::<i64>() != v.taxable
    {
        return Some(ValidationIssue::new("DEBT_INSTRUMENT_TOTAL_MISMATCH").at_field("taxable"));
    }
    if source_alignment
        && records.records.iter().map(|r| r.amounts.us_obligations).sum::<i64>() != v.us_obligations
    {
        return Some(
            ValidationIssue::new("DEBT_INSTRUMENT_TOTAL_MISMATCH").at_field("us_obligations"),
        );
    }
    if source_alignment && records.records.iter().map(|r| r.amounts.exempt).sum::<i64>() != v.exempt
    {
        return Some(ValidationIssue::new("DEBT_INSTRUMENT_TOTAL_MISMATCH").at_field("exempt"));
    }
    if source_alignment
        && records.records.iter().map(|r| r.amounts.private_activity).sum::<i64>()
            != v.private_activity
    {
        return Some(
            ValidationIssue::new("DEBT_INSTRUMENT_TOTAL_MISMATCH").at_field("private_activity"),
        );
    }
    if source_alignment
        && records.records.iter().map(|r| r.amounts.foreign_tax).sum::<i64>() != v.foreign_tax
    {
        return Some(
            ValidationIssue::new("DEBT_INSTRUMENT_TOTAL_MISMATCH").at_field("foreign_tax"),
        );
    }
    if source_alignment
        && records.records.iter().map(|r| r.amounts.market_discount).sum::<i64>()
            != v.market_discount
    {
        return Some(
            ValidationIssue::new("DEBT_INSTRUMENT_TOTAL_MISMATCH").at_field("market_discount"),
        );
    }
    if source_alignment
        && records.records.iter().map(|r| r.amounts.bond_premium).sum::<i64>() != v.bond_premium
    {
        return Some(
            ValidationIssue::new("DEBT_INSTRUMENT_TOTAL_MISMATCH").at_field("bond_premium"),
        );
    }
    if source_alignment
        && records.records.iter().map(|r| r.amounts.treasury_bond_premium).sum::<i64>()
            != v.treasury_bond_premium
    {
        return Some(
            ValidationIssue::new("DEBT_INSTRUMENT_TOTAL_MISMATCH")
                .at_field("treasury_bond_premium"),
        );
    }
    if source_alignment
        && records.records.iter().map(|r| r.amounts.tax_exempt_bond_premium).sum::<i64>()
            != v.tax_exempt_bond_premium
    {
        return Some(
            ValidationIssue::new("DEBT_INSTRUMENT_TOTAL_MISMATCH")
                .at_field("tax_exempt_bond_premium"),
        );
    }
    if source_alignment
        && records
            .records
            .iter()
            .map(|r| r.amounts.accrued_interest + r.amounts.treasury_accrued_interest)
            .sum::<i64>()
            != v.accrued_interest
    {
        return Some(
            ValidationIssue::new("DEBT_INSTRUMENT_TOTAL_MISMATCH").at_field("accrued_interest"),
        );
    }
    if let Some(index) = records.records.iter().position(|r| r.amounts.acquisition_premium != 0) {
        return Some(
            ValidationIssue::new("DEBT_WRONG_STATEMENT_COMPONENT")
                .at_field("acquisition_premium")
                .in_field("amounts")
                .in_list("records", index)
                .in_field("instrument_records")
                .in_field("adjustments"),
        );
    }
    if let Some(index) = records.records.iter().position(|r| r.amounts.oid != 0) {
        return Some(
            ValidationIssue::new("DEBT_WRONG_STATEMENT_COMPONENT")
                .at_field("oid")
                .in_field("amounts")
                .in_list("records", index)
                .in_field("instrument_records")
                .in_field("adjustments"),
        );
    }
    if let Some(index) = records.records.iter().position(|r| r.amounts.other_periodic_interest != 0)
    {
        return Some(
            ValidationIssue::new("DEBT_WRONG_STATEMENT_COMPONENT")
                .at_field("other_periodic_interest")
                .in_field("amounts")
                .in_list("records", index)
                .in_field("instrument_records")
                .in_field("adjustments"),
        );
    }
    if let Some(index) =
        records.records.iter().position(|r| r.amounts.treasury_acquisition_premium != 0)
    {
        return Some(
            ValidationIssue::new("DEBT_WRONG_STATEMENT_COMPONENT")
                .at_field("treasury_acquisition_premium")
                .in_field("amounts")
                .in_list("records", index)
                .in_field("instrument_records")
                .in_field("adjustments"),
        );
    }
    if let Some(index) = records.records.iter().position(|r| r.amounts.treasury_oid != 0) {
        return Some(
            ValidationIssue::new("DEBT_WRONG_STATEMENT_COMPONENT")
                .at_field("treasury_oid")
                .in_field("amounts")
                .in_list("records", index)
                .in_field("instrument_records")
                .in_field("adjustments"),
        );
    }
    None
}

pub(super) fn oid_issue(
    v: &crate::oid_source::FederalOid,
    records: &AnnualDebtRecords,
    source_alignment: bool,
) -> Option<ValidationIssue> {
    if let Some(issue) = records.issue() {
        return Some(issue.in_field("instrument_records").in_field("adjustments"));
    }
    if v.nominee.is_some()
        || v.adjustments.as_ref().is_some_and(|a| {
            a.previously_reported != 0
                || a.reporting != DebtReporting::Gross
                || a.net_premium_offset.is_some()
        })
    {
        return Some(
            ValidationIssue::new("DEBT_PAYER_ALLOCATION_OVERLAP")
                .at_field("instrument_records")
                .in_field("adjustments"),
        );
    }
    if source_alignment && records.records.iter().map(|r| r.amounts.oid).sum::<i64>() != v.oid {
        return Some(ValidationIssue::new("DEBT_INSTRUMENT_TOTAL_MISMATCH").at_field("oid"));
    }
    if source_alignment
        && records.records.iter().map(|r| r.amounts.other_periodic_interest).sum::<i64>()
            != v.other_periodic_interest
    {
        return Some(
            ValidationIssue::new("DEBT_INSTRUMENT_TOTAL_MISMATCH")
                .at_field("other_periodic_interest"),
        );
    }
    if source_alignment
        && records.records.iter().map(|r| r.amounts.treasury_oid).sum::<i64>() != v.treasury_oid
    {
        return Some(
            ValidationIssue::new("DEBT_INSTRUMENT_TOTAL_MISMATCH").at_field("treasury_oid"),
        );
    }
    if source_alignment && records.records.iter().map(|r| r.amounts.exempt).sum::<i64>() != v.exempt
    {
        return Some(ValidationIssue::new("DEBT_INSTRUMENT_TOTAL_MISMATCH").at_field("exempt"));
    }
    if source_alignment
        && records.records.iter().map(|r| r.amounts.private_activity).sum::<i64>()
            != v.private_activity
    {
        return Some(
            ValidationIssue::new("DEBT_INSTRUMENT_TOTAL_MISMATCH").at_field("private_activity"),
        );
    }
    if source_alignment
        && records.records.iter().map(|r| r.amounts.market_discount).sum::<i64>()
            != v.market_discount
    {
        return Some(
            ValidationIssue::new("DEBT_INSTRUMENT_TOTAL_MISMATCH").at_field("market_discount"),
        );
    }
    if source_alignment
        && records
            .records
            .iter()
            .map(|r| r.amounts.bond_premium + r.amounts.treasury_bond_premium)
            .sum::<i64>()
            != v.bond_premium
    {
        return Some(
            ValidationIssue::new("DEBT_INSTRUMENT_TOTAL_MISMATCH").at_field("bond_premium"),
        );
    }
    if source_alignment
        && records
            .records
            .iter()
            .map(|r| r.amounts.acquisition_premium + r.amounts.treasury_acquisition_premium)
            .sum::<i64>()
            != v.acquisition_premium
    {
        return Some(
            ValidationIssue::new("DEBT_INSTRUMENT_TOTAL_MISMATCH").at_field("acquisition_premium"),
        );
    }
    if source_alignment
        && records.records.iter().map(|r| r.amounts.tax_exempt_bond_premium).sum::<i64>()
            != v.tax_exempt_bond_premium
    {
        return Some(
            ValidationIssue::new("DEBT_INSTRUMENT_TOTAL_MISMATCH")
                .at_field("tax_exempt_bond_premium"),
        );
    }
    if let Some(index) = records.records.iter().position(|r| r.amounts.accrued_interest != 0) {
        return Some(
            ValidationIssue::new("DEBT_WRONG_STATEMENT_COMPONENT")
                .at_field("accrued_interest")
                .in_field("amounts")
                .in_list("records", index)
                .in_field("instrument_records")
                .in_field("adjustments"),
        );
    }
    if let Some(index) = records.records.iter().position(|r| r.amounts.foreign_tax != 0) {
        return Some(
            ValidationIssue::new("DEBT_WRONG_STATEMENT_COMPONENT")
                .at_field("foreign_tax")
                .in_field("amounts")
                .in_list("records", index)
                .in_field("instrument_records")
                .in_field("adjustments"),
        );
    }
    if let Some(index) = records.records.iter().position(|r| r.amounts.taxable != 0) {
        return Some(
            ValidationIssue::new("DEBT_WRONG_STATEMENT_COMPONENT")
                .at_field("taxable")
                .in_field("amounts")
                .in_list("records", index)
                .in_field("instrument_records")
                .in_field("adjustments"),
        );
    }
    if let Some(index) =
        records.records.iter().position(|r| r.amounts.treasury_accrued_interest != 0)
    {
        return Some(
            ValidationIssue::new("DEBT_WRONG_STATEMENT_COMPONENT")
                .at_field("treasury_accrued_interest")
                .in_field("amounts")
                .in_list("records", index)
                .in_field("instrument_records")
                .in_field("adjustments"),
        );
    }
    if let Some(index) = records.records.iter().position(|r| r.amounts.us_obligations != 0) {
        return Some(
            ValidationIssue::new("DEBT_WRONG_STATEMENT_COMPONENT")
                .at_field("us_obligations")
                .in_field("amounts")
                .in_list("records", index)
                .in_field("instrument_records")
                .in_field("adjustments"),
        );
    }
    None
}
