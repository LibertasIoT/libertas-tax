// Form 2106 employee business expenses: return admission, calculation and reporting.
// https://www.irs.gov/instructions/i2106
pub use crate::employee_expense_model::{EmployeeCapacity, EmployeeExpenseClaim};
use crate::kernel::lines::{LineValue, row as calculation_row};
use crate::kernel::ratio::{dollars, fraction, positive};
use crate::line_emission::emit as emit_lines;
use crate::{
    BusinessAsset, BusinessMileage, EquipmentHistory, FederalDraft, FederalIncome, FederalResult,
    FormLine, IssueContext, Message, MessageArg, Owner, Section179, TaxForm, TaxSection,
};
use alloc::{borrow::Cow, vec::Vec};

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

    pub(crate) fn all_assets(&self) -> Vec<BusinessAsset> {
        self.assets().into_iter().chain(self.equipment_assets.iter().cloned()).collect()
    }

    fn equipment_amounts(
        &self,
        mid_quarter: bool,
        amt: bool,
    ) -> crate::nonmacrs::AssetDepreciationAmounts {
        crate::nonmacrs::asset_depreciation_amounts(&self.equipment_assets, mid_quarter, amt)
    }

    pub(crate) fn equipment_issue(
        &self,
        mid_quarter: bool,
        macrs: Option<&crate::macrs_source::MacrsElectionView>,
    ) -> Option<(usize, &'static str)> {
        // Section62(a)(2)(E) admits reservist overnight duty-travel expenses,
        // not every job expense. The ordinary equipment record does not
        // establish the travel-attributable source needed for that exception.
        if self.capacity == EmployeeCapacity::Reservist && !self.equipment_assets.is_empty() {
            return Some((0, "EMPLOYEE_EQUIPMENT_RESERVIST_TRAVEL_SOURCE"));
        }
        for (index, asset) in self.equipment_assets.iter().enumerate() {
            if let Some(key) = asset.nonmacrs_issue() {
                return Some((index, key));
            }
            if !asset.qualifies() || asset.investment_credit != crate::InvestmentCredit::No {
                return Some((index, "EMPLOYEE_EQUIPMENT_RECOVERY_SOURCE"));
            }
            if let Some((key, _)) = asset.disposal_refusal_in_context(
                macrs.map_or_else(crate::macrs_source::AdsInvocation::default, |view| {
                    view.context(self.owner, None, asset)
                }),
            ) {
                return Some((index, key));
            }
        }
        for amt in [false, true] {
            let Some(recovery) = self.equipment_amounts(mid_quarter, amt).expense_cents(0) else {
                return Some((0, "ASSET_NONMACRS_FINANCIAL_UNAVAILABLE"));
            };
            if crate::kernel::ratio::checked_sum([
                recovery,
                self.other,
                self.local_transportation,
                self.travel,
                self.meals,
                self.reimbursements,
            ])
            .is_none()
            {
                return Some((0, "ASSET_NONMACRS_FINANCIAL_UNAVAILABLE"));
            }
        }
        None
    }

    // The W-2 of a chosen income record identity, when it is still this owner's.
    fn wage(&self, d: &FederalDraft, id: i64) -> Option<crate::FederalWage> {
        d.income.iter().find_map(|entry| match &entry.income {
            FederalIncome::Wage { data }
                if entry.id == id && entry.owner == crate::DocumentOwner::from(self.owner) =>
            {
                Some(crate::FederalWage::from(data))
            }
            _ => None,
        })
    }

    #[cfg(test)]
    pub(crate) fn lines(&self, d: &FederalDraft, mid_quarter: bool) -> Lines {
        self.lines_with_equipment(
            d,
            mid_quarter,
            crate::nonmacrs::asset_depreciation_amounts_for_owner(
                d,
                self.owner,
                &self.equipment_assets,
                mid_quarter,
                false,
                None,
            )
            .expense_cents(0)
            .expect("employee equipment source checked before calculation"),
            None,
        )
    }

    fn lines_with_equipment(
        &self,
        d: &FederalDraft,
        mid_quarter: bool,
        equipment: i64,
        macrs: Option<&crate::macrs_source::MacrsElectionView>,
    ) -> Lines {
        let assets = self.assets();
        let mut out = Lines::default();
        for asset in &assets {
            let operating = asset.vehicle_operating();
            let depreciation =
                if let Some(view) = macrs.filter(|view| view.affects(self.owner, None, asset)) {
                    crate::nonmacrs::asset_depreciation_amounts_for_view(
                        self.owner,
                        None,
                        core::slice::from_ref(asset),
                        mid_quarter,
                        false,
                        view,
                    )
                    .expense_cents(0)
                    .expect("employee vehicle source checked before calculation")
                } else {
                    asset.deduction(mid_quarter)
                };
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
        out.line4 = self
            .other
            .checked_add(equipment)
            .expect("employee equipment aggregate checked before calculation");
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
    equipment: [i64; 2],
}

pub(crate) struct Calculation {
    pub forms: Vec<Form>,
    macrs: alloc::sync::Arc<crate::macrs_source::MacrsElectionView>,
}

// Transient source keys match the existing Form4797 owner. Asset identities
// remain the accepted persistent IDs; the form position never replaces them.
pub(crate) fn equipment_source(d: &FederalDraft, source: i64) -> Option<&[BusinessAsset]> {
    d.adjustments
        .as_ref()?
        .employee_expenses
        .iter()
        .enumerate()
        .find(|(position, _)| crate::asset_sale::employee_source_id(*position) == source)
        .map(|(_, claim)| claim.equipment_assets.as_slice())
}

// Reg.1.179-2(c)(1),(6)(iv) and Pub946, Business Income Limit: compensation
// is computed without the section179 deduction or unreimbursed employee
// expenses. Reuse the Form2106 reimbursement worksheet with zero allowed179;
// the final Form2106 separately receives its actual bound allowance.
pub(crate) fn before179_compensation(
    d: &FederalDraft,
) -> Result<i64, crate::Section179PreparedIssue> {
    Ok(before179_compensation_sources(d, None)?.total)
}

// One native pass retains both the checked return total and each employee's
// actual compensation. Source cents cross into the same federal wage buckets.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Before179Compensation {
    pub total: i64,
    pub by_owner: [i64; 2],
}

