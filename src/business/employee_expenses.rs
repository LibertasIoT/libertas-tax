// Coverage: Form 2106 — Supported (audited 2026-10-09 against the 2026 draft f2106 created 8/18/26
//   and the 2026 draft i2106 proof 28-Aug-2026; the final revision must be reread).
//   Filers: an Armed Forces reservist, a qualified performing artist and a fee-basis state or local
//   government official (26 U.S.C. 62(a)(2)(B), (C), (E)); every other employee's expenses are
//   nondeductible after 2017 (section 67(h) as amended by P.L. 119-21 section 70110) and the App
//   never offers them. Impairment-related work expenses (Schedule A line 17j) belong to the
//   itemized "Investment interest and other deductions" page, not here; an eligible educator's
//   expenses stay on the Educator page. A person with two capacities enters two forms.
//   Lines: Part I 1 (standard mileage and Part II line 29), 2, 3, 4, 5, 6A/6B, 7 (W-2 box 12 code L
//   of the chosen W-2s plus the other reimbursements entered, split by the Reimbursement Allocation
//   Worksheet: line 5 / (6A + 6B) to three decimals), 8, 9 (50%, or DOT80%, of column B), 10
//   to Schedule 1 line 12 (return line schedule_1_employee_business_expenses). A line 8 excess in
//   column A is wage income on Form 1040 line 1a (employee_expenses_N_excess_reimbursement; a
//   Corrective note toward Wages). Part II: 12-14, 16, 22 (standard mileage 72.5 cents January-June,
//   76 cents July-December, Notice 2026-10 and Announcement 2026-11, through BusinessMileage), 23,
//   24a-24c (business lease payments less inclusion, floored at zero),
//   26-29 (business-mile share of operating costs) and 30-38 (cost, 100% bonus or regular MACRS,
//   section 280F limit by placed-in-service year, year_constants.rs) through the Schedule C
//   BusinessAsset machinery; the vehicles join the return-wide mid-quarter test and bonus
//   consistency (depreciation.rs assets).
//   Performing-artist tests (section 62(b)(1)-(3)), per form: at least two chosen W-2s paying $200
//   or more (WARN_PERFORMING_ARTIST_EMPLOYERS), line 10 above 10% of the chosen W-2s' box 1
//   (WARN_PERFORMING_ARTIST_TEN_PERCENT), AGI before this deduction, with every other adjustment
//   but without any artist's line 12, of $16,000 or less, combined on a joint return
//   (WARN_PERFORMING_ARTIST_AGI), and a married filing separately return only when the spouses lived
//   apart all year (WARN_PERFORMING_ARTIST_SEPARATE_RETURN). A failed test leaves that form out of
//   line 12 with one non-blocking note per failed test; the estimate proceeds. The reservist and
//   fee-basis deductions are figured with the other early adjustments, so the phase-outs that read
//   AGI see them; the artist deduction is figured last, after the AGI test.
//   Excluded or left out, each stated on its page or by a note: the reservist per diem caps (the
//   person applies the GSA rates, no locality table is held); rural mail carriers (section 162(o));
//   Unit11 supports DOT hours-of-service meals at80% on line9 and actual leases using the
//   Schedule C 2018-2026 inclusion tables; tests/schedule_c_specials covers both and zero payments.
//   Part II lines 15 and 18-21 (informational, no computation; the Vehicle page type has no
//   attestation field), and line 25
//   employer-provided vehicle value; a
//   mid-year personal-to-business conversion; section 179 on an employee vehicle
//   (WARN_EMPLOYEE_VEHICLE_179_LEFT_OUT: the elected amount is left out and the vehicle takes its
//   bonus or regular depreciation); a sale or stopped business use of an employee vehicle (unit
//   B11 4797: the business asset's Form 4797 rows under employee_expenses_<n>_vehicle_<i>_
//   disposal_*, the Part IV column (b) excess depreciation on Schedule 1 line 8z, no
//   self-employment tax and no QBI); a 150% declining-balance history
//   (the shared vehicle_supported refusal); car-loan interest and personal property tax (not
//   deductible by an employee, Pub 463; a page rule with its server twin and a stored-data
//   validity condition).
//   Tests: src/tests/cross/returns.rs F1-F3 (reservist-drill-travel, performing-artist-agi-boundary,
//   fee-basis-official-excess-reimbursement), src/tests/business/business.rs
//   employee_vehicle_actual_expenses_apply_the_2026_280f_table_in_line_1 and the 179, sale and
//   reservist cases, src/tests/business/interview.rs w2_box12_code_l_*, employee_expenses_*; sources 2026 draft
//   f2106 and i2106, Pub 463, 26 U.S.C. 62, 67, 162(p), 274(n), 280F, Notice 2026-10.
use crate::kernel::lines::{LineValue, emit as emit_lines, row as calculation_row};
use crate::kernel::ratio::{dollars, fraction, positive};
use crate::{
    BusinessAsset, BusinessMileage, EquipmentHistory, FederalDraft, FederalIncome, FederalResult,
    FormLine, IssueContext, Message, MessageArg, Owner, Section179, TaxForm, TaxSection,
};
use alloc::{borrow::Cow, string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Who you were in this job
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum EmployeeCapacity {
    /// Armed Forces reservist
    /// Army, Navy, Marine Corps, Air Force or Coast Guard Reserve, Army or Air National Guard, or the Public Health Service Reserve Corps. Enter only expenses of duty travel more than 100 miles from home with an overnight stay: lodging within the federal lodging rate and meals within the federal M&IE rate for the locality (GSA.gov/perdiem), transportation, parking, ferries and tolls, and vehicles at the standard mileage rate. Other reserve expenses are not deductible.
    Reservist,
    /// Qualified performing artist
    /// You performed services in the performing arts as an employee for at least two employers, at least two of them paying you $200 or more, your performing-arts expenses exceed 10% of your performing-arts wages, and your adjusted gross income before these expenses is $16,000 or less (a joint return uses your combined income). If you are married you must file jointly unless you lived apart from your spouse all year. The App checks these from your W-2s and this return; enter only performing-arts expenses.
    PerformingArtist,
    /// Fee-basis state or local government official
    /// You were employed by a state or a political subdivision of a state and paid in whole or in part on a fee basis. Enter only expenses of that office.
    FeeBasisOfficial,
}

/// Form 2106 record
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EmployeeExpenseClaim {
    /// DOT hours-of-service meals
    #[libertas_default(false)]
    pub dot_hours_of_service: bool,
    /// Leased actual-expense vehicles
    pub leased_vehicles: Vec<crate::BusinessLeasedVehicle>,
    /// Whose form
    pub owner: Owner,
    /// Occupation
    #[libertas_size(min = 1)]
    pub occupation: String,
    /// Capacity
    pub capacity: EmployeeCapacity,
    /// Chosen W-2 income record identities
    #[libertas_size(min = 1)]
    pub employers: Vec<i64>,
    /// Line 2 local transportation
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub local_transportation: i64,
    /// Line 3 travel
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub travel: i64,
    /// Line 4 other
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub other: i64,
    /// Line 5 meals
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub meals: i64,
    /// Line 7 reimbursements other than code L
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub reimbursements: i64,
    /// Standard-mileage vehicles
    pub mileage: Vec<BusinessMileage>,
    /// Actual-expense vehicles
    pub vehicle_assets: Vec<BusinessAsset>,
}

// The figures of one Form 2106, in cents.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Lines {
    pub line1: i64,
    pub line2: i64,
    pub line3: i64,
    pub line4: i64,
    pub line5: i64,
    // Line 7 in total: code L plus the other reimbursements.
    pub line7: i64,
    pub line8a: i64,
    pub line8b: i64,
    // Line 8's column A excess, wage income.
    pub excess: i64,
    pub line9: i64,
    pub line10: i64,
    // Parts of line 1 for the vehicle rows: line 27 and line 28 per actual vehicle.
    pub operating: Vec<i64>,
    pub depreciation: Vec<i64>,
    // Box 1 of the chosen W-2s (the artist's performing-arts gross income).
    pub gross: i64,
    // The chosen W-2s paying $200 or more.
    pub paying_200: usize,
    // Chosen W-2s that are no longer a W-2 of this owner on the return.
    pub missing: usize,
}

