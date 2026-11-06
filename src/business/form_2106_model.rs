// form_2106 items of src/federal/federal_model.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

// The Form 2106 topic of the adjustments hub: one row per occupation and
// capacity. Its stored counterpart is EmployeeExpenseClaim (employee_expenses.rs);
// the vehicles reuse the Schedule C page type and its stored mileage and asset
// records. `wages` is the read-only choice list of the return's W-2s, keyed by
// the income record identity the claim stores.
/// Reservist, performing artist and fee-basis official expenses
/// Form 2106. Only these employees may deduct job expenses in 2026; the deduction goes on Schedule 1 line 12 whether or not you itemize. Enter one form for each job. Impairment-related work expenses belong on Investment interest and other deductions.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EmployeeExpenses {
    /// Forms 2106
    #[libertas_unordered]
    /// ----
    /// Form 2106
    pub forms: Vec<EmployeeExpenseForm>,
    /// W-2 employers on this return
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub wages: Vec<WageChoice>,
}
/// W-2 employer
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct WageChoice {
    /// Income record identity
    #[libertas_key]
    #[libertas_unique(1)]
    #[libertas_hidden]
    pub id: i64,
    /// Employer
    #[libertas_formatted_text]
    #[libertas_read_only]
    pub label: Vec<u8>,
    /// Owner
    #[libertas_hidden]
    #[libertas_read_only]
    pub owner: Owner,
}
/// Form 2106
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EmployeeExpenseForm {
    /// Physical buildings containing separately recorded improvements
    /// Enter each building once; its accepted identity is offered in the improvement picker.
    pub improvement_buildings: Vec<crate::ImprovementBuildingFacts>,
    /// Subject to Department of Transportation hours-of-service limits
    /// Qualifying meals are 80% deductible rather than 50%.
    #[libertas_default(false)]
    pub dot_hours_of_service: bool,
    /// Whose form
    #[libertas_default(Taxpayer)]
    #[libertas_constrained_by("$.allowed_owners")]
    pub owner: Owner,
    /// Occupation in which you incurred these expenses
    #[libertas_size(min = 1)]
    pub occupation: String,
    /// Who you were in this job
    pub capacity: crate::EmployeeCapacity,
    /// Employers for this job
    /// Choose the W-2s of this job. Enter a missing W-2 on Wages first. Their box 12 code L reimbursements reduce these expenses.
    #[libertas_enum_source("^<EmployeeExpenses>.wages")]
    #[libertas_size(min = 1)]
    pub employers: Vec<i64>,
    /// Parking, tolls and local transportation (line 2)
    /// Trains, buses, taxis and parking that did not involve overnight travel. Not commuting between home and work. Parking and tolls of a standard-mileage vehicle are entered with that vehicle.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub local_transportation: i64,
    /// Travel away from home overnight (line 3)
    /// Lodging, airfare, car rental and similar costs of overnight trips away from your tax home. Do not include meals. Keep receipts for all lodging. Temporary jobs of more than one year do not qualify.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub travel: i64,
    /// Other job expenses (line 4)
    /// Business gifts, education, trade publications, union dues, supplies and equipment used in this job. Do not include meals, taxes, interest or educator expenses deducted on Educator expenses.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub other: i64,
    /// Meals (line 5)
    /// Meals while away from home overnight and other business meals; enter the full cost or the standard meal allowance (GSA.gov/perdiem). The App deducts 50%. Entertainment is not deductible.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub meals: i64,
    /// Reimbursements not in box 1 and not under code L (line 7)
    /// Amounts your employer or a third party paid you for these expenses that are not in W-2 box 1 and were not reported under box 12 code L. Code L amounts are read from the W-2s you chose above.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub reimbursements: i64,
    /// Vehicles
    /// A car, van, pickup or panel truck used in this job. Car-loan interest and personal property tax are not deductible on Form 2106; leave them at zero (property tax belongs on Taxes you paid). Keep a mileage log with dates, miles and business purpose. Section 179 is not calculated for an employee vehicle.
    #[libertas_unordered]
    /// ----
    /// Vehicle
    pub vehicles: Vec<Vehicle>,
    /// Equipment used in this job
    /// Enter the actual property's cost, use and original recovery records here.
    /// Do not also include its depreciation in other job expenses or another activity.
    #[libertas_unordered]
    /// ----
    /// Asset
    pub equipment_assets: Vec<Asset>,
}