pub(crate) fn before179_compensation_sources(
    d: &FederalDraft,
    macrs: Option<&crate::macrs_source::MacrsElectionView>,
) -> Result<Before179Compensation, crate::Section179PreparedIssue> {
    let mid_quarter = crate::depreciation::mid_quarter_with_macrs(d, macrs);
    d.adjustments.iter().flat_map(|a| &a.employee_expenses).enumerate().try_fold(
        Before179Compensation::default(),
        |mut total, (position, claim)| {
            // With no reimbursement source, unknown deductions cannot create
            // compensation. A missing employer is not a certified zero codeL.
            if claim.reimbursements == 0
                && claim.employers.iter().all(|id| {
                    claim.wage(d, *id).is_some_and(|wage| {
                        wage.other_box12
                            .iter()
                            .all(|entry| entry.code != crate::W2Code::L || entry.amount == 0)
                    })
                })
            {
                return Ok(total);
            }
            let source = crate::asset_sale::employee_source_id(position);
            if let Some(issue) = claim.admission_issue() {
                return Err(crate::Section179PreparedIssue::InvalidSource(source, issue.key));
            }
            if let Some((_, key)) = claim.equipment_issue(mid_quarter, macrs) {
                return Err(crate::Section179PreparedIssue::SourceBoundaryUnavailable(source, key));
            }
            if claim.assets().iter().any(|asset| !asset.vehicle_supported() || !asset.qualifies()) {
                return Err(crate::Section179PreparedIssue::SourceBoundaryUnavailable(
                    source,
                    "ISSUE_EMPLOYEE_VEHICLE_ACTUAL_EXPENSES_SCOPE",
                ));
            }
            if let Some(view) = macrs
                && let Some(key) = claim.assets().iter().find_map(|asset| {
                    asset.macrs_history_issue_in_context(view.context(claim.owner, None, asset))
                })
            {
                return Err(crate::Section179PreparedIssue::SourceBoundaryUnavailable(source, key));
            }
            let equipment = crate::nonmacrs::asset_depreciation_amounts_for_owner(
                d,
                claim.owner,
                &claim.equipment_assets,
                mid_quarter,
                false,
                macrs,
            )
            .expense_cents(0)
            .ok_or(crate::Section179PreparedIssue::AmountOverflow(source))?;
            let lines = claim.lines_with_equipment(d, mid_quarter, equipment, macrs);
            if lines.missing != 0 {
                return Err(crate::Section179PreparedIssue::SourceBoundaryUnavailable(
                    source,
                    "ISSUE_EMPLOYEE_EXPENSES_EMPLOYER_MISSING",
                ));
            }
            total.total = total
                .total
                .checked_add(lines.excess)
                .ok_or(crate::Section179PreparedIssue::AmountOverflow(source))?;
            let owner = usize::from(claim.owner == Owner::Spouse);
            total.by_owner[owner] = total.by_owner[owner]
                .checked_add(lines.excess)
                .ok_or(crate::Section179PreparedIssue::AmountOverflow(source))?;
            Ok(total)
        },
    )
}