impl EmployeeExpenseClaim {
    pub(crate) fn valid(&self) -> bool {
        self.admission_issue().is_none()
    }

    pub(crate) fn admission_issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        let key = "MESSAGE_CHECK_THE_FORM_2106_AMOUNTS_AND_VEHICLES";
        let amounts =
            [self.local_transportation, self.travel, self.other, self.meals, self.reimbursements];
        let records_valid = !self.occupation.trim().is_empty()
            && (1..=10).contains(&self.employers.len())
            && self
                .employers
                .iter()
                .enumerate()
                .all(|(i, id)| !self.employers[..i].contains(id))
            && amounts.iter().all(|v| (0..=100000000000).contains(v))
            && self.mileage.len() + self.vehicle_assets.len() + self.leased_vehicles.len() <= 10
            && self.leased_vehicles.iter().all(crate::BusinessLeasedVehicle::valid)
            // Car-loan interest and personal property tax are not an employee's
            // deductions (Pub. 463); a reservist uses only the standard mileage rate.
            && self
                .mileage
                .iter()
                .all(|m| m.valid() && m.interest == 0 && m.property_tax == 0)
            && self.vehicle_assets.iter().all(|a| {
                a.is_vehicle()
                    && a.valid()
                    && a.vehicle_loan_interest_entered() == 0
            });
        if !records_valid {
            return Some(ValidationIssue::new(key).at_record());
        }
        if self.capacity == EmployeeCapacity::Reservist
            && !(self.vehicle_assets.is_empty() && self.leased_vehicles.is_empty())
        {
            return Some(ValidationIssue::new(key).at_field(if self.vehicle_assets.is_empty() {
                "leased_vehicles"
            } else {
                "vehicle_assets"
            }));
        }
        None
    }

    // The actual-expense vehicles as the rules figure them: section 179 is left
    // out of an employee vehicle (it is not part of the return-wide allocation),
    // so a vehicle never claims it. Whether an elected amount was dropped is
    // the second part.
    fn effective_assets(&self) -> Vec<(BusinessAsset, bool)> {
        self.vehicle_assets
            .iter()
            .map(|asset| {
                let mut asset = asset.clone();
                let mut dropped = false;
                if let EquipmentHistory::New { section179, bonus, tables } = asset.history
                    && matches!(section179, Section179::Eligible { .. })
                {
                    dropped = section179.elected() > 0;
                    asset.history =
                        EquipmentHistory::New { section179: Section179::Ineligible, bonus, tables };
                }
                (asset, dropped)
            })
            .collect()
    }

    pub(crate) fn assets(&self) -> Vec<BusinessAsset> {
        self.effective_assets().into_iter().map(|(asset, _)| asset).collect()
    }

    // The W-2 of a chosen income record identity, when it is still this owner's.
    fn wage<'a>(&self, d: &'a FederalDraft, id: i64) -> Option<&'a crate::FederalWage> {
        d.income.iter().find_map(|entry| match &entry.income {
            FederalIncome::Wage { data } if entry.id == id && entry.owner == self.owner => {
                Some(data)
            }
            _ => None,
        })
    }

    pub(crate) fn lines(&self, d: &FederalDraft, mid_quarter: bool) -> Lines {
        let assets = self.assets();
        let mut out = Lines::default();
        for asset in &assets {
            let operating = asset.vehicle_operating();
            let depreciation = asset.deduction(mid_quarter);
            out.operating.push(operating);
            out.depreciation.push(depreciation);
        }
        // Part II line 22 (standard mileage) and line 29 (actual expenses).
        out.line1 = fraction(
            self.mileage.iter().map(BusinessMileage::mileage_cost_twice).sum::<i64>(),
            1,
            2,
        ) + out.operating.iter().sum::<i64>()
            + out.depreciation.iter().sum::<i64>()
            + self.leased_vehicles.iter().map(|v| v.amounts().total()).sum::<i64>();
        out.line2 =
            self.local_transportation + self.mileage.iter().map(|m| m.parking_tolls).sum::<i64>();
        out.line3 = self.travel;
        out.line4 = self.other;
        out.line5 = self.meals;
        let mut code_l = 0;
        for id in &self.employers {
            match self.wage(d, *id) {
                Some(wage) => {
                    code_l += wage
                        .other_box12
                        .iter()
                        .filter(|c| c.code == crate::W2Code::L)
                        .map(|c| c.amount)
                        .sum::<i64>();
                    out.gross += wage.wages;
                    if wage.wages >= 20000 {
                        out.paying_200 += 1;
                    }
                }
                None => out.missing += 1,
            }
        }
        let line6a = out.line1 + out.line2 + out.line3 + out.line4;
        let line6b = out.line5;
        out.line7 = code_l + self.reimbursements;
        // The Reimbursement Allocation Worksheet: the meals share of 6, to three
        // decimals, of the reimbursements goes to column B.
        let ratio = if line6a + line6b == 0 { 0 } else { fraction(line6b, 1000, line6a + line6b) };
        let line7b = fraction(out.line7, ratio, 1000);
        let line7a = out.line7 - line7b;
        out.line8a = positive(line6a - line7a);
        out.line8b = positive(line6b - line7b);
        out.excess = positive(line7a - line6a);
        out.line9 = if self.dot_hours_of_service {
            fraction(out.line8b, 80, 100)
        } else {
            fraction(out.line8b, 1, 2)
        };
        out.line10 = out.line8a + out.line9;
        out
    }

    // The context every note and issue of this form names.
    pub(crate) fn context(&self, position: usize) -> IssueContext {
        IssueContext::owner(TaxSection::EmployeeExpenses, self.owner)
            .item("employee_expenses", position)
            .named(&self.occupation)
    }
}

