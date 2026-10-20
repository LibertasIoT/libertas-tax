// Coverage: Form 4835 — Supported (unit B2, 2026-10-09; 2026 drafts f4835 of 5/28/26 and i4835 of
//   9/21/26, Schedule E 2026 draft Part V, i8582 (2025), Pub 225 (2025)).
//   Lines: Part I 1 (crop and livestock shares converted to money, with the net of a redeemed CCC
//   crop's sale), 2a/2b (Form 1099-PATR routed by the document's chosen farm), 3a/3b (program
//   payments, Form 1099-G box 7 routed the same way, CCC market gain), 4a/4b/4c (CCC loans under
//   the section 77 election and the forfeited principal), 5a/5b/5d (crop insurance and disaster
//   payments), 6 (other income and CCC surplus), 7; Part II 8 (car and truck), 9–29 and 30a–30f
//   (the Farm page's expense lines relabeled to Form 4835 numbers, one row per line), 12
//   (depreciation, section 179 never), 19b (vehicle loan interest), 10 (section 175 conservation
//   from the one pool shared with Schedule F farms), 31, 32 (net, with prior-year unallowed losses
//   as "PAL"; blank for a loss, which is line 34c after Form 8582, F7), 34c (allowed loss after Form 8582), the suspended loss; Schedule E Part V line 40
//   (schedule_e_farm_rental_total) and line 42 (schedule_e_farming_fishing_gross); Schedule 1 line
//   5 through the passive pool (extended_rules.rs rentals_for: one PassiveEntry per rental with
//   rental_allowance = active participation, Form 8582 line 1 or 2); Form 8995 through the rental
//   QBI answer; the farmer estimated-tax test (underpayment.rs two_thirds_return_year) and the farming
//   NOL test (farm.rs check_nol). No self-employment tax, earned income or optional method
//   (section 1402(a)(1)). Land sales and asset sales reuse the Farm page records; their gain is
//   nonpassive property income (WARN_FARM_RENTAL_GAIN_NOT_PASSIVE when a loss is suspended).
//   The former whole-rental special switch is removed. Actual named source limits
//   preserve known rental receipts; detailed farm_specials records own their income,
//   election and mandatory-basis boundaries. Remaining independent risk policy:
//   an answered No without method facts leaves an operating loss out (line34b),
//   while supplied methods use the source's actual Form6198 owner. Section179
//   is zeroed: the property is depreciated). A real estate professional keeps the rental passive
//   (WARN_FARM_RENTAL_PASSIVE_ASSUMED). Refused by name: Form 1116 with a farm rental (line 3e),
//   section 175 with cooperative distributions on the same rental, and the shared records'
//   existing refusals. The rental's property is itemized on Form 4562 like a business's
//   (federal_rules.rs form_4562_asset_lines and form_4562_totals: lines 14a, 17, 19a-19j and 22).
//   Evidence: farm_rental.rs, farm_page.rs (FarmRental, farm_rental_record), extended_rules.rs,
//   farm.rs (FarmForm, sources), farm_ccc.rs, farm_insurance.rs, farm_conservation.rs,
//   debt.rs (crop_forfeiture); src/tests/farm/farm_rental.rs.
use crate::farm::FarmForm;
use crate::federal_rules::{farm_cooperative, farm_routed_income};
use crate::kernel::lines::push_line;
use crate::kernel::lines::{LineValue, emit as emit_lines, row as calculation_row};
use crate::kernel::ratio::{dollars, positive};
use crate::{
    BusinessAsset, BusinessMileage, FederalBusinessExpense, FederalDraft, FederalFarmRental,
    FederalIncome, FederalIncomeEntry, FederalResult, FormLine, Owner, TaxForm,
};
use alloc::format;

