// Charitable gift and carryover editor models.
#[allow(unused_imports)]
use super::*;
pub use crate::charity_sources::{GoodsKind, VehicleUse};
/// Gifts of property and carryovers
/// Property and stock contributions, appraisals, Form 8283 Sections A and B, Form 1098-C, conservation easements and charitable carryover worksheets. Cash gifts belong on Deductions.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
// Numeric Enumeration indexes: GoodsKind clothing0/household1, condition NotGood1,
// CharityClass CapitalGain30=3; only the enclosing GiftKind uses a Union `is` test.
#[libertas_calculation_root("RetainedCharityAppreciationOptions")]
pub struct GiftCarryovers {
    /// Gifts of property
    /// One row for each contribution of goods, a vehicle, shares, other property or a conservation easement.
    #[libertas_unordered]
    /// ----
    /// Gift
    pub gifts: Vec<crate::PropertyGiftPage>,
    /// Vehicles, boats and airplanes
    /// One row for each Form 1098-C: a car, truck, boat or airplane you gave. The form's boxes are the ones the charity printed.
    #[libertas_unordered]
    /// ----
    /// Vehicle gift
    pub vehicles: Vec<crate::VehicleGiftPage>,
    /// Appraisals
    /// Required for Section B gifts and a single poor-condition clothing or household item worth over $500. One report can cover several gifts. Save an appraisal before selecting it on a gift.
    #[libertas_unordered]
    /// ----
    /// Appraisal
    pub appraisals: Vec<crate::Appraisal>,
    /// Appraisals on this return
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_copy_from("$.appraisal_choices")]
    #[libertas_output_only]
    pub appraisal_choices: Vec<RecordChoice>,
    /// Reduce all 2026 capital-gain gifts to public charities to their cost
    /// The section 170(b)(1)(C)(iii) election substitutes basis for value under the 50% class and refigures prior 30% capital-gain carryovers. Usually helpful only when appreciation is small.
    #[libertas_calculation_request("RetainedCharityAppreciationOptions")]
    pub basis_election: bool,
    /// Carryovers from earlier years
    /// One reconciled balance for each original year and class; conservation keeps its original 50% or 100% limit and fifteen-year period.
    #[libertas_unordered]
    #[libertas_calculation_request("RetainedCharityAppreciationOptions")]
    /// ----
    /// Carryover
    pub carryovers: Vec<crate::CharityCarryoverPage>,
    /// Out-of-pocket costs of volunteering
    /// Unreimbursed costs paid while giving services to a U.S. public charity or government. Your time is not deductible.
    #[libertas_unordered]
    /// ----
    /// Volunteer cost
    pub volunteer: Vec<crate::VolunteerExpense>,
    /// Less common gifts
    /// A student you hosted or a whaling captain's expenses.
    #[libertas_unordered]
    /// ----
    /// Gift
    pub less_common: Vec<LessCommonGift>,
}
/// Gift or carryover
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum LessCommonGift {
    /// A student you hosted under an education program (section 170(g))
    /// Up to $50 for each month the student lived with you at least 15 days.
    HostedStudent(crate::HostedStudent),
    /// Expenses of a recognized whaling captain (section 170(n))
    WhalingCaptain(WhalingCaptain),
}
/// Whaling captain
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct WhalingCaptain {
    /// Captain
    #[libertas_constrained_by("$.allowed_owners")]
    pub owner: Owner,
    /// AEWC captain recognition document reference
    #[libertas_size(min = 1)]
    pub captain_record_reference: String,
    /// Copies of the 2026 activity reports submitted to the AEWC
    #[libertas_size(min = 1)]
    pub aewc_report_reference: String,
    /// Log of each sanctioned trip's departure, return and area
    #[libertas_size(min = 1)]
    pub trip_log_reference: String,
    /// 2026 reasonable and necessary whaling expenses
    #[libertas_size(min = 1)]
    /// ----
    /// Expense
    pub expenses: Vec<crate::WhalingExpense>,
    /// AEWC recognition, 2026 sanctioned activity, complete logs, unreimbursed costs and no deduction elsewhere confirmed
    /// Check your AEWC recognition, submitted reports, trip logs and payment records to answer. No leaves these expenses undeducted.
    pub eligibility_and_records_confirmed: bool,
}
