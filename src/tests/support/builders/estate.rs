use crate::tests::*;

pub(in crate::tests) fn estate_trust_code_c_draft(short_gain: i64, long_gain: i64) -> FederalDraft {
    let mut draft = complete(FederalStatus::Single);
    draft.income[0] = wage(1, Owner::Taxpayer, 25_000_000, 5_000_000);
    add(
        &mut draft,
        income(FederalIncome::EstateTrustPortfolio {
            data: FederalEstateTrustPortfolioK1 {
                short_gain,
                long_gain,
                ..empty_estate_portfolio()
            },
        }),
    );
    draft
}

pub(in crate::tests) fn estate_trust_code_d_data(
    draft: &mut FederalDraft,
) -> &mut FederalEstateTrustPortfolioK1 {
    let FederalIncome::EstateTrustPortfolio { data } = &mut draft.income[1].income else {
        unreachable!()
    };
    data
}

pub(in crate::tests) fn empty_estate_portfolio() -> FederalEstateTrustPortfolioK1 {
    FederalEstateTrustPortfolioK1 {
        entity_period: None,
        minimum_tax_exclusion: None,
        final_character: None,
        amt_reconciliation: None,
        investment_income: None,
        reported_credits: vec![],
        foreign_statement: None,
        investment_property: vec![],
        ein: "012345678".into(),
        interest: 0,
        ordinary_dividends: 0,
        qualified_dividends: 0,
        section199a_dividends: ReitDividends::None,
        code_i_dividends_only: false,
        short_gain: 0,
        long_gain: 0,
        rate_28_gain: 0,
        unrecaptured_gain: 0,
        royalty_income: 0,
        box_5_royalty_only: false,
        box_9_royalty_depreciation: 0,
        box_9_royalty_sound_recording_only: false,
        box_9_royalty_depletion: 0,
        box_9_royalty_cost_depletion_only: false,
        box_9_royalty_amortization: 0,
        box_9_royalty_amortization_only: false,
        box_10_interest_ird_estate_tax: 0,
        box_10_interest_ird_only: false,
        final_67e_deduction: 0,
        final_67e_no_niit_allocation: false,
        final_personal_property_tax: 0,
        final_real_estate_tax: 0,
        final_state_income_tax: 0,
        final_state_sales_tax: 0,
        final_taxes_only: false,
        final_short_carry: 0,
        final_long_carry: 0,
        final_year_successor: false,
        final_carry_same_tax_character: false,
        final_nol: None,
        amt_box12a: 0,
        amt_box12a_standalone: false,
        amt_box12b: 0,
        amt_box12_components_confirmed: false,
        amt_box12c: 0,
        amt_box12d: 0,
        amt_box12e: 0,
        amt_box12e_included_in_d: false,
        amt_box12f: 0,
        amt_box12f_included_in_d: false,
        backup_withholding: 0,
        allocated_estimated_tax: 0,
        form_1041t_timely: false,
        tax_exempt_interest: 0,
        private_activity_interest: 0,
        niit_adjustment: None,
        complete_portfolio_only: true,
        unentered: vec![],
        activities: crate::EstateActivities::empty(),
        qbi: crate::EstateQbiStatements { statements: vec![] },
        gross_farming_fishing_income: 0,
    }
}