impl FederalFarmRental {
    pub(crate) fn valid(&self) -> bool {
        self.farm.valid()
            && matches!(self.farm.accounting, crate::FarmAccounting::Cash { .. })
            && self.farm.crp.is_empty()
            && self.farm.supplies.is_none()
            && crate::federal_rules::expenses_valid(&self.expenses)
            && self.expenses.iter().all(|e| EXPENSE_LINES[e.category.index()].is_some())
            && self.vehicles.len() <= 100
            && self.vehicles.iter().all(|vehicle| vehicle.valid())
            && self.assets.len() <= 100
            && self.assets.iter().all(BusinessAsset::valid)
            && self.payroll.valid()
            && crate::activity_risk::farm_rental_source_issue(&self.at_risk, &self.history)
                .is_none()
            && self.qbi_aggregation_group.as_ref().is_none_or(|group| !group.is_empty())
    }
    // Form 4835 line 7 in cents: the rental's own gross income plus the Form
    // 1099-G box 7 payments and taxable Form 1099-PATR distributions routed
    // to it.
    pub(crate) fn gross(
        &self,
        d: &FederalDraft,
        owner: usize,
        id: i64,
    ) -> Result<i64, crate::Section179PreparedIssue> {
        self.farm
            .gross()
            .checked_add(farm_routed_income(d, owner, id)?)
            .ok_or(crate::Section179PreparedIssue::AmountOverflow(id))
    }
    // Form 4835 line 12 in cents: depreciation, regular or alternative minimum
    // tax. Section 179 is never taken (it is zeroed before the rules run).
    pub(crate) fn depreciation(&self, mid_quarter: bool, amt: bool) -> i64 {
        self.assets.iter().map(|asset| asset.deduction_for(mid_quarter, amt)).sum()
    }
    // Form 4835 line 31 in cents.
    pub(crate) fn expenses_total(
        &self,
        d: &FederalDraft,
        id: i64,
        mid_quarter: bool,
        amt: bool,
    ) -> i64 {
        self.expenses_with_recovery(
            d,
            id,
            amt,
            self.depreciation(mid_quarter, amt),
            BusinessMileage::total(&self.vehicles),
        )
    }
    fn expenses_with_recovery(
        &self,
        d: &FederalDraft,
        id: i64,
        amt: bool,
        depreciation: i64,
        mileage: i64,
    ) -> i64 {
        crate::farm_conservation::rental_deduction(d, id, &self.farm, amt)
            + self.farm.ccc.as_ref().map_or(0, crate::FarmCcc::deficiency)
            + self.farm.specials.fertilizer_deduction()
            + self.farm.specials.production_deduction(true, amt)
            + FederalBusinessExpense::total(&self.expenses)
            + mileage
            + self
                .assets
                .iter()
                .map(|asset| asset.vehicle_operating() + asset.vehicle_loan_interest())
                .sum::<i64>()
            + depreciation
    }
    pub(crate) fn return_depreciation(
        &self,
        d: &FederalDraft,
        id: i64,
        mid_quarter: bool,
        amt: bool,
    ) -> Result<i64, crate::Section179PreparedIssue> {
        let whole = d.income.iter().find_map(|entry| match &entry.income {
            FederalIncome::FarmRental { data } if entry.id == id => Some(data),
            _ => None,
        });
        if let Some(whole) = whole {
            let amount =
                crate::cooperative_basis::depreciation(d, id, &whole.assets, mid_quarter, amt)?;
            if let Some(amount) = crate::community::farm_rental_amount(d, id, |_| amount) {
                return Ok(amount);
            }
        }
        crate::cooperative_basis::depreciation(d, id, &self.assets, mid_quarter, amt)
    }
    pub(crate) fn return_mileage(&self, d: &FederalDraft, id: i64) -> i64 {
        crate::community::farm_rental_amount(d, id, |whole| BusinessMileage::total(&whole.vehicles))
            .unwrap_or_else(|| BusinessMileage::total(&self.vehicles))
    }
    pub(crate) fn net_on_return(
        &self,
        d: &FederalDraft,
        owner: usize,
        id: i64,
        mid_quarter: bool,
        amt: bool,
    ) -> Result<i64, crate::Section179PreparedIssue> {
        let routed = crate::federal_rules::farm_routed_income_with_refunds(d, owner, id, amt)?;
        Ok(self.farm.gross() + crate::community::income_amount(d, id, routed)
            - self.expenses_with_recovery(
                d,
                id,
                amt,
                self.return_depreciation(d, id, mid_quarter, amt)?,
                self.return_mileage(d, id),
            ))
    }
    // Line 7 less line 31 in cents, before prior-year unallowed losses.
    pub(crate) fn net(
        &self,
        d: &FederalDraft,
        owner: usize,
        id: i64,
        mid_quarter: bool,
        amt: bool,
    ) -> Result<i64, crate::Section179PreparedIssue> {
        Ok(self.gross(d, owner, id)? - self.expenses_total(d, id, mid_quarter, amt))
    }
}