// One form's figures once per estimate.
pub(crate) struct Form {
    pub position: usize,
    pub claim: EmployeeExpenseClaim,
    pub lines: Lines,
}

pub(crate) struct Calculation {
    pub forms: Vec<Form>,
}

// The forms' lines, figured before the return's income (the excess
// reimbursement is wage income) and checked for the scope the rules support.
pub(crate) fn calculate(d: &FederalDraft, r: &mut FederalResult) -> Calculation {
    let Some(adjustments) = d.adjustments.as_ref() else {
        return Calculation { forms: Vec::new() };
    };
    let mid_quarter = crate::depreciation::mid_quarter(d);
    let mut forms = Vec::new();
    for (position, claim) in adjustments.employee_expenses.iter().enumerate() {
        let lines = claim.lines(d, mid_quarter);
        let context = claim.context(position);
        crate::require_localized!(
            r,
            lines.missing == 0,
            &context,
            "Form 2106",
            "ISSUE_EMPLOYEE_EXPENSES_EMPLOYER_MISSING",
            "A W-2 chosen for this Form 2106 is no longer on the return. Choose this job's W-2 again.",
        );
        for (index, (asset, dropped)) in claim.effective_assets().iter().enumerate() {
            if *dropped {
                crate::federal_rules::note_once_scoped(
                    r,
                    Message::new(
                        "WARN_EMPLOYEE_VEHICLE_179_LEFT_OUT",
                        alloc::vec![MessageArg::Text(claim.occupation.clone())],
                    ),
                    context.item("vehicle_assets", index),
                );
            }
            crate::require_localized!(
                r,
                asset.vehicle_supported() && asset.qualifies(),
                &context.item("vehicle_assets", index),
                "Form 2106 vehicle",
                "ISSUE_EMPLOYEE_VEHICLE_ACTUAL_EXPENSES_SCOPE",
                "Form 2106 actual vehicle expenses need an owned 5-year passenger automobile placed in service in 2025 or 2026, or a vehicle outside the passenger-automobile limit, with the IRS table percentages and no section 179 carryforward. Earlier passenger automobiles, a 150% declining-balance history, an employer-provided vehicle and a personal-use conversion need separate rules (Form 2106 Part II).",
            );
        }
        // A sale, condemnation or conversion to personal use of an actual-expense
        // vehicle takes the same Form 4797 rows as a business asset (Pub 463): the
        // gain is no self-employment income and no qualified business income, and
        // the Part IV recapture is Schedule 1 line 8z income.
        let assets = claim.assets();
        for index in 0..assets.len() {
            crate::asset_sale::report_business_asset(
                r,
                #[cfg(test)]
                "Form 2106 vehicle",
                &context.item("vehicle_assets", index),
                &alloc::format!("employee_expenses_{}_vehicle_{}", position + 1, index + 1),
                index,
                &assets,
                claim.mileage.len(),
                assets[index].sale_for(mid_quarter, false),
                assets[index].partial_for(),
                "income.business.assets",
            );
        }
        forms.push(Form { position, claim: claim.clone(), lines });
    }
    Calculation { forms }
}

