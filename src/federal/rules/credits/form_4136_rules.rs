// form_4136 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) struct Form4136FuelOut {
    pub(super) fuel_credit_cents: i64,
    pub(super) fuel_credit: i64,
    pub(super) injured_fuel_credit_sources:
        Option<Box<crate::injured_spouse::income_categories::NativeFuelCreditSources>>,
}

pub(super) fn form_4136_fuel_phase(pass: &Pass) -> Form4136FuelOut {
    let &Pass { d, allocated_entries, pay, .. } = pass;
    // Retain the one existing native fuel collection before source attribution.
    // Later reporting and payments reuse these unchanged scalar amounts.
    let home_fuel_credit_cents =
        pay.home_kerosene.as_ref().map_or(0, crate::FederalHomeKerosene::allowed_cents);
    let mut business_fuel_credit_cents = Some([0_i64; 2]);
    let fuel_credit_cents = allocated_entries
        .iter()
        .map(|entry| {
            let amount = match &entry.income {
                FederalIncome::Business { data } => data
                    .fuel_tax_credit
                    .as_ref()
                    .map_or(0, crate::FederalFuelTaxCredit::credit_cents),
                FederalIncome::Partnership { data } => crate::community::income_amount(
                    d,
                    entry.id,
                    data.fuel_tax_credit
                        .as_ref()
                        .map_or(0, crate::PartnerFuelTaxCredit::credit_cents),
                ),
                _ => 0,
            };
            business_fuel_credit_cents = business_fuel_credit_cents.and_then(|mut owners| {
                if amount < 0 {
                    return None;
                }
                let owner = usize::from(entry.owner == crate::DocumentOwner::Spouses);
                owners[owner] = owners[owner].checked_add(amount)?;
                Some(owners)
            });
            amount
        })
        .sum::<i64>()
        + home_fuel_credit_cents;
    let fuel_credit = dollars(fuel_credit_cents);

    let injured_fuel_credit_sources = business_fuel_credit_cents
        .filter(|owners| {
            home_fuel_credit_cents >= 0
                && owners[0]
                    .checked_add(owners[1])
                    .and_then(|amount| amount.checked_add(home_fuel_credit_cents))
                    == Some(fuel_credit_cents)
        })
        .map(|business_cents| {
            Box::new(crate::injured_spouse::income_categories::NativeFuelCreditSources {
                joint_cents: fuel_credit_cents,
                business_cents,
                home_cents: home_fuel_credit_cents,
            })
        });
    Form4136FuelOut { fuel_credit_cents, fuel_credit, injured_fuel_credit_sources }
}

pub(super) fn form_4136_output_phase(pass: &Pass, r: &mut FederalResult, fuel_credit_cents: i64) {
    let &Pass { pay, .. } = pass;
    if let Some(home) = &pay.home_kerosene {
        // Fuel not qualifying or claimed elsewhere takes no credit; an
        // incomplete record is left out with a notice.
        if !home.records_complete {
            note_once_scoped(
                r,
                msg!("WARN_HOME_KEROSENE_LEFT_OUT"),
                crate::IssueContext::Section(crate::TaxSection::MoreCredits),
            );
        }
        line_cents(
            r,
            "home_kerosene_fuel_4a",
            home.allowed_cents(),
            &["payments.home_kerosene"],
            "https://www.irs.gov/instructions/i4136",
            msg!("EXPLAIN_HOME_KEROSENE_FUEL_4A"),
        );
    }

    line_cents(
        r,
        "fuel_tax_credit",
        fuel_credit_cents,
        &[
            "income.business.fuel_tax_credit",
            "income.partnership.fuel_tax_credit",
            "payments.home_kerosene",
        ],
        "https://www.irs.gov/pub/irs-dft/f4136--dft.pdf",
        msg!("EXPLAIN_FUEL_TAX_CREDIT"),
    );
}