pub(crate) struct RiskSource {
    pub before_risk: i64,
    pub prior_risk: i64,
    pub prior_operating_allowed: Option<i64>,
    pub operating: i64,
    pub property: Option<crate::farm_land::LandGain>,
    pub property_rows: crate::asset_sale::FarmRentalPropertyRows,
}

// Form6198 uses the operating source's cents and the property owner's already
// recognized Form4797 dollars. The checked scale change is only the shared
// settlement unit; it does not reconstruct unrounded property recognition.
pub(crate) fn risk_source(
    d: &FederalDraft,
    id: i64,
    v: &FederalFarmRental,
    owner: usize,
    mid_quarter: bool,
    amt: bool,
    r: Option<&mut FederalResult>,
) -> Result<RiskSource, crate::activity_risk::SourceIssue> {
    use crate::activity_risk::{Part1Line, ProjectionIssue, SourceIssue};
    let issue = |error| SourceIssue::new(id, error);
    let arithmetic =
        || issue(ProjectionIssue::Arithmetic(crate::kernel::loss_limits::LossLimitError::Overflow));
    if let Some(source) = crate::activity_risk::farm_rental_source_issue(&v.at_risk, &v.history) {
        return Err(issue(ProjectionIssue::Source(source.key)));
    }
    let risk = v
        .at_risk
        .as_ref()
        .ok_or_else(|| issue(ProjectionIssue::Source("ACTIVITY_GROUP_INDIVIDUAL_RISK_REQUIRED")))?;
    let source = crate::asset_sale::farm_rental_property_source(d, id, amt)?;
    let mut property = source.value;
    let current_operating =
        v.net_on_return(d, owner, id, mid_quarter, amt).map_err(|source| match source {
            crate::Section179PreparedIssue::SourceBoundaryUnavailable(_, key) => {
                issue(ProjectionIssue::Source(key))
            }
            _ => arithmetic(),
        })?;
    let operating = current_operating.checked_sub(risk.prior(amt)).ok_or_else(arithmetic)?;
    let crate::RentalAtRisk::Limited { record: Some(record), .. } = risk else {
        // Answered No without method facts preserves the native profit path;
        // a genuine property/operating net loss still needs the missing method.
        if matches!(risk, crate::RentalAtRisk::Limited { record: None, .. }) {
            let net = crate::kernel::ratio::checked_sum([
                operating,
                property
                    .as_ref()
                    .map_or(0, |value| value.gain)
                    .checked_mul(100)
                    .ok_or_else(arithmetic)?,
                -risk.prior_sale(amt),
            ])
            .ok_or_else(arithmetic)?;
            if net < 0 {
                return Err(issue(ProjectionIssue::Source(
                    "ACTIVITY_GROUP_INDIVIDUAL_RISK_REQUIRED",
                )));
            }
        }
        if risk.prior_sale(amt) != 0 {
            let value = property.get_or_insert(crate::farm_land::LandGain::zero());
            value.section1231 = dollars(
                value
                    .section1231
                    .checked_mul(100)
                    .and_then(|amount| amount.checked_sub(risk.prior_sale(amt)))
                    .ok_or_else(arithmetic)?,
            );
            value.gain = value.ordinary.checked_add(value.section1231).ok_or_else(arithmetic)?;
        }
        return Ok(RiskSource {
            before_risk: operating,
            prior_risk: risk.prior(amt),
            prior_operating_allowed: Some(risk.prior(amt)),
            operating,
            property_rows: source
                .allowed_rows(
                    &property.unwrap_or_else(crate::farm_land::LandGain::zero),
                    dollars(risk.prior_sale(amt)),
                )
                .map_err(issue)?,
            property,
        });
    };
    let ordinary = property
        .as_ref()
        .map_or(0, |value| value.ordinary)
        .checked_mul(100)
        .ok_or_else(arithmetic)?;
    let section1231 = property
        .as_ref()
        .map_or(0, |value| value.section1231)
        .checked_mul(100)
        .and_then(|value| value.checked_sub(risk.prior_sale(amt)))
        .ok_or_else(arithmetic)?;
    let projection = crate::activity_risk::project_record(
        &[
            (Part1Line::Operating, operating),
            (Part1Line::OtherDisposition, ordinary),
            (Part1Line::Property, section1231),
        ],
        record,
        &crate::RiskEffectiveCategory::HoldingRealProperty,
        amt,
    )
    .map_err(issue)?;
    let prior_operating_allowed = crate::activity_risk::prior_operating_allowed(
        Some(current_operating),
        risk.prior(amt),
        projection.components.suspended[0],
    )
    .map_err(issue)?;
    let allowed = &projection.components.allowed;
    let before_risk = operating;
    let operating = allowed[0]
        .checked_add(projection.recapture.clone().map_err(issue)?)
        .ok_or_else(arithmetic)?;
    if property.is_some() || risk.prior_sale(amt) != 0 {
        let mut value = property.take().unwrap_or_else(crate::farm_land::LandGain::zero);
        value.ordinary = dollars(allowed[1]);
        value.section1231 = dollars(allowed[2]);
        value.gain = value.ordinary.checked_add(value.section1231).ok_or_else(arithmetic)?;
        // Signed settlement suspends only negative components. Positive native
        // recapture/unrecaptured amounts retain their actual source character.
        property = Some(value);
    }
    let property_rows = source
        .allowed_rows(
            &property.unwrap_or_else(crate::farm_land::LandGain::zero),
            dollars(risk.prior_sale(amt)),
        )
        .map_err(issue)?;
    if let Some(r) = r {
        crate::extended_rules::report_risk_projection(
            id,
            amt,
            "income.farm_rental.at_risk",
            projection,
            r,
        )
        .map_err(issue)?;
    }
    Ok(RiskSource {
        before_risk,
        prior_risk: risk.prior(amt),
        prior_operating_allowed,
        operating,
        property,
        property_rows,
    })
}