impl Calculation {
    // The excess depreciation (Form 4797 Part IV column (b)) of the actual-expense
    // vehicles whose business use fell to 50% or less, in cents: Schedule 1 line
    // 8z income, the schedule the Form 2106 deduction reached (i4797 Part IV).
    // The same native asset recapture, retaining its actual Form2106 owner
    // before the ordinary joint subtotal is read. No fabricated income ID.
    pub(crate) fn recapture_by_owner(&self) -> [i64; 2] {
        let mut owners = [0; 2];
        for form in &self.forms {
            let owner = usize::from(form.claim.owner == Owner::Spouse);
            for recapture in form.claim.assets().iter().filter_map(|asset| asset.recapture()) {
                owners[owner] += recapture.excess();
            }
        }
        owners
    }

    // Line 8's column A excess of each owner, in cents: wage income.
    pub(crate) fn excess(&self) -> [i64; 2] {
        let mut excess = [0; 2];
        for form in &self.forms {
            excess[usize::from(form.claim.owner == Owner::Spouse)] += form.lines.excess;
        }
        excess
    }

    // Actual positive line8 excess from reimbursements outside W-2 box1
    // and codeL. Reuse this invocation's already calculated line7/8; no
    // second expense worksheet or inferred share of mixed reimbursements.
    pub(crate) fn non_w2_excess_cents(&self) -> Option<[i64; 2]> {
        self.forms.iter().try_fold([0; 2], |mut owners, form| {
            if form.lines.excess == 0 || form.claim.reimbursements == 0 {
                return Some(owners);
            }
            if form.lines.line7 != form.claim.reimbursements {
                // Both actual codeL and other reimbursements contributed.
                // Their combined excess has no source split in this owner.
                return None;
            }
            owners[usize::from(form.claim.owner == Owner::Spouse)] += form.lines.excess;
            Some(owners)
        })
    }

