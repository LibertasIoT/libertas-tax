use crate::tests::*;

pub(in crate::tests) fn broker_b_paper(draft: &FederalDraft) -> Form1099B {
    Form1099B {
        year: u32::try_from(draft.tax_year).unwrap(),
        payer: PrintedParty {
            name: "Synthetic Clearing, Inc.".into(),
            federal_id: "00-0123456".into(),
            address: "10 Example Road\nExample City, NJ 07001".into(),
        },
        payer_telephone: "800-555-0100".into(),
        recipient: PrintedParty {
            federal_id: "***-**-0042".into(),
            address: "20 Example Street\nExample City, NJ 07001".into(),
            ..super::fixture_document_names(draft, -1).unwrap().recipient
        },
        account_number: "00001234-A".into(),
        cusip_number: "001234567".into(),
        fatca_filing_requirement: true,
        form_8949_checkbox: Some(Form1099B8949Checkbox::D),
        box_1a_description: "0.125 Example shares".into(),
        box_1a_quantity: Some(0.125),
        box_1b_acquired: Some(BrokerAcquisitionDate::Date { date: 20230301 }),
        box_1c_sold: Some(20260630),
        box_1d_proceeds: Some(450_000),
        box_1e_basis: Some(150_000),
        box_1f_market_discount: Some(0),
        box_1g_wash_sale: None,
        box_2_long_term: true,
        box_4_withholding: Some(12_345),
        box_6_gross_proceeds: true,
        box_12_basis_reported: true,
        state_details: StateWithholdingFields {
            states: vec![StateTaxWithholding {
                state: TaxState::NJ,
                payer_state_id: "00007-X".into(),
                withholding: Some(3_456),
            }],
        },
        ..Default::default()
    }
}

pub(in crate::tests) fn broker_da_paper(draft: &FederalDraft) -> Form1099DA {
    let stock = broker_b_paper(draft);
    Form1099DA {
        year: u32::try_from(draft.tax_year).unwrap(),
        filer: stock.payer,
        filer_telephone: stock.payer_telephone,
        recipient: stock.recipient,
        account_number: stock.account_number,
        cusip_number: stock.cusip_number,
        form_8949_checkbox: Some(Form1099DA8949Checkbox::J),
        box_1a_asset_code: "000001".into(),
        box_1b_asset_name: "Example fractional token".into(),
        box_1c_units: Some(0.000_125),
        box_1d_acquired: Some(BrokerAcquisitionDate::Various),
        box_1e_sold: stock.box_1c_sold,
        box_1f_proceeds: stock.box_1d_proceeds,
        box_1g_basis: stock.box_1e_basis,
        box_1h_market_discount: Some(0),
        box_1i_wash_sale: Some(0),
        box_2_basis_reported: true,
        box_3a_gross_proceeds: true,
        box_4_withholding: stock.box_4_withholding.unwrap_or(0),
        box_6_long_term: true,
        box_7_cash_only: true,
        box_8_customer_acquisition: true,
        box_9_noncovered: true,
        box_11a_stablecoins: true,
        box_11b_transactions: Some(2),
        box_12a_transferred_units: Some(0.000_125),
        box_12b_transfer_date: None,
        state_details: stock.state_details,
        ..Default::default()
    }
}

pub(in crate::tests) fn broker_manual_control(draft: &FederalDraft) -> FederalDraft {
    let mut control = draft.clone();
    add(
        &mut control,
        income(FederalIncome::Sale {
            data: sale_source(FederalSale {
                withholding: 12_345,
                ..native_sale(GainTerm::Long, 450_000, Some(150_000))
            }),
        }),
    );
    add(
        &mut control,
        income(FederalIncome::Section1256 {
            data: contracts_source(FederalSection1256 {
                aggregate: 50_000,
                withholding: 0,
                special_treatment: false,
            }),
        }),
    );
    control
}

pub(in crate::tests) fn broker_community_return() -> FederalDraft {
    let mut draft = complete(FederalStatus::Separate);
    draft.setup.as_mut().unwrap().community_property = super::community_arizona();
    draft.deductions.as_mut().unwrap().itemize_override = None;
    for entry in &mut draft.income {
        entry.community = CommunityShare::Community;
    }
    draft
}
