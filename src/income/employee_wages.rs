// Coverage: Form 8919 — Supported (unit B6b, 2026-10-09; audited against f8919 (2026 draft),
//   Form 1040 line 1g, Schedule 2 line 16b, Form 8959 line 3, 26 U.S.C. 3101 and 3121).
//   Lines: the Wages page's Employee pay without a W-2 (Form 8919) kind, one for each firm
//   (EmployeeWages): line 6 pay to Form 1040 line 1g, line 10 the pay under the Social Security
//   wage base left after W-2 box 3 wages, covered tips and Form 4137 wages, lines 11 (6.2%) and
//   12 (1.45%) in whole dollars and line 13 to Schedule 2 line 16b, per person; the pay is
//   wages for the EIC, IRA compensation and section 179, and a 1099 firm's withholding is line
//   25b (other withholding 25c). Reason codes A and C (dated), G (Form SS-8 filed; one review
//   note) and H (the same firm's W-2 and 1099; the Form 1099 is required). 1099-NEC box 1 or
//   1099-MISC box 3 and box 4 are imported; the reason and the ordinary-coverage answer are the
//   person's.
//   Exclusions: railroad (RRTA) pay, a section 218 state or local employee, totalization or
//   foreign employment payroll tax (coverage No preserves pay on Form 1040 line 1h,
//   with WARN_EMPLOYEE_WAGES_PAYROLL_TAX_LEFT_OUT). Earlier-year corrections are not
//   entered as 2026 pay. Form SS-8 itself and the firm's name and EIN
//   (printed columns, not collected).
//   Evidence: this module, federal_rules.rs, section179.rs, archer_contributions.rs,
//   unknowns.rs; tests: tests/relief_income.rs a_form_8919_* (fixture A: line 6 5,000, 11 310, 12
//   73 (72.50), 13 383, tax 5,623), tests/business.rs employee_8919_*, health_accounts.rs
//   employee_8919_*, taxes.rs form_8919_withholding_uses_the_reported_document_category.
// Form 8919: employee compensation omitted from W-2, not Schedule C income.
use libertas_macros::*;

/// Employee pay without a W-2 (Form 8919)
/// You owe your half of Social Security and Medicare tax on this pay instead of self-employment tax. Not pay already on a W-2, tips, or pay for work you did as an independent contractor.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read one Form 1099-NEC or Form 1099-MISC from this firm exactly. Check the form's tax year against the supplied year and report a mismatch as an error. For a 1099-NEC read box 1 nonemployee compensation; for a 1099-MISC read box 3 other income; report an error when that box is blank. Read box 4 federal income tax withheld, returning an empty string when it is clearly blank. Return true for form1099. Report an illegible amount, or two values that conflict, as an error. Do not read any other box. Do not choose the reason code, its date or whether ordinary payroll coverage applies; the filer answers those. The file may hold several copies of this form, such as Copy B and Copy 2; copies of the same form are one form.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*"
)]
pub struct EmployeeWages {
    /// Pay received as this firm's employee with no Social Security or Medicare tax withheld
    /// Form 8919 column (f). With reason H, only the 1099 amount that belonged on the firm's W-2. Do not also enter it as business receipts.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub wages: i64,
    /// Federal income tax withheld from this pay
    /// Including backup withholding; not Social Security or Medicare tax.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub withholding: i64,
    /// You received a Form 1099-NEC or 1099-MISC from this firm for this pay
    /// Form 8919 column (e).
    #[libertas_llm_doc_field]
    pub form1099: bool,
    /// Why you are filing Form 8919
    pub reason: EmployeeWageReason,
    /// Ordinary employee pay covered by Social Security and Medicare
    /// No for railroad (RRTA) pay or a state or local position whose coverage depends on a section 218 agreement. No keeps the pay on Form 1040 line 1h and leaves out only its payroll tax with a notice; your tax may be higher. Enter only 2026 pay; earlier pay belongs on that year's return.
    pub complete: bool,
}

/// Form 8919 filing reason
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum EmployeeWageReason {
    /// IRS employee determination after Form SS-8 — code A
    Ss8Determination {
        /// Date of IRS determination
        #[libertas_date_only]
        date: u32,
    },
    /// Other IRS correspondence establishing employee status — code C
    IrsCorrespondence {
        /// Date of IRS correspondence
        #[libertas_date_only]
        date: u32,
    },
    /// Form SS-8 filed; IRS reply pending — code G
    /// Form SS-8 must be filed separately on or before the return filing date. A pending request does not guarantee IRS agreement with employee status.
    Ss8Pending {
        /// Date Form SS-8 was filed
        #[libertas_date_only]
        filed_on: u32,
    },
    /// Same firm issued W-2 and 1099 for employee payments — code H
    /// The additional 1099-MISC/NEC pay should have been wages on that firm's W-2. Enter only the omitted amount here. Do not file Form SS-8 for this reason.
    SameFirmForms,
}

impl EmployeeWages {
    // The page's client rules, repeated on save (the server twins).

    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        if matches!(self.reason, EmployeeWageReason::SameFirmForms) && !self.form1099 {
            Some(
                crate::validation::ValidationIssue::new("FORM_8919_CODE_H_NEEDS_1099")
                    .at_field("form1099"),
            )
        } else if self.withholding > self.wages {
            Some(
                crate::validation::ValidationIssue::new("FORM_8919_WITHHOLDING_ABOVE_PAY")
                    .at_field("withholding"),
            )
        } else {
            None
        }
    }
    pub(crate) fn valid(&self) -> bool {
        (0..=100000000000).contains(&self.wages)
            && (0..=100000000000).contains(&self.withholding)
            && (!matches!(self.reason, EmployeeWageReason::SameFirmForms) || self.form1099)
    }
    // Pay remains earned income when ordinary coverage is No; the federal owner
    // sends it to line 1h and omits only the unimplemented payroll-tax part.
    pub(crate) fn income(&self) -> Result<i64, &'static str> {
        if !self.valid() {
            return Err(crate::fixture_english!(
                "Enter a valid Form 8919 reason and pay amounts; reason H needs the Form 1099 you received."
            ));
        }
        Ok(self.wages)
    }
}