    // The reservist and fee-basis deductions, whole dollars: figured with the
    // return's other early adjustments.
    pub(crate) fn ordinary_deduction(&self) -> i64 {
        self.forms
            .iter()
            .filter(|f| f.claim.capacity != EmployeeCapacity::PerformingArtist)
            .map(|f| dollars(f.lines.line10))
            .sum()
    }

    // The same calculated ordinary Form2106 line10 amounts before filing
    // rounding. Pub974 Iterative work consumes these existing owner cents.
    pub(crate) fn ordinary_deduction_cents(&self) -> i64 {
        self.forms
            .iter()
            .filter(|f| f.claim.capacity != EmployeeCapacity::PerformingArtist)
            .map(|f| f.lines.line10)
            .sum()
    }

    // The Schedule 1 line 12 deduction of each filer, whole dollars (Form 8379
    // line 14 allocates it to the filer whose Form 2106 it is).
    pub(crate) fn deduction_by_owner(&self, counted: &[usize]) -> [i64; 2] {
        let mut by_owner = [0; 2];
        for f in &self.forms {
            if f.claim.capacity == EmployeeCapacity::PerformingArtist
                && !counted.contains(&f.position)
            {
                continue;
            }
            by_owner[usize::from(f.claim.owner == Owner::Spouse)] += dollars(f.lines.line10);
        }
        by_owner
    }

    // The difference of regular and AMT depreciation of the employee vehicles,
    // as a business's assets make it (positive means higher AMT income).
    pub(crate) fn amt_adjustment(&self, d: &FederalDraft) -> i64 {
        let mid_quarter = crate::depreciation::mid_quarter(d);
        self.forms
            .iter()
            .flat_map(|f| f.claim.assets())
            .map(|a| a.deduction(mid_quarter) - a.deduction_for(mid_quarter, true))
            .sum()
    }

