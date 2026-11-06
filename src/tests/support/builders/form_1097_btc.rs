use crate::tests::*;

fn btc_paper_native(form: Form1097Btc) -> Form1097BtcPaper {
    let Form1097Btc { issuer, recipient, unique_identifier, box3_code_199, box1_credit } = form;
    Form1097BtcPaper {
        issuer,
        recipient,
        unique_identifier: Some(unique_identifier),
        box1_credit: Some(box1_credit),
        bond_type: box3_code_199.then_some(Form1097BtcBondType::Other),
        calendar_year: Some(2026),
        ..Default::default()
    }
}

paper_adapter!(btc_paper, btc_paper_native, BondCreditBoxFixture, Form1097Btc, Form1097BtcPaper);
