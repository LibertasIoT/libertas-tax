// form_1040_es items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) struct Form1040EsIn {
    pub(super) reported_credit_cents: i64,
    pub(super) agi: i64,
    pub(super) deduction: i64,
    pub(super) qbi_ded: i64,
    pub(super) senior_ded: i64,
    pub(super) additional: [i64; 3],
    pub(super) regular_tax_pref: i64,
    pub(super) regular_collectible: i64,
    pub(super) regular_unrecaptured: i64,
    pub(super) parent_rate_required: bool,
    pub(super) excluded: i64,
    pub(super) regular_tax: i64,
    pub(super) unaveraged_regular_tax: i64,
    pub(super) amt: i64,
    pub(super) bba_additional_tax: i64,
    pub(super) premium_repayment: i64,
    pub(super) clean_vehicle_repayment: i64,
    pub(super) credits: i64,
    pub(super) residential_energy_credit: i64,
    pub(super) personal_refueling_credit: i64,
    pub(super) used_clean_vehicle_credit: i64,
    pub(super) new_clean_vehicle_credit: i64,
    pub(super) legacy_ev_credit: i64,
    pub(super) se_tax: i64,
    pub(super) line_2_other: i64,
    pub(super) education_recapture_tax: i64,
    pub(super) rmd_tax: i64,
    pub(super) hsa_tax: i64,
    pub(super) hsa_chapter_one_tax: i64,
    pub(super) archer_excess_tax: i64,
    pub(super) able_excess_tax: i64,
    pub(super) coverdell_excess_tax: i64,
    pub(super) trump_excess_tax: i64,
    pub(super) ira_excess_tax: i64,
    pub(super) household_tax: i64,
    pub(super) refundable: i64,
    pub(super) fuel_credit: i64,
    pub(super) income_tax: i64,
    pub(super) withholding_25d: i64,
    pub(super) farming_return_year: Result<bool, &'static str>,
}

pub(super) fn form_1040_es_phase(pass: &Pass, r: &mut FederalResult, i: Form1040EsIn) {
    let &Pass { s, p, pay, .. } = pass;
    let Form1040EsIn {
        reported_credit_cents,
        agi,
        deduction,
        qbi_ded,
        senior_ded,
        additional,
        regular_tax_pref,
        regular_collectible,
        regular_unrecaptured,
        parent_rate_required,
        excluded,
        regular_tax,
        unaveraged_regular_tax,
        amt,
        bba_additional_tax,
        premium_repayment,
        clean_vehicle_repayment,
        credits,
        residential_energy_credit,
        personal_refueling_credit,
        used_clean_vehicle_credit,
        new_clean_vehicle_credit,
        legacy_ev_credit,
        se_tax,
        line_2_other,
        education_recapture_tax,
        rmd_tax,
        hsa_tax,
        hsa_chapter_one_tax,
        archer_excess_tax,
        able_excess_tax,
        coverdell_excess_tax,
        trump_excess_tax,
        ira_excess_tax,
        household_tax,
        refundable,
        fuel_credit,
        income_tax,
        withholding_25d,
        farming_return_year,
    } = i;
    // Form 1040-ES: the 2027 Estimated Tax Worksheet from this return,
    // when the payments page asks for it.
    crate::forms::form_1040_es::compute(
        &crate::forms::form_1040_es::Form1040EsContext {
            request: &NextYearEstimate::from(pay.next_year.clone()),
            applied_credit_cents: if s.filing_status() == FederalStatus::Joint
                || pay.credit_recipient == Some(crate::OverpaymentRecipient::TaxpayerAccount)
            {
                reported_credit_cents
            } else {
                0
            },
            status: s.filing_status(),
            agi,
            deduction,
            qbi: qbi_ded,
            additional: senior_ded + additional.iter().sum::<i64>(),
            preference: [regular_tax_pref, regular_collectible, regular_unrecaptured],
            plain_tax: !parent_rate_required
                && excluded == 0
                && regular_tax == unaveraged_regular_tax,
            amt,
            line_16_other: bba_additional_tax,
            part_one_repayments: premium_repayment + clean_vehicle_repayment,
            credits,
            expired_credits: residential_energy_credit
                + personal_refueling_credit
                + used_clean_vehicle_credit
                + new_clean_vehicle_credit
                + legacy_ev_credit,
            se_tax,
            other_taxes: line_2_other
                + education_recapture_tax
                + rmd_tax
                + (hsa_tax - hsa_chapter_one_tax)
                + archer_excess_tax
                + able_excess_tax
                + coverdell_excess_tax
                + trump_excess_tax
                + ira_excess_tax,
            household_tax,
            refundable_credits: refundable + fuel_credit,
            this_year_tax: positive(
                positive(income_tax - credits) + se_tax + line_2_other + household_tax
                    - refundable
                    - fuel_credit,
            ),
            withholding: withholding_25d,
            farming_return_year,
            deceased: p.taxpayer.died.is_some()
                || p.spouse.as_ref().is_some_and(|v| v.died.is_some()),
            annualized: pay.annualized,
        },
        r,
    );
}