    // The performing artists' tests against the AGI before any artist's
    // deduction (whole dollars). Returns the positions of the forms that count
    // (a form with nothing to deduct counts trivially); each failed test is one
    // note and leaves its form out.
    pub(crate) fn artists(
        &self,
        agi_before: i64,
        separate_without_apart: bool,
        r: &mut FederalResult,
    ) -> Vec<usize> {
        let mut counted = Vec::new();
        for form in
            self.forms.iter().filter(|f| f.claim.capacity == EmployeeCapacity::PerformingArtist)
        {
            let lines = &form.lines;
            let amount = dollars(lines.line10);
            if lines.line10 == 0 {
                counted.push(form.position);
                continue;
            }
            let context = form.claim.context(form.position);
            let name = form.claim.occupation.clone();
            let mut passed = true;
            let mut fail = |key: &'static str| {
                passed = false;
                crate::federal_rules::note_once_scoped(
                    r,
                    Message::new(
                        key,
                        alloc::vec![
                            MessageArg::Text(name.clone()),
                            MessageArg::usd_dollars(amount)
                        ],
                    ),
                    context.clone(),
                );
            };
            if lines.paying_200 < 2 {
                fail("WARN_PERFORMING_ARTIST_EMPLOYERS");
            }
            // More than 10% of the performing-arts gross income (box 1 of the chosen W-2s).
            if lines.line10 * 10 <= lines.gross {
                fail("WARN_PERFORMING_ARTIST_TEN_PERCENT");
            }
            if agi_before > 16000 {
                fail("WARN_PERFORMING_ARTIST_AGI");
            }
            if separate_without_apart {
                fail("WARN_PERFORMING_ARTIST_SEPARATE_RETURN");
            }
            if passed {
                counted.push(form.position);
            }
        }
        counted
    }

    // The artists' deduction, whole dollars, of the forms that counted.
    pub(crate) fn artist_deduction(&self, counted: &[usize]) -> i64 {
        self.forms
            .iter()
            .filter(|f| {
                f.claim.capacity == EmployeeCapacity::PerformingArtist
                    && counted.contains(&f.position)
            })
            .map(|f| dollars(f.lines.line10))
            .sum()
    }

    // The forms' calculation lines and the Schedule 1 line 12 total. A form
    // left out by an artist test shows line 10 as zero.
    pub(crate) fn emit(&self, r: &mut FederalResult, counted: &[usize]) {
        let source = "https://www.irs.gov/instructions/i2106";
        let mut total = 0;
        for form in &self.forms {
            let lines = &form.lines;
            let n = form.position + 1;
            let line10 = if form.claim.capacity == EmployeeCapacity::PerformingArtist
                && !counted.contains(&form.position)
            {
                0
            } else {
                dollars(lines.line10)
            };
            total += line10;
            emit_lines(
                r,
                &mut (),
                [
                    ("1", dollars(lines.line1)),
                    ("2", dollars(lines.line2)),
                    ("3", dollars(lines.line3)),
                    ("4", dollars(lines.line4)),
                    ("5", dollars(lines.line5)),
                    ("7", dollars(lines.line7)),
                    ("9", dollars(lines.line9)),
                    ("10", line10),
                ],
                |(line, amount), _, _| {
                    Some(calculation_row(
                        Some(FormLine::new(TaxForm::Form2106, line)),
                        &alloc::format!("employee_expenses_{n}_line_{line}"),
                        LineValue::MoneyDollars(amount),
                        &["adjustments.employee_expenses", "income.wages"],
                        source,
                        Message::new("EXPLAIN_EMPLOYEE_EXPENSES_N_LINE_N", alloc::vec![]),
                    ))
                },
            );
            for (index, (operating, depreciation)) in
                lines.operating.iter().zip(&lines.depreciation).enumerate()
            {
                emit_lines(
                    r,
                    &mut (),
                    [("operating", "27", *operating), ("depreciation", "28", *depreciation)],
                    |(suffix, line, amount), _, _| {
                        Some(calculation_row(
                            Some(FormLine::new(TaxForm::Form2106, line)),
                            &alloc::format!("employee_expenses_{n}_vehicle_{}_{suffix}", index + 1),
                            LineValue::MoneyDollars(dollars(amount)),
                            &["adjustments.employee_expenses"],
                            source,
                            Message::new("EXPLAIN_EMPLOYEE_EXPENSES_N_VEHICLE_N", alloc::vec![]),
                        ))
                    },
                );
            }
            for (index, lease) in form.claim.leased_vehicles.iter().enumerate() {
                let a = lease.amounts();
                emit_lines(
                    r,
                    &mut (),
                    [
                        ("24a", a.payments),
                        ("24b", a.inclusion),
                        ("24c", a.net_payments()),
                        ("29", a.total()),
                    ],
                    |(line, amount), _, _| {
                        Some(calculation_row(
                            Some(FormLine::new(TaxForm::Form2106, line)),
                            &alloc::format!(
                                "employee_expenses_{n}_lease_{}_line_{line}",
                                index + 1
                            ),
                            LineValue::MoneyDollars(dollars(amount)),
                            &["adjustments.employee_expenses"],
                            source,
                            Message::new("EXPLAIN_LEASE_AMOUNT", alloc::vec![]),
                        ))
                    },
                );
            }
            // Form 4797 Part IV column (b): a vehicle whose business use fell to 50%
            // or less adds its excess depreciation to Schedule 1 line 8z.
            for (index, asset) in form.claim.assets().iter().enumerate() {
                let Some(recapture) = asset.recapture() else {
                    continue;
                };
                emit_lines(
                    r,
                    &mut (),
                    [
                        (
                            "listed_prior_depreciation",
                            Some(FormLine::new(TaxForm::Form4797, "35(b)")),
                            recapture.allowed,
                            "EXPLAIN_BUSINESS_N_ASSET_N_LISTED_PRIOR_DEPRECIATION",
                        ),
                        (
                            "listed_recomputed",
                            Some(FormLine::new(TaxForm::Form4797, "36(b)")),
                            recapture.recomputed,
                            "EXPLAIN_BUSINESS_N_ASSET_N_LISTED_RECOMPUTED",
                        ),
                        (
                            "listed_recapture",
                            Some(FormLine::new(TaxForm::Form4797, "37(b)")),
                            recapture.excess(),
                            "EXPLAIN_BUSINESS_N_ASSET_N_LISTED_RECAPTURE",
                        ),
                        (
                            "other_income",
                            None,
                            recapture.excess(),
                            "EXPLAIN_EMPLOYEE_EXPENSES_N_VEHICLE_N_OTHER_INCOME",
                        ),
                    ],
                    |(id_suffix, line, amount, explanation), _, _| {
                        Some(calculation_row(
                            line,
                            &alloc::format!(
                                "employee_expenses_{n}_vehicle_{}_{id_suffix}",
                                index + 1
                            ),
                            LineValue::MoneyCents(amount),
                            &["adjustments.employee_expenses"],
                            "https://www.irs.gov/instructions/i4797",
                            Message::new(explanation, alloc::vec![]),
                        ))
                    },
                );
            }
            if lines.excess != 0 {
                crate::kernel::lines::line(
                    r,
                    &alloc::format!("employee_expenses_{n}_excess_reimbursement"),
                    dollars(lines.excess),
                    &["adjustments.employee_expenses", "income.wages"],
                    source,
                    Message::new("EXPLAIN_EMPLOYEE_EXPENSES_N_EXCESS", alloc::vec![]),
                );
                crate::federal_rules::note_once_scoped(
                    r,
                    Message::new(
                        "NOTE_EMPLOYEE_EXPENSES_EXCESS_REIMBURSEMENT",
                        alloc::vec![
                            MessageArg::Text(form.claim.occupation.clone()),
                            MessageArg::usd_dollars(dollars(lines.excess)),
                        ],
                    ),
                    form.claim
                        .context(form.position)
                        .alternatives(&[TaxSection::EmployeeExpenses, TaxSection::Wages]),
                );
            }
        }
        if total != 0 {
            crate::kernel::lines::form_line(
                r,
                FormLine::new(TaxForm::Schedule1, "12"),
                "schedule_1_employee_business_expenses",
                total,
                &["adjustments.employee_expenses"],
                source,
                Message::new("EXPLAIN_SCHEDULE_1_12", alloc::vec![]),
            );
        }
    }
}

// The employee vehicle assets of the return as depreciation::assets shares
// them: owned copies with section 179 left out.
pub(crate) fn vehicle_assets(d: &FederalDraft) -> impl Iterator<Item = Cow<'_, BusinessAsset>> {
    d.adjustments
        .iter()
        .flat_map(|a| a.employee_expenses.iter())
        .flat_map(|claim| claim.assets())
        .map(Cow::Owned)
}
