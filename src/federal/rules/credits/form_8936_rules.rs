// form_8936 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) struct Form8936In<'a> {
    pub(super) clean_vehicle: &'a crate::clean_vehicle::PersonalCredits,
    pub(super) creditable_tax: i64,
    pub(super) care_used: i64,
    pub(super) elderly: i64,
    pub(super) education_used: i64,
    pub(super) saver: i64,
}

pub(super) struct Form8936Out {
    pub(super) used_clean_vehicle_credit: i64,
    pub(super) new_clean_vehicle_credit: i64,
}

pub(super) fn form_8936_phase(
    _pass: &Pass,
    r: &mut FederalResult,
    i: Form8936In<'_>,
) -> Form8936Out {
    let Form8936In { clean_vehicle, creditable_tax, care_used, elderly, education_used, saver } = i;
    // Form 8936 line 16 omits the new credit, while line 11 includes the
    // previously owned credit. Both follow Schedule 3's earlier personal credits.
    let tax_before_clean = positive(creditable_tax - care_used - elderly - education_used - saver);
    let used_clean_vehicle_credit = phase::cap(clean_vehicle.used, [tax_before_clean]);
    let new_clean_vehicle_credit =
        phase::cap(clean_vehicle.new, [tax_before_clean - used_clean_vehicle_credit]);
    emit_lines(
        r,
        &mut (),
        [
            (
                "previously_owned_clean_vehicle_credit",
                used_clean_vehicle_credit,
                msg!("EXPLAIN_PREVIOUSLY_OWNED_CLEAN_VEHICLE_CREDIT"),
            ),
            (
                "new_clean_vehicle_credit",
                new_clean_vehicle_credit,
                msg!("EXPLAIN_NEW_CLEAN_VEHICLE_CREDIT"),
            ),
            (
                "previously_owned_clean_vehicle_credit_unusable",
                clean_vehicle.used - used_clean_vehicle_credit,
                msg!("EXPLAIN_PREVIOUSLY_OWNED_CLEAN_VEHICLE_CREDIT_UNUSABLE"),
            ),
            (
                "new_clean_vehicle_credit_unusable",
                clean_vehicle.new - new_clean_vehicle_credit,
                msg!("EXPLAIN_NEW_CLEAN_VEHICLE_CREDIT_UNUSABLE"),
            ),
        ],
        |(id, amount, explanation), _, _| {
            Some(calculation_row(
                None,
                id,
                LineValue::MoneyDollars(amount),
                &["credits.clean_vehicles", "income_tax", "credits"],
                "https://www.irs.gov/pub/irs-dft/f8936--dft.pdf",
                explanation,
            ))
        },
    );
    Form8936Out { used_clean_vehicle_credit, new_clean_vehicle_credit }
}
