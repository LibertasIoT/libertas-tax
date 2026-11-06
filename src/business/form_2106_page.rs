// form_2106 items of src/federal/interview/mod.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;
// The reverse of employee_expenses_candidate's mapping for one stored form:
// the chosen W-2s that are still the owner's, the standard-mileage vehicles
// and then the actual-expense ones as the Vehicle page lists them.
pub(super) fn employee_expense_form(
    claim: &crate::EmployeeExpenseClaim,
    wages: &[crate::WageChoice],
) -> crate::EmployeeExpenseForm {
    let mut vehicles: Vec<Vehicle> = claim.mileage.iter().map(mileage_item).collect();
    for asset in &claim.vehicle_assets {
        vehicles.extend(vehicle_item(asset));
    }
    vehicles.extend(
        claim
            .leased_vehicles
            .iter()
            .cloned()
            .map(|vehicle| Vehicle { method: VehicleMethod::Leased { vehicle } }),
    );
    crate::EmployeeExpenseForm {
        improvement_buildings: claim.improvement_buildings.clone(),
        owner: claim.owner,
        equipment_assets: claim.equipment_assets.iter().map(asset_item).collect(),
        occupation: claim.occupation.clone(),
        capacity: claim.capacity,
        dot_hours_of_service: claim.dot_hours_of_service,
        employers: claim
            .employers
            .iter()
            .filter(|id| wages.iter().any(|w| w.id == **id && w.owner == claim.owner))
            .copied()
            .collect(),
        local_transportation: claim.local_transportation,
        travel: claim.travel,
        other: claim.other,
        meals: claim.meals,
        reimbursements: claim.reimbursements,
        vehicles,
    }
}

