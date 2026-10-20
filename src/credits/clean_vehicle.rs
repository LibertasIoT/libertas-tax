// Coverage: Form 8936 and Schedule A — Supported (audited 2026-10-18).
//   Lines: Personal-use new (Part III, ≤$7,500 seller-report tentative) and used (Part IV: model
//   year, ≤$25,000 price, 3-year history) credits; MAGI line 2 with 2025 prior-year alternative;
//   Required kept/returned/resold 30-day disposition; returns nullify dealer transfers without buyer repayment.
//   dealer-transfer reconciliation and Schedule 2 lines 1b/1c repayment. Refused explicitly:
//   spouse's vehicle on separate return, dependents; no commercial-vehicle (Part V) line.
//   New vehicle business split: accepted owned-vehicle identity, exact credit-use fraction,
//   personal residual, Form3800 line1y/passive carry owner, full tentative basis correction.
//   Part II line 6 is own schedules; federal aggregation adds current K-1 line 7 for line 8.
//   Credit-use source fractions retain conversion-month, employee and lessor semantics
//   separately from property mileage. Business dealer transfers retain the reconciliation boundary.
//   Property sources are the existing owned C/F records; no standalone investment/lessor supplier.
//   Evidence: M/clean_vehicle.rs, federal_model.rs FederalCleanVehicleClaim; src/tests/credits/credits.rs
//   clean_vehicle_credits_follow_used_then_new_form_8936_limits,
//   clean_vehicle_used_price_model_year_and_transfer_are_explicit; src/tests/credits/family.rs
//   clean_vehicle_credit_precedes_mortgage_and_child_credits; IRS draft f8936, f8936sa.

use crate::kernel::lines::{LineValue, emit as emit_lines, row as calculation_row};
use crate::kernel::phase;
use crate::{
    FederalCleanVehicle, FederalCleanVehicleClaim, FederalCleanVehicleDetails, FederalPeople,
    FederalResult, FederalStatus, Owner,
};

use crate::federal_rules::valid_date;
use crate::kernel::lines::line;
use crate::kernel::ratio::{dollars, exact, fraction};
use alloc::vec::Vec;
use libertas_macros::*;

/// Saved business or farm using a new clean vehicle
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum CleanVehicleActivity {
    /// A business on Self-employment
    Business {
        /// Saved business
        business: i64,
    },
    /// A farm on Farming
    Farm {
        /// Saved farm
        farm: i64,
    },
}

/// Credit-use fraction and the saved owned vehicle supplying basis and activity identity
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CleanVehicleBusinessUse {
    /// Business or farm that owns this vehicle
    pub activity: CleanVehicleActivity,
    /// Same saved owned vehicle on that activity
    /// Select the same saved owned vehicle. Correct an actual-expense vehicle's credit basis reduction in its saved property record on Self-employment or Farming.
    pub property: i64,
    /// Business/investment use fraction for Schedule A (Form 8936) line 10
    /// From your actual use records, enter business or income-producing miles over all miles, excluding commuting. For a personal-to-business conversion or its reverse, use the business-use period's fraction multiplied by its business-use months over 12. Employee personal use included in wages or reimbursed by employees, and a vehicle you own and lease to others, count as 100%. This credit-use fraction can differ from the annual depreciation-use fraction of the selected property; do not round it to a whole percent.
    pub credit_use: crate::UseFraction,
}
impl CleanVehicleBusinessUse {
    fn fraction_valid(&self) -> bool {
        self.credit_use.total > 0 && self.credit_use.used <= self.credit_use.total
    }
    fn entry<'a>(&self, draft: &'a crate::FederalDraft) -> Option<&'a crate::FederalIncomeEntry> {
        let source = match self.activity {
            CleanVehicleActivity::Business { business } => {
                crate::CreditSource::Business { business: Some(business), reduced: true }
            }
            CleanVehicleActivity::Farm { farm } => {
                crate::CreditSource::Farm { farm: Some(farm), reduced: true }
            }
        };
        crate::general_business_credit::resolve_source(draft, &source)
    }
    fn source_valid(&self, draft: &crate::FederalDraft) -> bool {
        let Some(entry) = self.entry(draft) else {
            return false;
        };
        let sources = crate::source_associations::SourceAssociations::new(draft);
        sources.asset(entry.id, self.property).is_ok_and(|asset| asset.is_vehicle())
            || sources.mileage(entry.id, self.property).is_ok_and(|row| row.owned && row.total > 0)
    }
}

