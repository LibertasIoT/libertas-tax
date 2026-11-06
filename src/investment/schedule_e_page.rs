// schedule_e items of src/federal/interview/mod.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

// The Schedule E page (unit M3-P6) is a projection of FederalRental: each
// nonzero line one ordinary RentalExpense, the building then the improvements
// as RentalBuilding records, and the "more" answers' Nothing branch an
// active, fully at-risk, investment-only rental with no suspended losses,
// payroll, group, home-use carryovers or special treatment. The reverse
// mapping rebuilds the page and is accepted only when saving it again
// reproduces the stored record exactly.
// Schedule E lines 5-17 in printed order, each with its stored category.
pub(super) fn rental_expense_lines(e: &RentalExpenses) -> [(RentalExpenseCategory, i64); 14] {
    use RentalExpenseCategory as C;
    [
        (C::Advertising, e.advertising),
        (C::AutoAndTravel, e.auto_and_travel),
        (C::CleaningAndMaintenance, e.cleaning),
        (C::Commissions, e.commissions),
        (C::Insurance, e.insurance),
        (C::LegalAndProfessional, e.legal),
        (C::ManagementFees, e.management),
        (C::MortgageInterest, e.mortgage_interest),
        (C::VehicleLoanInterest, e.vehicle_loan_interest),
        (C::OtherInterest, e.other_interest),
        (C::Repairs, e.repairs),
        (C::Supplies, e.supplies),
        (C::TaxesAndLicenses, e.taxes),
        (C::Utilities, e.utilities),
    ]
}
pub(super) fn rental_expense_records(e: &RentalExpenses) -> Vec<RentalExpense> {
    rental_expense_lines(e)
        .into_iter()
        .filter(|(_, amount)| *amount != 0)
        .map(|(category, amount)| RentalExpense {
            category,
            amount,
            treatment: BusinessExpenseKind::Ordinary,
        })
        .chain(e.other.iter().map(|other| RentalExpense {
            category: RentalExpenseCategory::Other { label: other.description.clone() },
            amount: other.amount,
            treatment: BusinessExpenseKind::Ordinary,
        }))
        .collect()
}
pub(super) fn rental_expense_items(records: &[RentalExpense]) -> RentalExpenses {
    let mut e = RentalExpenses {
        advertising: 0,
        auto_and_travel: 0,
        cleaning: 0,
        commissions: 0,
        insurance: 0,
        legal: 0,
        management: 0,
        mortgage_interest: 0,
        vehicle_loan_interest: 0,
        other_interest: 0,
        repairs: 0,
        supplies: 0,
        taxes: 0,
        utilities: 0,
        other: Vec::new(),
    };
    for record in records {
        use RentalExpenseCategory as C;
        let line = match &record.category {
            C::Advertising => &mut e.advertising,
            C::AutoAndTravel => &mut e.auto_and_travel,
            C::CleaningAndMaintenance => &mut e.cleaning,
            C::Commissions => &mut e.commissions,
            C::Insurance => &mut e.insurance,
            C::LegalAndProfessional => &mut e.legal,
            C::ManagementFees => &mut e.management,
            C::MortgageInterest => &mut e.mortgage_interest,
            C::VehicleLoanInterest => &mut e.vehicle_loan_interest,
            C::OtherInterest => &mut e.other_interest,
            C::Repairs => &mut e.repairs,
            C::Supplies => &mut e.supplies,
            C::TaxesAndLicenses => &mut e.taxes,
            C::Utilities => &mut e.utilities,
            C::Other { label } => {
                e.other.push(OtherExpense { description: label.clone(), amount: record.amount });
                continue;
            }
        };
        *line = record.amount;
    }
    e
}
pub(super) fn building_record(b: &Building) -> RentalBuilding {
    RentalBuilding {
        casualty: b.casualty.clone(),
        casualty_property: b.casualty_property,
        id: b.id,
        energy: b.energy.as_ref().map(Into::into),
        sale_history: b.sale_history.clone(),
        credit_reduction: b.credit_reduction,
        cost: b.cost,
        land: b.land,
        in_service: b.in_service,
        commercial_1993_transition: b.commercial_1993_transition.clone(),
        acquired: b.acquired.clone(),
        rehabilitation: None,
        partial: b.partial.clone(),
    }
}
// An improvement is a stored building row with no land.
pub(super) fn improvement_record(b: &Improvement) -> RentalBuilding {
    RentalBuilding {
        casualty: b.casualty.clone(),
        casualty_property: b.casualty_property,
        id: b.id,
        energy: b.energy.as_ref().map(Into::into),
        sale_history: b.sale_history.clone(),
        credit_reduction: b.credit_reduction,
        cost: b.cost,
        land: 0,
        in_service: b.in_service,
        commercial_1993_transition: b.commercial_1993_transition.clone(),
        acquired: crate::RentalAcquisition::Purchased,
        rehabilitation: b.rehabilitation.clone(),
        partial: b.partial.clone(),
    }
}
pub(super) fn building_item(
    record: &RentalBuilding,
    kind: &RentalPropertyType,
    location: &crate::RentalLocation,
) -> Building {
    Building {
        casualty: record.casualty.clone(),
        casualty_property: record.casualty_property,
        id: record.id,
        energy: record.energy.as_ref().map(Into::into),
        sale_history: record.sale_history.clone(),
        credit_reduction: record.credit_reduction,
        cost: record.cost,
        land: record.land,
        in_service: record.in_service,
        commercial_1993_transition: record.commercial_1993_transition.clone(),
        allowed_commercial_1993_transition: crate::rental_recovery::allowed(
            kind,
            location,
            record.in_service,
        ),
        acquired: record.acquired.clone(),
        partial: record.partial.clone(),
    }
}
// None for a stored improvement row the page cannot show (it holds land).
pub(super) fn improvement_item(
    record: &RentalBuilding,
    kind: &RentalPropertyType,
    location: &crate::RentalLocation,
) -> Option<Improvement> {
    (record.land == 0).then(|| Improvement {
        casualty: record.casualty.clone(),
        casualty_property: record.casualty_property,
        id: record.id,
        energy: record.energy.as_ref().map(Into::into),
        sale_history: record.sale_history.clone(),
        credit_reduction: record.credit_reduction,
        cost: record.cost,
        in_service: record.in_service,
        commercial_1993_transition: record.commercial_1993_transition.clone(),
        allowed_commercial_1993_transition: crate::rental_recovery::allowed(
            kind,
            location,
            record.in_service,
        ),
        rehabilitation: record.rehabilitation.clone(),
        partial: record.partial.clone(),
    })
}
// Personal use onto RentalPersonalUse; last year's home-use carryovers are
// stored beside it, so a rental with none this year is a plain No (they are
// still limited to the rent left, section 280A(c)(5)).
pub(super) fn personal_use_record(personal_use: &PersonalUse) -> RentalPersonalUse {
    match personal_use {
        PersonalUse::No => RentalPersonalUse::No,
        PersonalUse::Yes { fair_rental_days, personal_days, overlap_days, direct_expenses } => {
            RentalPersonalUse::Yes {
                fair_rental_days: *fair_rental_days,
                personal_days: *personal_days,
                overlap_days: *overlap_days,
                direct_expenses: *direct_expenses,
            }
        }
    }
}
// The suspended losses onto a rental's history. The loss-year participation
// answer belongs to a loss; with none it is the default.
pub(crate) fn rental_history(suspended: &SuspendedLoss) -> RentalHistory {
    RentalHistory {
        regular: suspended.regular,
        amt: suspended.amt,
        active_in_loss_year: suspended.active_in_loss_year
            || (suspended.regular == 0 && suspended.amt == 0),
    }
}
// The page's rental onto FederalRental.
pub(super) fn rental_record(r: &Rental) -> FederalRental {
    let more = &r.more;
    let mut buildings = alloc::vec![building_record(&r.building)];
    buildings.extend(more.improvements.iter().map(improvement_record));
    FederalRental {
        improvement_buildings: r.improvement_buildings.clone(),
        interest_limit_election: r.interest_limit_election.clone().map(Into::into),
        location: r.location.clone(),
        energy_buildings: more.energy_buildings.iter().cloned().map(Into::into).collect(),
        property_qbi_history: more.property_qbi_history.clone(),
        related: r.related.forms.iter().map(|row| row.key).collect(),
        rent: r.rent,
        expenses: rental_expense_records(&r.expenses),
        buildings,
        participation: more.participation,
        ordinary_rental: !more.special,
        at_risk: more.at_risk.clone(),
        assets: r.assets.iter().cloned().map(Into::into).collect(),
        qbi_business: match &more.qbi {
            RentalBusiness::No => RentalBusiness::No,
            RentalBusiness::Yes { material_participation, .. } => {
                RentalBusiness::Yes { material_participation: *material_participation }
            }
        },
        history: rental_history(&more.suspended),
        method_changes: more.method_changes.clone(),
        payroll: more.payroll.clone(),
        qbi_aggregation_group: crate::common::optional_text(&more.qbi_group),
        personal_use: personal_use_record(&r.personal_use),
        home_operating_carryover: more.home_operating_carryover,
        prior_home_carry: r.prior_home_carry.clone(),
        property_type: r.kind.clone(),
        sale: r.sale.clone(),
        statement: r.statement.clone(),
        section179_business: r.section179_business,
    }
}
// The reverse of rental_record; None for a stored rental the page's answers
// would not reproduce. Zero days with carryovers reopen as no personal use.
pub(super) fn rental_item(entry_label: &str, data: &FederalRental) -> Option<Rental> {
    let (building, improvements) = data.buildings.split_first()?;
    let personal_use = match data.personal_use {
        RentalPersonalUse::No => PersonalUse::No,
        RentalPersonalUse::Yes {
            fair_rental_days,
            personal_days,
            overlap_days,
            direct_expenses,
        } => PersonalUse::Yes { fair_rental_days, personal_days, overlap_days, direct_expenses },
    };
    let item = Rental {
        improvement_buildings: data.improvement_buildings.clone(),
        interest_limit_election: data.interest_limit_election.clone().map(Into::into),
        allowed_interest_limit_election: crate::interest_limit::choices(
            data.interest_limit_election.is_some(),
        ),
        location: data.location.clone(),
        related: crate::RelatedForms {
            candidates: Vec::new(),
            forms: data
                .related
                .iter()
                .map(|&key| crate::RelatedFormView { key, summary: None })
                .collect(),
        },
        address: entry_label.into(),
        kind: data.property_type.clone(),
        assets: data.assets.iter().cloned().map(Into::into).collect(),
        rent: data.rent,
        expenses: rental_expense_items(&data.expenses),
        building: building_item(building, &data.property_type, &data.location),
        sale: data.sale.clone(),
        statement: data.statement.clone(),
        personal_use,
        more: RentalDetails {
            energy_buildings: data.energy_buildings.iter().cloned().map(Into::into).collect(),
            property_qbi_history: data.property_qbi_history.clone(),
            improvements: improvements
                .iter()
                .map(|row| improvement_item(row, &data.property_type, &data.location))
                .collect::<Option<_>>()?,
            participation: data.participation,
            suspended: SuspendedLoss {
                regular: data.history.regular,
                amt: data.history.amt,
                active_in_loss_year: data.history.active_in_loss_year,
            },
            method_changes: data.method_changes.clone(),
            home_operating_carryover: data.home_operating_carryover,
            qbi: data.qbi_business.clone(),
            qbi_group: data.qbi_aggregation_group.clone().unwrap_or_default(),
            payroll: data.payroll.clone(),
            at_risk: data.at_risk.clone(),
            known_at_risk: data.at_risk.iter().cloned().collect(),
            special: !data.ordinary_rental,
        },
        section179_business: data.section179_business,
        prior_home_carry: data.prior_home_carry.clone(),
    };
    (rental_record(&item) == *data).then_some(item)
}
// The page rules repeated on save: the land part of the building (and, on
// the server only, of each improvement) cannot exceed its cost, and the
// personal-use days cannot exceed the year.
// Intrinsic document checks have no return context. Save and preview supply
// the actual accepted year here for the casualty election window.
pub(super) fn rental_casualty_year_issue(
    document: &IncomeSource,
    year: i32,
) -> Option<ValidationIssue> {
    let IncomeSource::Record { value: crate::IncomeRecord::Rental(rental) } = document else {
        return None;
    };
    if let Some(source) = &rental.building.casualty
        && let Some(issue) = source.casualty.year_issue(year)
    {
        return Some(issue.in_field("casualty").in_field("casualty").in_field("building"));
    }
    rental.more.improvements.iter().enumerate().find_map(|(index, improvement)| {
        improvement.casualty.as_ref()?.casualty.year_issue(year).map(|issue| {
            issue
                .in_field("casualty")
                .in_field("casualty")
                .in_list("improvements", index)
                .in_field("more")
        })
    })
}