impl Interview {
    // The Form 2106 page, opened with its complete value: one row per stored
    // form, and the return's W-2s as the employers a form can choose.
    pub(super) fn employee_expenses_page(&self, review: bool) -> Reply {
        let d = &self.draft;
        let wages = employee_wage_choices(d);
        let mut forms: Vec<crate::EmployeeExpenseForm> = d
            .adjustments
            .as_ref()
            .map(|a| {
                a.employee_expenses
                    .iter()
                    .map(|claim| employee_expense_form(claim, &wages))
                    .collect()
            })
            .unwrap_or_default();
        let macrs = crate::macrs_source::MacrsElectionView::new(d);
        for form in &mut forms {
            for asset in &mut form.equipment_assets {
                refresh_asset_macrs_questions(asset, form.owner, None, false, &macrs);
            }
        }
        Reply::Edit(P::SaveEmployeeExpenses {
            rental_energy_buildings: self.rental_energy_buildings(),
            initial_asset_amt_carry_choices: Vec::new(),
            user: self.user,
            cookie: d.cookie.clone(),
            revision: d.revision,
            page: ADJUSTMENTS_PAGE,
            previous: ADJUSTMENTS_PAGE,
            review,
            estimate: self.estimate_now(TaxSection::EmployeeExpenses),
            warnings: Vec::new(),
            progress: self.topic_progress(ADJUSTMENTS_PAGE, Some(TaxSection::EmployeeExpenses)),
            allowed_owners: self.allowed_owners(),
            value: EmployeeExpenses { forms, wages },
        })
    }
    // The Form 2106 page merged into the accepted return, with the server
    // twins of its client rules (the employer pick, the reservist's vehicles,
    // the interest and property tax an employee cannot deduct) and the checks
    // no client rule states: each employer is the row owner's own W-2, chosen
    // by one form only.
    pub(super) fn employee_expenses_candidate(
        &self,
        value: &EmployeeExpenses,
    ) -> Result<FederalDraft, SaveFailure> {
        let status = |key: &str| SaveFailure::from(crate::interview::status(key));
        let d = &self.draft;
        let owners = self.allowed_owners();
        let wages = employee_wage_choices(d);
        let identity_rows: Vec<_> = value
            .forms
            .iter()
            .enumerate()
            .flat_map(|(form_index, form)| {
                form.vehicles
                    .iter()
                    .enumerate()
                    .filter(|(_, vehicle)| matches!(vehicle.method, VehicleMethod::Actual { .. }))
                    .map(move |(vehicle_index, _)| {
                        ValidationIssue::new("SOURCE_ID")
                            .at_field("id")
                            .in_field("method")
                            .in_list("vehicles", vehicle_index)
                            .in_list("forms", form_index)
                            .in_field("value")
                    })
                    .chain(form.equipment_assets.iter().enumerate().map(move |(index, _)| {
                        ValidationIssue::new("SOURCE_ID")
                            .at_field("id")
                            .in_list("equipment_assets", index)
                            .in_list("forms", form_index)
                            .in_field("value")
                    }))
            })
            .collect();
        let mut claims = Vec::new();
        let mut used: Vec<i64> = Vec::new();
        for (form_index, form) in value.forms.iter().enumerate() {
            if !owners.contains(&(form.owner as i32)) {
                return Err(ValidationIssue::new(
                    "MESSAGE_YOUR_SPOUSE_S_EXPENSES_BELONG_ON_THIS_RETURN_ONLY_WHEN_YOU_FILE_JOINTLY",
                ).at_field("owner").in_list("forms", form_index).in_field("value").into());
            }
            if form.employers.is_empty() {
                return Err(ValidationIssue::new("EMPLOYEE_EXPENSES_EMPLOYER_NEEDED")
                    .at_field("employers")
                    .in_list("forms", form_index)
                    .in_field("value")
                    .into());
            }
            let employers = &form.employers;
            if let Some((employer_index, _)) =
                employers.iter().enumerate().find(|(i, id)| employers[..*i].contains(id))
            {
                return Err(ValidationIssue::new("EMPLOYEE_EXPENSES_EMPLOYER_TWICE")
                    .at_list_item("employers", employer_index)
                    .in_list("forms", form_index)
                    .in_field("value")
                    .into());
            }
            if let Some(employer_index) = employers
                .iter()
                .position(|id| !wages.iter().any(|w| w.id == *id && w.owner == form.owner))
            {
                return Err(ValidationIssue::new("MESSAGE_EMPLOYER_MUST_BE_YOUR_W_2")
                    .at_list_item("employers", employer_index)
                    .in_list("forms", form_index)
                    .in_field("value")
                    .into());
            }
            if let Some(employer_index) = employers.iter().position(|id| used.contains(id)) {
                return Err(ValidationIssue::new("MESSAGE_EMPLOYER_CHOSEN_ON_ONE_FORM_2106_ONLY")
                    .at_list_item("employers", employer_index)
                    .in_list("forms", form_index)
                    .in_field("value")
                    .into());
            }
            used.extend(employers);
            if form.capacity == EmployeeCapacity::Reservist
                && let Some(vehicle_index) = form
                    .vehicles
                    .iter()
                    .position(|vehicle| matches!(vehicle.method, VehicleMethod::Actual { .. }))
            {
                return Err(ValidationIssue::new("RESERVIST_VEHICLE_STANDARD_MILEAGE")
                    .at_field("method")
                    .in_list("vehicles", vehicle_index)
                    .in_list("forms", form_index)
                    .in_field("value")
                    .into());
            }
            if let Some(issue) =
                form.vehicles.iter().enumerate().find_map(|(vehicle_index, vehicle)| {
                    let issue = match &vehicle.method {
                        VehicleMethod::StandardMileage { more, .. }
                            if more.interest > 0 || more.property_tax > 0 =>
                        {
                            Some(
                                ValidationIssue::new("EMPLOYEE_VEHICLE_NO_INTEREST_OR_TAX")
                                    .at_field(if more.interest > 0 {
                                        "interest"
                                    } else {
                                        "property_tax"
                                    })
                                    .in_field("more"),
                            )
                        }
                        VehicleMethod::Actual { loan_interest, .. } if *loan_interest > 0 => Some(
                            ValidationIssue::new("EMPLOYEE_VEHICLE_NO_INTEREST_OR_TAX")
                                .at_field("loan_interest"),
                        ),
                        _ => None,
                    };
                    issue.map(|issue| issue.in_field("method").in_list("vehicles", vehicle_index))
                })
            {
                return Err(issue.in_list("forms", form_index).in_field("value").into());
            }
            if let Some(issue) = shared_business_issue(&form.vehicles, &form.equipment_assets) {
                return Err(issue
                    .map_field_prefix(&["assets"], &["equipment_assets"])
                    .in_list("forms", form_index)
                    .in_field("value")
                    .into());
            }
            let (mileage, assets) =
                vehicle_and_asset_records(&form.vehicles, &form.equipment_assets);
            let (vehicle_assets, equipment_assets) =
                assets.into_iter().partition(BusinessAsset::is_vehicle);
            let claim = crate::EmployeeExpenseClaim {
                improvement_buildings: form.improvement_buildings.clone(),
                owner: form.owner,
                occupation: form.occupation.trim().into(),
                capacity: form.capacity,
                dot_hours_of_service: form.dot_hours_of_service,
                leased_vehicles: form
                    .vehicles
                    .iter()
                    .filter_map(|v| {
                        if let VehicleMethod::Leased { vehicle } = &v.method {
                            Some(vehicle.clone())
                        } else {
                            None
                        }
                    })
                    .collect(),
                employers: employers.clone(),
                local_transportation: form.local_transportation,
                travel: form.travel,
                other: form.other,
                meals: form.meals,
                reimbursements: form.reimbursements,
                mileage,
                vehicle_assets,
                equipment_assets,
            };
            if let Some(issue) = claim.admission_issue() {
                return Err(issue
                    .map_field_prefix(&["leased_vehicles"], &["vehicles"])
                    .map_field_prefix(&["vehicle_assets"], &["vehicles"])
                    .in_list("forms", form_index)
                    .in_field("value")
                    .into());
            }
            if let Some(issue) =
                crate::macrs_records::building_facts_issue(&claim.improvement_buildings)
            {
                return Err(issue.in_list("forms", form_index).in_field("value").into());
            }
            claims.push(claim);
        }
        let mut draft = d.clone();
        let Some(adjustments) = draft.adjustments.as_mut() else {
            return Err(status("PROBLEM_SECTION_UNAVAILABLE_YET"));
        };
        assign_record_ids(
            claims.iter_mut().flat_map(|claim| {
                claim
                    .vehicle_assets
                    .iter_mut()
                    .chain(&mut claim.equipment_assets)
                    .map(|asset| &mut asset.id)
            }),
            d.adjustments
                .iter()
                .flat_map(|adjustments| adjustments.employee_expenses.iter())
                .flat_map(|claim| {
                    claim.vehicle_assets.iter().chain(&claim.equipment_assets).map(|asset| asset.id)
                }),
            &mut draft.next_id,
        )
        .map_err(|(key, _, index)| {
            let mut issue = identity_rows[index].clone();
            issue.key = key;
            issue
        })?;
        let building_positions: Vec<_> = claims
            .iter()
            .enumerate()
            .flat_map(|(form, claim)| {
                (0..claim.improvement_buildings.len()).map(move |row| (form, row))
            })
            .collect();
        assign_record_ids(
            claims
                .iter_mut()
                .flat_map(|claim| claim.improvement_buildings.iter_mut().map(|row| &mut row.id)),
            d.adjustments
                .iter()
                .flat_map(|a| &a.employee_expenses)
                .flat_map(|claim| claim.improvement_buildings.iter().map(|row| row.id)),
            &mut draft.next_id,
        )
        .map_err(|(key, _, index)| {
            let (form, row) = building_positions[index];
            ValidationIssue::new(key)
                .at_field("id")
                .in_list("improvement_buildings", row)
                .in_list("forms", form)
                .in_field("value")
        })?;
        adjustments.employee_expenses = claims;
        visit(&self.draft, &mut draft, TaxSection::Adjustments, TaxSection::EmployeeExpenses);
        Ok(draft)
    }
}

impl Interview {
    pub(super) fn save_employee_expenses(
        &self,
        value: &EmployeeExpenses,
        page: i32,
    ) -> Result<FederalDraft, SaveFailure> {
        if page != ADJUSTMENTS_PAGE || !self.reached(page) {
            return Err(status("PROBLEM_SECTION_UNAVAILABLE_YET").into());
        }
        self.employee_expenses_candidate(value)
    }
}