pub(crate) fn property_choices(draft: &crate::FederalDraft) -> Vec<crate::RecordChoice> {
    draft
        .income
        .iter()
        .flat_map(|entry| {
            let assets = match &entry.income {
                crate::FederalIncome::Business { data } => data.assets.as_slice(),
                crate::FederalIncome::FarmRental { data } => data.assets.as_slice(),
                _ => &[],
            };
            assets
                .iter()
                .filter(|asset| asset.is_vehicle())
                .map(|asset| asset.id)
                .chain(
                    crate::BusinessMileage::records(&entry.income)
                        .iter()
                        .filter(|row| row.owned && row.total > 0)
                        .map(|row| row.id),
                )
                .filter(|id| *id > 0)
                .map(|id| crate::RecordChoice {
                    id,
                    label: crate::Message::new(
                        "CLEAN_VEHICLE_PROPERTY_CHOICE",
                        alloc::vec![
                            crate::MessageArg::Text(entry.label.clone()),
                            crate::MessageArg::Number(id)
                        ],
                    )
                    .encode(),
                })
        })
        .collect()
}

pub(crate) fn source_issue(
    draft: &crate::FederalDraft,
    claim: &FederalCleanVehicleClaim,
) -> Option<crate::validation::ValidationIssue> {
    claim.vehicles.iter().enumerate().find_map(|(index, vehicle)| {
        if let FederalCleanVehicle::New { business_use: Some(use_), details, .. } = vehicle {
            if !use_.fraction_valid() {
                return Some(crate::validation::ValidationIssue::new("CLEAN_VEHICLE_USE_FRACTION_INVALID")
                    .at_field(if use_.credit_use.total == 0 { "total" } else { "used" }).in_field("credit_use").in_field("business_use").in_list("vehicles", index));
            }
            let source = use_.entry(draft);
            if !use_.source_valid(draft) || source.is_none_or(|entry| entry.owner != details.buyer) {
                return Some(crate::validation::ValidationIssue::new("CLEAN_VEHICLE_PROPERTY_INVALID")
                    .at_field("property").in_field("business_use").in_list("vehicles", index));
            }
            if claim.vehicles[..index].iter().any(|other| matches!(other,
                FederalCleanVehicle::New { business_use: Some(other), .. } if other.property == use_.property)) {
                return Some(crate::validation::ValidationIssue::new("CLEAN_VEHICLE_PROPERTY_DUPLICATE")
                    .at_field("property").in_field("business_use").in_list("vehicles", index));
            }
        }
        None
    })
}

// The acquisition deadline in the issue key is the statutory credit termination date.
const FORM: &str = "https://www.irs.gov/pub/irs-dft/f8936--dft.pdf";
const SCHEDULE: &str = "https://www.irs.gov/pub/irs-dft/f8936sa--dft.pdf";
const MAX_CENTS: i64 = 100_000_000_000;

#[derive(Default)]
pub(crate) struct PersonalCredits {
    pub new: i64,
    pub used: i64,
    pub new_repayment: i64,
    pub used_repayment: i64,
    pub business: Vec<crate::general_business_credit::CreditEntry>,
    // Runtime projection of the existing final repayment, in whole dollars.
    pub repayment_by_owner: [i64; 2],
}

impl FederalCleanVehicle {
    fn details(&self) -> &FederalCleanVehicleDetails {
        match self {
            Self::New { details, .. } | Self::PreviouslyOwned { details, .. } => details,
        }
    }
}