// Form 4835 Part II line of each `BusinessExpenseCategory` the Farm page can
// write, in its `index()` order: (return-line suffix, printed line, explanation).
// The Farm page's expense lines keep their Schedule F labels in the source;
// these are the Form 4835 numbers, two lower for lines 11–31 of Schedule F.
// Other expenses are lines 30a–30f, one row, untagged (which of the six lines
// is a layout choice). Categories the page never writes are None.
type ExpenseLine = (&'static str, Option<&'static str>, &'static str);
const EXPENSE_LINES: [Option<ExpenseLine>; crate::BusinessExpenseCategory::COUNT] = [
    None,
    None,
    None,
    Some((
        "line13_employee_benefit_programs",
        Some("13"),
        "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE13_EMPLOYEE_BENEFIT_PROGRAMS",
    )),
    Some(("line18_insurance", Some("18"), "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE18_INSURANCE")),
    Some((
        "line19a_mortgage_interest",
        Some("19a"),
        "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE19A_MORTGAGE_INTEREST",
    )),
    Some((
        "line19c_other_interest",
        Some("19c"),
        "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE19C_OTHER_INTEREST",
    )),
    None,
    None,
    Some((
        "line21_pension_and_profit_sharing",
        Some("21"),
        "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE21_PENSION_AND_PROFIT_SHARING",
    )),
    Some((
        "line22a_rent_vehicles_equipment",
        Some("22a"),
        "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE22A_RENT_VEHICLES_EQUIPMENT",
    )),
    Some(("line22b_rent_other", Some("22b"), "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE22B_RENT_OTHER")),
    Some(("line23_repairs", Some("23"), "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE23_REPAIRS")),
    Some(("line26_supplies", Some("26"), "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE26_SUPPLIES")),
    Some(("line27_taxes", Some("27"), "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE27_TAXES")),
    None,
    Some(("line30_meals", None, "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE30_MEALS")),
    Some(("line28_utilities", Some("28"), "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE28_UTILITIES")),
    Some(("line20_labor_hired", Some("20"), "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE20_LABOR_HIRED")),
    Some(("line30_other", None, "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE30_OTHER")),
    Some(("line9_chemicals", Some("9"), "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE9_CHEMICALS")),
    Some(("line11_custom_hire", Some("11"), "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE11_CUSTOM_HIRE")),
    Some(("line14_feed", Some("14"), "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE14_FEED")),
    Some((
        "line15_fertilizers_and_lime",
        Some("15"),
        "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE15_FERTILIZERS_AND_LIME",
    )),
    Some((
        "line16_freight_and_trucking",
        Some("16"),
        "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE16_FREIGHT_AND_TRUCKING",
    )),
    Some((
        "line17_gasoline_fuel_and_oil",
        Some("17"),
        "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE17_GASOLINE_FUEL_AND_OIL",
    )),
    Some((
        "line24_seeds_and_plants",
        Some("24"),
        "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE24_SEEDS_AND_PLANTS",
    )),
    Some((
        "line25_storage_and_warehousing",
        Some("25"),
        "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE25_STORAGE_AND_WAREHOUSING",
    )),
    Some((
        "line29_veterinary_breeding_and_medicine",
        Some("29"),
        "EXPLAIN_FARM_RENTAL_N_EXPENSE_LINE29_VETERINARY_BREEDING_AND_MEDICINE",
    )),
];

// One rental's Form 4835 lines 1–7, Part II and the notes and refusals of its
// shared records. Lines 32 and 34c come from the passive pool (extended_rules.rs).
pub(crate) fn report(
    d: &FederalDraft,
    e: &FederalIncomeEntry,
    v: &FederalFarmRental,
    r: &mut FederalResult,
) {
    let id = e.id;
    let owner = usize::from(e.owner == Owner::Spouse);
    let mq = crate::depreciation::mid_quarter(d);
    let farm = &v.farm;
    let death = crate::farm::owner_death_date(d, id);
    let farming = crate::IssueContext::Section(crate::TaxSection::Farming);
    let accepted = d.income.iter().find(|entry| entry.id == id).unwrap_or(e);
    let property = crate::community::property_entry(d, accepted);
    let FederalIncome::FarmRental { data: property } = &property.income else {
        unreachable!("same farm-rental source")
    };
    for (index, sale) in property.farm.land_sales.iter().enumerate() {
        crate::require_localized!(
            r,
            death.is_none_or(|date| sale.sold <= date),
            &farming,
            "Farm land final-return allocation",
            "ISSUE_A_SALE_AFTER_THIS_FARM_OWNER_S_DEATH_DOES_NOT_BELONG_TO_THE_OWNER_S_E146F4F3",
            "A sale after this farm owner's death does not belong to the owner's final individual return. Resolve the estate or beneficiary's ownership and reporting; a surviving spouse's own farm follows that spouse's dates.",
        );
        sale.report(d, &property.farm, id, index, FarmForm::Form4835, r);
    }
    if let Some(conservation) = &farm.conservation {
        conservation.report(farm.gross(), d, id, FarmForm::Form4835, r);
    }
    farm.insurance.report(true, id, FarmForm::Form4835, r);
    if let Some(ccc) = &farm.ccc {
        ccc.report(true, id, FarmForm::Form4835, r);
    }
    farm.specials.report(
        true,
        id,
        FarmForm::Form4835,
        crate::interview::income_issue_context(e),
        r,
    );
    farm.report_parcels(d, id, FarmForm::Form4835, r);
    farm.report_property_history(&v.assets, crate::interview::income_issue_context(e), r);
    let mut cooperative = match farm_cooperative(d, owner, id) {
        Ok(amounts) => amounts,
        Err(issue) => {
            crate::section179_return::Issue::Prepared(issue).report(r);
            return;
        }
    };
    cooperative.distributions = crate::community::income_amount(d, id, cooperative.distributions);
    cooperative.taxable = crate::community::income_amount(d, id, cooperative.taxable);
    cooperative.nonqualified = crate::community::income_amount(d, id, cooperative.nonqualified);
    cooperative.qualified_payments =
        crate::community::income_amount(d, id, cooperative.qualified_payments);
    crate::farm::require_farm_depreciation(r, &v.assets);
    let program_1099g = crate::community::income_amount(
        d,
        id,
        crate::federal_rules::farm_1099g_program(d, owner, id),
    );
    let (market, taxable_market, sale_net, surplus) =
        farm.ccc.as_ref().map_or((0, 0, 0, 0), crate::FarmCcc::cash_parts);
    let receipts = |pick: fn(&crate::FarmReceipt) -> bool| -> i64 {
        match &farm.accounting {
            crate::FarmAccounting::Cash { receipts } => receipts
                .iter()
                .filter(|receipt| pick(receipt))
                .map(|receipt| receipt.amount())
                .sum(),
            crate::FarmAccounting::Accrual { .. } => 0,
        }
    };
    let production = receipts(|r| matches!(r, crate::FarmReceipt::Raised { .. }));
    let program = receipts(|r| matches!(r, crate::FarmReceipt::Program { .. }));
    let other = receipts(|r| matches!(r, crate::FarmReceipt::Other { .. }));
    let routed = program_1099g + cooperative.taxable;
    let gross = farm.gross() + routed;
    let depreciation = match v.return_depreciation(d, id, mq, false) {
        Ok(amount) => amount,
        Err(issue) => {
            crate::section179_return::Issue::Prepared(issue).report(r);
            return;
        }
    };
    let expenses = v.expenses_with_recovery(d, id, false, depreciation, v.return_mileage(d, id));
    let path = |tail: &str| FarmForm::Form4835.input(tail);
    emit_lines(
        r,
        &mut (),
        [
            (
                "production",
                production + sale_net + farm.specials.weather_lines(true).iter().sum::<i64>(),
                Some("1"),
                msg!("EXPLAIN_FARM_RENTAL_N_PRODUCTION"),
            ),
            (
                "cooperative_distributions",
                cooperative.distributions,
                Some("2a"),
                msg!("EXPLAIN_FARM_RENTAL_N_COOPERATIVE_DISTRIBUTIONS"),
            ),
            (
                "cooperative_taxable",
                cooperative.taxable,
                Some("2b"),
                msg!("EXPLAIN_FARM_RENTAL_N_COOPERATIVE_TAXABLE"),
            ),
            (
                "program_payments",
                program + program_1099g + market + farm.specials.cost_sharing_gross(),
                Some("3a"),
                msg!("EXPLAIN_FARM_RENTAL_N_PROGRAM_PAYMENTS"),
            ),
            (
                "program_taxable",
                program + program_1099g + taxable_market + farm.specials.cost_sharing_income(),
                Some("3b"),
                msg!("EXPLAIN_FARM_RENTAL_N_PROGRAM_TAXABLE"),
            ),
            (
                "other_income",
                other + surplus + farm.specials.other_income(),
                Some("6"),
                msg!("EXPLAIN_FARM_RENTAL_N_OTHER_INCOME"),
            ),
            ("gross_income", gross, Some("7"), msg!("EXPLAIN_FARM_RENTAL_N_GROSS_INCOME")),
            ("depreciation", depreciation, Some("12"), msg!("EXPLAIN_FARM_RENTAL_N_DEPRECIATION")),
            ("expenses", expenses, Some("31"), msg!("EXPLAIN_FARM_RENTAL_N_EXPENSES")),
        ],
        |(name, value, line, note), _, _| {
            Some(calculation_row(
                line.map(|line| FormLine::new(TaxForm::Form4835, line)),
                &format!("farm_rental_{id}_{name}"),
                LineValue::MoneyCents(value),
                &[FarmForm::Form4835.root()],
                "https://www.irs.gov/instructions/i4835",
                note,
            ))
        },
    );
    // Line 8 holds the standard-mileage and actual-expense vehicles' costs
    // except their business share of vehicle-loan interest, which is the new
    // 2026 line 19b; the total is unchanged.
    if !v.vehicles.is_empty() || v.assets.iter().any(BusinessAsset::is_vehicle) {
        let loan_interest = BusinessMileage::loan_interest(&v.vehicles)
            + v.assets.iter().map(BusinessAsset::vehicle_loan_interest).sum::<i64>();
        push_line(
            r,
            Some(FormLine::new(TaxForm::Form4835, "8")),
            &format!("farm_rental_{id}_car_and_truck"),
            v.return_mileage(d, id) - BusinessMileage::loan_interest(&v.vehicles)
                + v.assets.iter().map(BusinessAsset::vehicle_operating).sum::<i64>(),
            &[&path("vehicles"), &path("assets")],
            "https://www.irs.gov/instructions/i4835",
            msg!("EXPLAIN_FARM_RENTAL_N_CAR_AND_TRUCK"),
        );
        if loan_interest != 0 {
            push_line(
                r,
                Some(FormLine::new(TaxForm::Form4835, "19b")),
                &format!("farm_rental_{id}_vehicle_loan_interest"),
                loan_interest,
                &[&path("vehicles"), &path("assets")],
                "https://www.irs.gov/instructions/i4835",
                msg!("EXPLAIN_FARM_RENTAL_N_VEHICLE_LOAN_INTEREST"),
            );
        }
    }
    // Part II lines from the stored expense categories, one row per printed
    // line; categories that share a line are added together.
    let mut totals = FederalBusinessExpense::category_totals(&v.expenses);
    totals[crate::BusinessExpenseCategory::Fertilizers.index()] +=
        farm.specials.fertilizer_deduction();
    for (category, amount) in crate::farm_production::category_totals(&farm.specials, true) {
        totals[category.index()] += amount;
    }
    let mut rows: alloc::vec::Vec<(&str, Option<&str>, &str, i64)> = alloc::vec::Vec::new();
    for (entry, amount) in EXPENSE_LINES.into_iter().zip(totals) {
        let Some((suffix, line, explanation)) = entry else {
            continue;
        };
        match rows.iter_mut().find(|row| row.0 == suffix) {
            Some(row) => row.3 += amount,
            None => rows.push((suffix, line, explanation, amount)),
        }
    }
    let capital = crate::farm_production::uniform_capital(&farm.specials, true);
    rows.push((
        "line30g_production_capitalized",
        Some("30g"),
        "EXPLAIN_FARM_PRODUCTION_CAPITALIZED",
        -capital,
    ));
    for (suffix, line, explanation, amount) in rows {
        if amount != 0 {
            push_line(
                r,
                line.map(|line| FormLine::new(TaxForm::Form4835, line)),
                &format!("farm_rental_{id}_expense_{suffix}"),
                amount,
                &[&path("expenses")],
                "https://www.irs.gov/instructions/i4835",
                crate::Message::new(explanation, alloc::vec![]),
            );
        }
    }
}

// The rental's Form 8995 activity: the allowed net, its payroll W-2 wages and
// the unadjusted basis of the depreciable property still within its
// depreciable period (land is never depreciable property). `active` follows
// the section 469(h) answer, the section 199A(i) minimum-deduction test.
pub(crate) fn qbi_activity(
    v: &FederalFarmRental,
    income: i64,
) -> Option<crate::business_rules::QbiActivity> {
    let crate::RentalBusiness::Yes { material_participation } = &v.qbi_business else {
        return None;
    };
    let property = v
        .assets
        .iter()
        .filter(|asset| asset.qbi_property())
        .map(BusinessAsset::business_basis)
        .sum();
    Some(crate::business_rules::QbiActivity {
        source_wages_cents: None,
        source_property_cents: None,
        source_income_cents: None,
        owner: None,
        income,
        wages: dollars(v.payroll.qbi_wages()),
        property: dollars(property),
        active: *material_participation,
        service: Some(false),
        group: v.qbi_aggregation_group.clone(),
    })
}

// Lines 32 and 34c of Form 4835 and the notes of the pool's result for one
// rental. `net` is the rental's amount after prior-year unallowed losses (the
// Form 8582 entry), `allowed_net` the income or negative allowed loss that
// reaches Schedule E line 40.
pub(crate) fn report_pool(
    d: &FederalDraft,
    r: &mut FederalResult,
    id: i64,
    net: i64,
    allowed_loss: i64,
    suspended: i64,
    real_estate_professional: bool,
) {
    let Some(entry) = d.income.iter().find(|entry| entry.id == id) else {
        return;
    };
    let crate::FederalIncome::FarmRental { .. } = &entry.income else {
        return;
    };
    // Form 4835 line 32 (2026 draft and instructions): "If line 31 is more than
    // line 7, don't enter your loss on line 32"; a loss goes through the at-risk
    // and passive activity loss rules to line 34c. A rental that is income only
    // before its prior-year losses prints "PAL" as zero; its loss is the
    // Form 8582 entry and line 34c.
    let line_32 = positive(net);
    let path = FarmForm::Form4835.root();
    crate::kernel::lines::form_line(
        r,
        FormLine::new(TaxForm::Form4835, "32"),
        &format!("farm_rental_{id}_net_income"),
        line_32,
        &[path, "rental_magi"],
        "https://www.irs.gov/instructions/i4835",
        msg!("EXPLAIN_FARM_RENTAL_N_NET_INCOME"),
    );
    if net < 0 {
        crate::kernel::lines::form_line(
            r,
            FormLine::new(TaxForm::Form4835, "34c"),
            &format!("farm_rental_{id}_line34c_allowed_loss"),
            -allowed_loss,
            &[path, "rental_magi"],
            "https://www.irs.gov/publications/p925",
            msg!("EXPLAIN_FARM_RENTAL_N_LINE34C_ALLOWED_LOSS"),
        );
    }
    if suspended > 0 {
        let context = crate::interview::income_issue_context(entry);
        let name = crate::interview::income_display_arg(entry);
        if real_estate_professional {
            crate::federal_rules::note_once_scoped(
                r,
                crate::Message::new("WARN_FARM_RENTAL_PASSIVE_ASSUMED", alloc::vec![name.clone()]),
                context.clone(),
            );
        }
        let gain = crate::asset_sale::direct_sources(d, false)
            .into_iter()
            .any(|(source, value)| source == id && value.gain != 0);
        if gain {
            crate::federal_rules::note_once_scoped(
                r,
                crate::Message::new("WARN_FARM_RENTAL_GAIN_NOT_PASSIVE", alloc::vec![name]),
                context,
            );
        }
    }
}

// The taxable Form 1099-PATR distributions routed to a rental that its box 8
// qualified items do not cover, in whole dollars: not qualified business income.
pub(crate) fn nonqualified_cooperative(
    d: &FederalDraft,
    id: i64,
) -> Result<i64, crate::Section179PreparedIssue> {
    let entry = d
        .income
        .iter()
        .find(|entry| entry.id == id)
        .ok_or(crate::Section179PreparedIssue::UnknownSource(id))?;
    Ok(dollars(farm_cooperative(d, usize::from(entry.owner == Owner::Spouse), id)?.nonqualified))
}