pub(super) fn rental_issue(r: &Rental) -> Option<crate::validation::ValidationIssue> {
    use crate::validation::ValidationIssue;
    if let Some(casualty) = &r.building.casualty
        && let Some(issue) =
            casualty.issue(r.building.cost.checked_sub(r.building.land).unwrap_or(-1))
    {
        return Some(issue.in_field("casualty").in_field("building"));
    }
    // A shared-event link describes an entered casualty record.
    if r.building.casualty.is_none() && r.building.casualty_property.is_some() {
        return Some(
            ValidationIssue::new("RENTAL_CASUALTY_EVENT_WITHOUT_CASUALTY")
                .at_field("casualty_property")
                .in_field("building"),
        );
    }
    for (index, improvement) in r.more.improvements.iter().enumerate() {
        if let Some(casualty) = &improvement.casualty
            && let Some(issue) = casualty.issue(improvement.cost)
        {
            return Some(
                issue.in_field("casualty").in_list("improvements", index).in_field("more"),
            );
        }
        if improvement.casualty.is_none() && improvement.casualty_property.is_some() {
            return Some(
                ValidationIssue::new("RENTAL_CASUALTY_EVENT_WITHOUT_CASUALTY")
                    .at_field("casualty_property")
                    .in_list("improvements", index)
                    .in_field("more"),
            );
        }
    }
    if let Some(issue) = crate::rental_home_carry::issue(&rental_record(r)) {
        return Some(issue.in_field("more"));
    }
    if let Some(key) = crate::rental_energy::source_issue(&rental_record(r)) {
        let issue = ValidationIssue::new(key);
        return Some(
            if key == "RENTAL_ENERGY_BUILDING_RECORDS" || key == "RENTAL_ENERGY_HISTORY_RECORDS" {
                issue.at_field("energy_buildings").in_field("more")
            } else if r.building.energy.is_some() {
                issue.at_field("energy").in_field("building")
            } else {
                issue.at_field("improvements").in_field("more")
            },
        );
    }
    if let Some(issue) = r.more.payroll.issue() {
        return Some(issue.in_field("payroll").in_field("more"));
    }
    if !matches!(r.personal_use, PersonalUse::No) && r.expenses.mortgage_interest != 0 {
        let issue = ValidationIssue::new("RENTAL_MORTGAGE_SINGLE_SOURCE");
        return Some(
            if !matches!(r.personal_use, PersonalUse::No) && r.expenses.mortgage_interest != 0 {
                issue.at_field("mortgage_interest").in_field("expenses")
            } else {
                issue.in_field("related")
            },
        );
    }
    if let Some(issue) =
        crate::activity_risk::source_issue(&r.more.at_risk, &rental_history(&r.more.suspended))
    {
        return Some(issue.in_field("more"));
    }
    if let Some(index) = r.assets.iter().position(|asset| asset.elected179() > 0) {
        return Some(
            ValidationIssue::new("RENTAL_ASSET_NO_SECTION_179")
                .at_field("elected")
                .in_field("section179")
                .in_field("history")
                .in_list("assets", index),
        );
    }
    if let Some(material) = r.kind.business_participation()
        && let RentalBusiness::Yes { material_participation } = r.more.qbi
        && material_participation != material
    {
        return Some(
            ValidationIssue::new("SHORT_TERM_PARTICIPATION_AGREES")
                .at_field("material_participation")
                .in_field("qbi")
                .in_field("more"),
        );
    }
    if let Some(issue) = crate::method_change::list_issue(&r.more.method_changes) {
        return Some(issue.in_field("more"));
    }
    if let Some(issue) = crate::rental_property_history::history_issue(
        &r.more.property_qbi_history,
        r.more.at_risk.as_ref().map_or(0, |risk| risk.prior_sale(false)),
    ) {
        return Some(issue.in_field("more"));
    }
    // Check both signs and ordering before subtracting untrusted source amounts.
    if r.building.land < 0 || r.building.cost < r.building.land {
        return Some(
            ValidationIssue::new("RENTAL_LAND_ABOVE_COST").at_field("land").in_field("building"),
        );
    }
    if r.building
        .sale_history
        .as_ref()
        .is_some_and(|history| !history.valid(r.building.cost - r.building.land))
    {
        return Some(
            ValidationIssue::new("ISSUE_RENTAL_BUILDING_DEPRECIATION_RECORDS")
                .at_field("sale_history")
                .in_field("building"),
        );
    }
    for (index, building) in r.more.improvements.iter().enumerate() {
        if building.sale_history.as_ref().is_some_and(|history| !history.valid(building.cost)) {
            return Some(
                ValidationIssue::new("ISSUE_RENTAL_BUILDING_DEPRECIATION_RECORDS")
                    .at_field("sale_history")
                    .in_list("improvements", index)
                    .in_field("more"),
            );
        }
    }
    let improvements: &[Improvement] = &r.more.improvements;
    if !(0..=r.building.cost - r.building.land).contains(&r.building.credit_reduction) {
        return Some(
            ValidationIssue::new("RENTAL_CREDIT_REDUCTION_ABOVE_BASIS")
                .at_field("credit_reduction")
                .in_field("building"),
        );
    }
    if let Some(index) = improvements
        .iter()
        .position(|building| !(0..=building.cost).contains(&building.credit_reduction))
    {
        return Some(
            ValidationIssue::new("RENTAL_CREDIT_REDUCTION_ABOVE_BASIS")
                .at_field("credit_reduction")
                .in_list("improvements", index)
                .in_field("more"),
        );
    }
    for (index, improvement) in improvements.iter().enumerate() {
        let Some(claim) = &improvement.rehabilitation else {
            continue;
        };
        if improvement.in_service
            < crate::year_constants::REHABILITATION_FIRST_PLACED_YEAR * 10000 + 101
        {
            return Some(
                ValidationIssue::new("REHABILITATION_NEEDS_AN_IMPROVEMENT_ROW")
                    .at_field("in_service")
                    .in_list("improvements", index)
                    .in_field("more"),
            );
        }
        if claim.period_start >= improvement.in_service {
            return Some(
                ValidationIssue::new("REHABILITATION_PERIOD_AFTER_PLACED")
                    .at_field("period_start")
                    .in_field("rehabilitation")
                    .in_list("improvements", index)
                    .in_field("more"),
            );
        }
        if let Some(issue) = claim.issue(improvement.in_service, improvement.cost) {
            return Some(
                issue.in_field("rehabilitation").in_list("improvements", index).in_field("more"),
            );
        }
    }
    if let Some(statement) = &r.statement {
        match r.sale {
            crate::RentalSale::Sold { date, price, .. } => {
                if date != statement.box_1_date_of_closing {
                    return Some(
                        ValidationIssue::new("RENTAL_STATEMENT_DATE_DIFFERS")
                            .at_field("date")
                            .in_field("sale"),
                    );
                }
                if price != statement.box_2a_total_gross_proceeds {
                    return Some(
                        ValidationIssue::new("RENTAL_STATEMENT_PROCEEDS_DIFFER")
                            .at_field("price")
                            .in_field("sale"),
                    );
                }
            }
            crate::RentalSale::Exchanged(ref exchange) => {
                if exchange.transferred != statement.box_1_date_of_closing {
                    return Some(
                        ValidationIssue::new("RENTAL_STATEMENT_DATE_DIFFERS")
                            .at_field("transferred")
                            .in_field("sale"),
                    );
                }
            }
            crate::RentalSale::Condemned { date, award, .. } => {
                if date != statement.box_1_date_of_closing {
                    return Some(
                        ValidationIssue::new("RENTAL_STATEMENT_DATE_DIFFERS")
                            .at_field("date")
                            .in_field("sale"),
                    );
                }
                if award != statement.box_2a_total_gross_proceeds {
                    return Some(
                        ValidationIssue::new("RENTAL_STATEMENT_PROCEEDS_DIFFER")
                            .at_field("award")
                            .in_field("sale"),
                    );
                }
            }
            crate::RentalSale::No => {}
        }
    }
    if let crate::RentalSale::Sold {
        date,
        price,
        land_price,
        ref payment,
        buyer,
        personal_share_percent,
        ..
    } = r.sale
    {
        if buyer != crate::Buyer::Unrelated && payment.installment() {
            return Some(
                ValidationIssue::new("INSTALLMENT_RELATED_BUYER")
                    .at_field("buyer")
                    .in_field("sale"),
            );
        }
        if personal_share_percent == 0 && !matches!(r.personal_use, PersonalUse::No) {
            return Some(
                ValidationIssue::new("RENTAL_SALE_PERSONAL_SHARE_NEEDED")
                    .at_field("personal_share_percent")
                    .in_field("sale"),
            );
        }
        if let Some(issue) = payment.issue(price) {
            return Some(issue.in_field("payment").in_field("sale"));
        }
        if core::iter::once(r.building.in_service)
            .chain(improvements.iter().map(|b| b.in_service))
            .any(|in_service| in_service >= date)
        {
            return Some(
                ValidationIssue::new("RENTAL_SOLD_BEFORE_IN_SERVICE")
                    .at_field("date")
                    .in_field("sale"),
            );
        }
        if land_price > price {
            return Some(
                ValidationIssue::new("RENTAL_LAND_PRICE_ABOVE_PRICE")
                    .at_field("land_price")
                    .in_field("sale"),
            );
        }
    }
    if let crate::RentalSale::Condemned { date, award, land_award, .. } = r.sale {
        if land_award > award {
            return Some(
                ValidationIssue::new("CONDEMNATION_LAND_ABOVE_AWARD")
                    .at_field("land_award")
                    .in_field("sale"),
            );
        }
        if core::iter::once(r.building.in_service)
            .chain(improvements.iter().map(|b| b.in_service))
            .any(|in_service| in_service >= date)
        {
            return Some(
                ValidationIssue::new("RENTAL_SOLD_BEFORE_IN_SERVICE")
                    .at_field("date")
                    .in_field("sale"),
            );
        }
    }
    for (index, (part, cost, in_service)) in core::iter::once((
        &r.building.partial,
        r.building.cost - r.building.land,
        r.building.in_service,
    ))
    .chain(
        improvements.iter().map(|building| (&building.partial, building.cost, building.in_service)),
    )
    .enumerate()
    {
        if let Some(part) = part {
            let key = if part.cost_of_part >= cost {
                Some(("PARTIAL_DISPOSITION_PART_ABOVE_COST", "cost_of_part"))
            } else if part.date <= in_service {
                Some(("PARTIAL_DISPOSITION_BEFORE_SERVICE", "date"))
            } else {
                None
            };
            if let Some((key, field)) = key {
                let issue = ValidationIssue::new(key).at_field(field).in_field("partial");
                return Some(if index == 0 {
                    issue.in_field("building")
                } else {
                    issue.in_list("improvements", index - 1).in_field("more")
                });
            }
        }
    }
    if let crate::RentalSale::Exchanged(ref exchange) = r.sale {
        if core::iter::once(r.building.in_service)
            .chain(improvements.iter().map(|b| b.in_service))
            .any(|in_service| in_service >= exchange.transferred)
        {
            return Some(
                ValidationIssue::new("RENTAL_EXCHANGED_BEFORE_IN_SERVICE")
                    .at_field("transferred")
                    .in_field("sale"),
            );
        }
        if let Some(issue) = exchange.issue() {
            return Some(issue.in_field("sale"));
        }
    }
    match r.personal_use {
        PersonalUse::Yes { fair_rental_days, personal_days, .. }
            if fair_rental_days + personal_days > 365 =>
        {
            Some(
                ValidationIssue::new("RENTAL_DAYS_ABOVE_365")
                    .at_field("personal_days")
                    .in_field("personal_use"),
            )
        }
        PersonalUse::Yes { fair_rental_days, overlap_days, .. }
            if overlap_days > fair_rental_days =>
        {
            Some(
                ValidationIssue::new("RENTAL_OVERLAP_DAYS_ABOVE_RENTAL_DAYS")
                    .at_field("overlap_days")
                    .in_field("personal_use"),
            )
        }
        _ => None,
    }
    .or_else(|| {
        r.assets.iter().enumerate().find_map(|(index, asset)| {
            if let crate::CanonicalAssetPage::NonMacrs { facts } = asset {
                let record: crate::BusinessAsset = facts.clone().into();
                return record
                    .nonmacrs_fact_issue()
                    .or_else(|| (!record.valid()).then_some("ASSET_NONMACRS_SOURCE_FACTS"))
                    .map(|key| ValidationIssue::new(key).in_list("assets", index));
            }
            let record = asset.native();
            asset_partial_issue(
                asset_class(record.class),
                record.cost,
                record.placed,
                &record.partial,
            )
            .map(|issue| issue.in_list("assets", index))
        })
    })
}

impl Interview {
    // Saved personal casualty properties a home-use rental can be, by identity.
    pub(super) fn rental_casualty_properties(&self) -> Vec<RecordChoice> {
        self.draft
            .deductions
            .iter()
            .flat_map(|ded| &ded.personal_casualties)
            .flat_map(|event| event.properties.iter().map(move |property| (event, property)))
            .map(|(event, property)| RecordChoice {
                id: property.id,
                label: Message::new(
                    "RENTAL_SAVED_CASUALTY_PROPERTY",
                    alloc::vec![
                        MessageArg::Text(event.label.clone()),
                        MessageArg::Text(property.label.clone())
                    ],
                )
                .encode(),
            })
            .collect()
    }
    // Choices come only from accepted first-lender records, never echoed catalogs.
    pub(super) fn rental_energy_buildings(&self) -> Vec<RecordChoice> {
        crate::source_associations::SourceAssociations::new(&self.draft).physical_building_choices()
    }
}