pub(crate) fn valid(claim: &FederalCleanVehicleClaim) -> bool {
    if !(0..=MAX_CENTS).contains(&claim.current_territory_addbacks)
        || claim.prior_year.as_ref().is_some_and(|prior| !(0..=MAX_CENTS).contains(&prior.magi))
    {
        return false;
    }
    for (index, vehicle) in claim.vehicles.iter().enumerate() {
        let details = vehicle.details();
        if details.vin.len() != 17
            || !details.vin.bytes().all(|byte| {
                byte.is_ascii_digit()
                    || (byte.is_ascii_alphabetic()
                        && !matches!(byte.to_ascii_uppercase(), b'I' | b'O' | b'Q'))
            })
            || !valid_date(details.acquired)
            || !valid_date(details.placed_in_service)
            || details.dealer_transfer.is_some_and(|amount| !(1..=750_000).contains(&amount))
            || claim.vehicles[..index]
                .iter()
                .any(|earlier| earlier.details().vin.eq_ignore_ascii_case(&details.vin))
        {
            return false;
        }
        match vehicle {
            FederalCleanVehicle::New { seller_credit, business_use, .. } => {
                if !(0..=750_000).contains(seller_credit)
                    || business_use.as_ref().is_some_and(|use_| !use_.fraction_valid())
                {
                    return false;
                }
            }
            FederalCleanVehicle::PreviouslyOwned { model_year, sales_price, .. } => {
                if !(1900..=2023).contains(model_year) || !(0..=MAX_CENTS).contains(sales_price) {
                    return false;
                }
            }
        }
    }
    true
}

fn threshold(status: FederalStatus, used: bool) -> i64 {
    match (status, used) {
        (FederalStatus::Joint | FederalStatus::Surviving, false) => 300_000,
        (FederalStatus::Head, false) => 225_000,
        (_, false) => 150_000,
        (FederalStatus::Joint | FederalStatus::Surviving, true) => 150_000,
        (FederalStatus::Head, true) => 112_500,
        (_, true) => 75_000,
    }
}

// Section 30D(f)(10), shared by personal credits and Form 3800 line 1y.
pub(crate) fn magi_eligible(status: FederalStatus, magi: i64, used: bool) -> bool {
    magi <= threshold(status, used)
}