// The forms' lines, figured before the return's income (the excess
// reimbursement is wage income) and checked for the scope the rules support.
#[cfg(test)]
pub(crate) fn calculate(d: &FederalDraft, r: &mut FederalResult) -> Calculation {
    calculate_with_section179(d, r, None, None)
}

pub(crate) fn calculate_with_section179(
    d: &FederalDraft,
    r: &mut FederalResult,
    selected: Option<&[crate::section179_return::Selected; 2]>,
    macrs: Option<&alloc::sync::Arc<crate::macrs_source::MacrsElectionView>>,
) -> Calculation {
    let macrs = macrs
        .cloned()
        .unwrap_or_else(|| alloc::sync::Arc::new(crate::macrs_source::MacrsElectionView::new(d)));
    let Some(adjustments) = d.adjustments.as_ref() else {
        return Calculation { forms: Vec::new(), macrs };
    };
    let mid_quarter = crate::depreciation::mid_quarter_with_macrs(d, Some(macrs.as_ref()));
    let mut forms = Vec::new();
    for (position, claim) in adjustments.employee_expenses.iter().enumerate() {
        let context = claim.context(position);
        if let Some((index, key)) = claim.equipment_issue(mid_quarter, Some(&macrs)) {
            crate::issue_localized!(
                r,
                &context.item("equipment_assets", index),
                "Form 2106 equipment",
                key,
                "Retain the actual equipment recovery, election and disposal records. This employee equipment source is not yet available for a complete estimate."
            );
            continue;
        }
        if let Some((index, key)) = claim.assets().iter().enumerate().find_map(|(index, asset)| {
            asset
                .macrs_history_issue_in_context(macrs.context(claim.owner, None, asset))
                .map(|key| (index, key))
        }) {
            crate::issue_localized!(
                r,
                &context.item("vehicle_assets", index),
                "Form 2106 vehicle",
                key,
                "The property's actual recovery history is needed for disposition or recapture."
            );
            continue;
        }
        let equipment = (|| {
            let mut amounts = [0; 2];
            for (system, amt) in [false, true].into_iter().enumerate() {
                let (assets, allowed) = if let Some(selected) = selected {
                    let allocation = &selected[system].allocation;
                    (
                        allocation.employee_equipment(position)?,
                        allocation.employee_allowed(position)?,
                    )
                } else {
                    if claim.equipment_assets.iter().any(|asset| {
                        asset.available179() > 0 || asset.amt_section179_carry() != Ok(0)
                    }) {
                        return Err(crate::Section179PreparedIssue::SourceBoundaryUnavailable(
                            crate::asset_sale::employee_source_id(position),
                            "EMPLOYEE_EQUIPMENT_SECTION179_SOURCE",
                        ));
                    }
                    (&claim.equipment_assets[..], 0)
                };
                amounts[system] = crate::nonmacrs::asset_depreciation_amounts_for_owner(
                    d,
                    claim.owner,
                    assets,
                    mid_quarter,
                    selected.is_none() && amt,
                    Some(macrs.as_ref()),
                )
                .expense_cents(allowed)
                .ok_or(crate::Section179PreparedIssue::AmountOverflow(
                    crate::asset_sale::employee_source_id(position),
                ))?;
                crate::kernel::ratio::checked_sum([
                    amounts[system],
                    claim.other,
                    claim.local_transportation,
                    claim.travel,
                    claim.meals,
                    claim.reimbursements,
                ])
                .ok_or(crate::Section179PreparedIssue::AmountOverflow(
                    crate::asset_sale::employee_source_id(position),
                ))?;
            }
            Ok::<_, crate::Section179PreparedIssue>(amounts)
        })();
        let equipment = match equipment {
            Ok(amounts) => amounts,
            Err(issue) => {
                crate::section179_return::Issue::Prepared(issue).report(r);
                continue;
            }
        };
        let lines = claim.lines_with_equipment(d, mid_quarter, equipment[0], Some(macrs.as_ref()));
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
        for (index, asset) in claim.equipment_assets.iter().enumerate() {
            crate::asset_sale::report_business_asset(
                r,
                #[cfg(test)]
                "Form 2106 equipment",
                &context.item("equipment_assets", index),
                &alloc::format!("employee_expenses_{}_equipment_{}", position + 1, index + 1),
                index,
                &claim.equipment_assets,
                0,
                asset.sale_in_context(mid_quarter, false, macrs.context(claim.owner, None, asset)),
                asset.partial_for(),
                "adjustments.employee_expenses.equipment_assets",
            );
        }
        forms.push(Form { position, claim: claim.clone(), lines, equipment });
    }
    Calculation { forms, macrs }
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

    // i6251 line2l compares the regular and AMT depreciation deductions.
    // Equipment includes each system's bound section179 allowance once;
    // reimbursement and artist worksheets retain their existing owners.
    // https://www.irs.gov/instructions/i6251
    pub(crate) fn amt_adjustment(&self, d: &FederalDraft) -> i64 {
        let mid_quarter = crate::depreciation::mid_quarter_with_macrs(d, Some(self.macrs.as_ref()));
        self.forms
            .iter()
            .map(|form| {
                let vehicles: i64 = form
                    .claim
                    .assets()
                    .iter()
                    .map(|a| {
                        if !self.macrs.affects(form.claim.owner, None, a) {
                            return a.deduction(mid_quarter) - a.deduction_for(mid_quarter, true);
                        }
                        let amount = |amt| {
                            crate::nonmacrs::asset_depreciation_amounts_for_view(
                                form.claim.owner,
                                None,
                                core::slice::from_ref(a),
                                mid_quarter,
                                amt,
                                &self.macrs,
                            )
                            .expense_cents(0)
                            .expect("employee vehicle source checked before calculation")
                        };
                        amount(false) - amount(true)
                    })
                    .sum();
                vehicles + form.equipment[0] - form.equipment[1]
            })
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
    pub(crate) fn emit(&self, d: &FederalDraft, r: &mut FederalResult, counted: &[usize]) {
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
            let equipment = crate::nonmacrs::asset_depreciation_amounts_for_owner(
                d,
                form.claim.owner,
                &form.claim.equipment_assets,
                crate::depreciation::mid_quarter_with_macrs(d, Some(self.macrs.as_ref())),
                false,
                Some(self.macrs.as_ref()),
            );
            if crate::depreciation::assets_need_form_4562(&form.claim.equipment_assets) {
                for (class, amount) in &equipment.classes {
                    if *amount == crate::kernel::ratio::exact::whole(0) {
                        continue;
                    }
                    crate::line_emission::form_line(
                        r,
                        FormLine::new(TaxForm::Form4562, class),
                        &alloc::format!(
                            "employee_expenses_{n}_equipment_depreciation_class_{}",
                            class.to_ascii_uppercase()
                        ),
                        crate::kernel::ratio::exact::dollars(amount)
                            .expect("admitted employee equipment recovery"),
                        &["adjustments.employee_expenses.equipment_assets"],
                        "https://www.irs.gov/instructions/i4562",
                        Message::new("EXPLAIN_BUSINESS_N_ASSET_N_N_NONMACRS", alloc::vec![]),
                    );
                }
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
                crate::line_emission::line(
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
            crate::line_emission::form_line(
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

// Employee property for shared recovery conventions. Only vehicle elections
// follow their existing omitted-election treatment; equipment keeps its sources.
pub(crate) fn property_assets(d: &FederalDraft) -> impl Iterator<Item = Cow<'_, BusinessAsset>> {
    d.adjustments
        .iter()
        .flat_map(|a| a.employee_expenses.iter())
        .flat_map(|claim| claim.all_assets())
        .map(Cow::Owned)
}