pub(crate) fn calculate(
    draft: &crate::FederalDraft,
    claim: Option<&FederalCleanVehicleClaim>,
    people: &FederalPeople,
    status: FederalStatus,
    current_magi: i64,
    result: &mut FederalResult,
) -> PersonalCredits {
    let Some(claim) = claim else {
        return PersonalCredits::default();
    };
    if !claim.vehicles.is_empty() {
        result.review_notes.push(
            crate::federal_model::ReviewNote::localized(#[cfg(test)] "Form 8936 and Schedule A are 2026 drafts. The personal-credit tax-limit order follows the published 2025 instructions; reconcile with final 2026 instructions before filing.", crate::Message::new("NOTE_FORM_8936_AND_SCHEDULE_A_ARE_CURRENT_YEAR_DRAFTS_THE_PERSONAL_CREDIT_TAX_E83F9C5B", alloc::vec![])),
        );
    }
    let current_magi = current_magi + dollars(claim.current_territory_addbacks);
    line(
        result,
        "clean_vehicle_modified_agi_current_year",
        current_magi,
        &["agi", "income.foreign", "credits.clean_vehicles"],
        FORM,
        msg!("EXPLAIN_CLEAN_VEHICLE_MODIFIED_AGI_N"),
    );
    if let Some(prior) = &claim.prior_year {
        line(
            result,
            "clean_vehicle_modified_agi_prior_year",
            dollars(prior.magi),
            &["credits.clean_vehicles.prior_year"],
            FORM,
            msg!("EXPLAIN_CLEAN_VEHICLE_MODIFIED_AGI_N_2"),
        );
    }
    let mut credits = PersonalCredits::default();
    let mut used_taxpayer = false;
    let mut used_spouse = false;
    for (index, vehicle) in claim.vehicles.iter().enumerate() {
        let details = vehicle.details();
        #[cfg(test)]
        let section = alloc::format!("Clean vehicle {}", details.vin);
        let used = matches!(vehicle, FederalCleanVehicle::PreviouslyOwned { .. });
        let current_eligible = magi_eligible(status, current_magi, used);
        let prior_eligible = claim
            .prior_year
            .as_ref()
            .is_some_and(|prior| magi_eligible(prior.status, dollars(prior.magi), used));
        if !(20230101..=20250930).contains(&details.acquired)
            || !(20260101..=20261231).contains(&details.placed_in_service)
            || details.acquired > details.placed_in_service
        {
            crate::issue_localized!(
                result,
                &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
                &section,
                "ISSUE_THE_BINDING_CONTRACT_AND_PAYMENT_MUST_PRECEDE_OCTOBER_1_2025_AND_PO_255C3C91",
                "The binding contract and payment must precede October 1, 2025, and possession must occur in 2026.",
            );
        }
        let buyer = match details.buyer {
            Owner::Taxpayer => &people.taxpayer,
            Owner::Spouse => {
                if status != FederalStatus::Joint {
                    crate::issue_localized!(
                        result,
                        &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
                        &section,
                        "ISSUE_A_SPOUSE_S_VEHICLE_BELONGS_ON_A_JOINT_RETURN_OR_THAT_SPOUSE_S_OWN_R_E75DC3CE",
                        "A spouse's vehicle belongs on a joint return or that spouse's own return.",
                    );
                    continue;
                }
                let Some(spouse) = people.spouse.as_ref() else {
                    crate::issue_localized!(
                        result,
                        &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
                        &section,
                        "ISSUE_THE_NAMED_BUYER_IS_NOT_ON_THIS_RETURN_EDEA8376",
                        "The named buyer is not on this return.",
                    );
                    continue;
                };
                spouse
            }
        };
        let eligible_facts = details.accepted_seller_report
            && details.qualified_vehicle
            && details.owned_for_use
            && details.disposition == crate::CleanVehicleDisposition::Kept
            && (20230101..=20250930).contains(&details.acquired)
            && (20260101..=20261231).contains(&details.placed_in_service)
            && details.acquired <= details.placed_in_service;
        if !current_eligible
            && claim.prior_year.is_none()
            && details.accepted_seller_report
            && details.qualified_vehicle
            && details.owned_for_use
            && details.disposition == crate::CleanVehicleDisposition::Kept
        {
            crate::issue_localized!(
                result,
                &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
                &section,
                "ISSUE_ENTER_PRIOR_YEAR_MODIFIED_AGI_AND_FILING_STATUS_TO_TEST_THE_PRIOR_YEAR_AL_7E22D52B",
                "Enter 2025 modified AGI and filing status to test the prior-year alternative.",
            );
        }
        let magi_eligible = current_eligible || prior_eligible;
        let amount = match vehicle {
            FederalCleanVehicle::New { seller_credit, .. } => {
                if eligible_facts && magi_eligible {
                    dollars(*seller_credit)
                } else {
                    0
                }
            }
            FederalCleanVehicle::PreviouslyOwned {
                model_year,
                sales_price,
                first_eligible_transfer,
                no_credit_in_three_years,
                ..
            } => {
                let prior_used = match details.buyer {
                    Owner::Taxpayer => &mut used_taxpayer,
                    Owner::Spouse => &mut used_spouse,
                };
                let another_used = *prior_used;
                if *model_year + 2 > details.acquired / 10_000 || *sales_price > 2_500_000 {
                    crate::issue_localized!(
                        result,
                        &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
                        &section,
                        "ISSUE_A_USED_VEHICLE_MUST_BE_AT_LEAST_TWO_MODEL_YEARS_OLDER_THAN_THE_ACQU_6460EF4D",
                        "A used vehicle must be at least two model years older than the acquisition year and have a sales price no greater than $25,000.",
                    );
                }
                let candidate = if eligible_facts
                    && magi_eligible
                    && *model_year + 2 <= details.acquired / 10_000
                    && *sales_price <= 2_500_000
                    && *first_eligible_transfer
                    && *no_credit_in_three_years
                    && !buyer.dependency.claimable()
                {
                    phase::cap(fraction(dollars(*sales_price), 30, 100), [4000])
                } else {
                    0
                };
                if candidate > 0 && another_used {
                    crate::issue_localized!(
                        result,
                        &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
                        &section,
                        "ISSUE_ONLY_ONE_PREVIOUSLY_OWNED_VEHICLE_CREDIT_PER_BUYER_MAY_BE_SELECTED_9C05777E",
                        "Only one previously owned vehicle credit per buyer may be selected within this three-year window.",
                    );
                    0
                } else {
                    if candidate > 0 {
                        *prior_used = true;
                    }
                    candidate
                }
            }
        };
        let transferred = details.dealer_transfer.map_or(0, dollars);
        // 26 CFR 1.30D-4(e)(1)(ii), 1.25E-2(c)(1)(ii): the return
        // nullifies the election, so neither transfer reconciliation nor buyer repayment applies.
        let returned = details.disposition == crate::CleanVehicleDisposition::ReturnedWithin30Days;
        if returned {
            crate::federal_rules::note_once_scoped(
                result,
                crate::Message::new(
                    "NOTE_CLEAN_VEHICLE_RETURNED_DEALER_RECOVERS_TRANSFER",
                    alloc::vec![crate::MessageArg::Text(details.vin.clone())],
                ),
                crate::IssueContext::owner(crate::TaxSection::MoreCredits, details.buyer)
                    .item("clean-vehicles", index),
            );
        }
        if transferred > 0 && !returned {
            if !details.accepted_seller_report
                || !details.qualified_vehicle
                || !details.owned_for_use
                // Acquisition-time predominantly-personal intent is not established by
                // annual mileage. Keep this slice at the existing reconciliation boundary.
                || matches!(vehicle, FederalCleanVehicle::New { business_use: Some(_), .. })
                || !(20230101..=20250930).contains(&details.acquired)
                || !(20260101..=20261231).contains(&details.placed_in_service)
                || details.acquired > details.placed_in_service
            {
                crate::issue_localized!(
                    result,
                    &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
                    &section,
                    "ISSUE_RECONCILE_THE_DEALER_TRANSFER_AGAINST_THE_ACCEPTED_REPORT_VEHICLE_Q_E6F60241",
                    "Reconcile the dealer transfer against the accepted report, vehicle qualification, buyer and 2026 possession before calculating repayment.",
                );
            }
            if magi_eligible
                && details.disposition == crate::CleanVehicleDisposition::Kept
                && transferred != amount
            {
                crate::issue_localized!(
                    result,
                    &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
                    &section,
                    "ISSUE_THE_TRANSFERRED_AMOUNT_MUST_EQUAL_THE_VEHICLE_CREDIT_ON_THE_ACCEPTE_7A758C4B",
                    "The transferred amount must equal the vehicle credit on the accepted seller report; reconcile the dealer report.",
                );
            }
            if !magi_eligible
                || details.disposition == crate::CleanVehicleDisposition::ResoldWithin30Days
            {
                credits.repayment_by_owner[usize::from(details.buyer == crate::Owner::Spouse)] +=
                    transferred;
                if used {
                    credits.used_repayment += transferred;
                } else {
                    credits.new_repayment += transferred;
                }
            }
        }
        let mut personal_amount = amount;
        if let FederalCleanVehicle::New { business_use: Some(use_), seller_credit, .. } = vehicle {
            if let Some(entry) =
                use_.entry(draft).filter(|_| use_.source_valid(draft) && use_.fraction_valid())
            {
                let total_cents = exact::whole(*seller_credit);
                let business_cents = &total_cents
                    * exact::ratio(
                        i64::from(use_.credit_use.used),
                        i64::from(use_.credit_use.total),
                    )
                    .expect("validated positive source denominator");
                // Schedule A line 11 rounds first; line 12 is the exact residual of line 9.
                // The approved line10 source fraction remains independent of annual property use.
                let parts = exact::project_rounded_pair(
                    dollars(*seller_credit),
                    &total_cents,
                    [business_cents.clone(), &total_cents - business_cents],
                    0,
                )
                .expect("native vehicle source cents foot to the seller credit");
                let business_amount = if amount > 0 && transferred == 0 { parts[0] } else { 0 };
                personal_amount = if amount > 0 { parts[1] } else { 0 };
                line(
                    result,
                    &alloc::format!("clean_vehicle_{}_business_part", index + 1),
                    business_amount,
                    &["credits.clean_vehicles"],
                    SCHEDULE,
                    crate::Message::new(
                        "EXPLAIN_CLEAN_VEHICLE_BUSINESS_PART",
                        alloc::vec![crate::MessageArg::Text(details.vin.clone())],
                    ),
                );
                if business_amount > 0 {
                    credits.business.push(crate::general_business_credit::CreditEntry {
                        conditional_origin: None,
                        credit: crate::BusinessCredit::CleanVehicle,
                        year: 2026,
                        amount: business_amount,
                        passive: match &entry.income {
                            crate::FederalIncome::Business { data }
                                if data.material_participation =>
                            {
                                None
                            }
                            _ => Some(crate::PassiveWorksheet::Other),
                        },
                        transfer: 0,
                        elective: 0,
                        activity: Some(entry.id),
                        source_owner: Some(entry.owner),
                        filer: crate::CarryFiler::SameFilers,
                    });
                }
                // The whole tentative credit reduces basis, including a dealer transfer.
                let expected = if eligible_facts && magi_eligible { *seller_credit } else { 0 };
                if let Ok(asset) = crate::source_associations::SourceAssociations::new(draft)
                    .asset(entry.id, use_.property)
                    && asset.credit_reduction != expected
                {
                    let assets = match &entry.income {
                        crate::FederalIncome::Business { data } => data.assets.as_slice(),
                        crate::FederalIncome::FarmRental { data } => data.assets.as_slice(),
                        _ => &[],
                    };
                    if let Some(position) = assets.iter().position(|row| row.id == asset.id) {
                        crate::federal_rules::note_once_scoped(
                            result,
                            crate::Message::new(
                                "WARN_CLEAN_VEHICLE_BASIS_REDUCTION",
                                alloc::vec![
                                    crate::MessageArg::Text(details.vin.clone()),
                                    crate::MessageArg::usd_cents(expected)
                                ],
                            ),
                            crate::IssueContext::income_id(entry.id)
                                .item("business_assets", position),
                        );
                    }
                }
            } else {
                personal_amount = 0;
                crate::federal_rules::note_once_scoped(
                    result,
                    crate::Message::new(
                        "WARN_CLEAN_VEHICLE_PROPERTY_UNAVAILABLE",
                        alloc::vec![crate::MessageArg::Text(details.vin.clone())],
                    ),
                    crate::IssueContext::Section(crate::TaxSection::MoreCredits)
                        .item("clean-vehicles", index),
                );
            }
        }
        line(
            result,
            &alloc::format!("clean_vehicle_{}_potential", index + 1),
            if transferred > 0 { 0 } else { personal_amount },
            &["credits.clean_vehicles", "clean_vehicle_modified_agi_current_year"],
            SCHEDULE,
            crate::Message::new(
                "EXPLAIN_CLEAN_VEHICLE_N_POTENTIAL",
                alloc::vec![crate::MessageArg::Text(details.vin.clone())],
            ),
        );
        if transferred == 0 {
            if used {
                credits.used += amount;
            } else {
                credits.new += personal_amount;
            }
        }
    }
    if claim
        .vehicles
        .iter()
        .any(|vehicle| matches!(vehicle, FederalCleanVehicle::New { business_use: Some(_), .. }))
    {
        crate::kernel::lines::form_line(
            result,
            crate::FormLine::new(crate::TaxForm::Form8936, "6"),
            "form_8936_own_business_part",
            credits.business.iter().map(|entry| entry.amount).sum(),
            &["credits.clean_vehicles"],
            FORM,
            msg!("EXPLAIN_CLEAN_VEHICLE_OWN_BUSINESS_TOTAL"),
        );
    }
    emit_lines(
        result,
        &mut (),
        [
            (
                "new_clean_vehicle_transfer_repayment",
                credits.new_repayment,
                msg!("EXPLAIN_NEW_CLEAN_VEHICLE_TRANSFER_REPAYMENT"),
            ),
            (
                "previously_owned_clean_vehicle_transfer_repayment",
                credits.used_repayment,
                msg!("EXPLAIN_PREVIOUSLY_OWNED_CLEAN_VEHICLE_TRANSFER_REPAYMENT"),
            ),
        ],
        |(id, amount, explanation), _, _| {
            Some(calculation_row(
                None,
                id,
                LineValue::MoneyDollars(amount),
                &["credits.clean_vehicles", "clean_vehicle_modified_agi_current_year"],
                "https://www.irs.gov/pub/irs-dft/f1040s2--dft.pdf",
                explanation,
            ))
        },
    );
    credits
}
