use super::*;
use crate::tests::cross::interview::accepted;
use crate::tests::foreign::foreign::{k3_blank, passive_fund_return};
use crate::tests::support::client::income_pages::open_document_center;

#[test]
fn section1256_independent_basis_and_risk_feed_existing_amt_capital_owner() {
    for (basis, risk) in
        [([1_000_000, 500_000], [2_000_000; 2]), ([2_000_000; 2], [1_000_000, 500_000])]
    {
        for source in k1_section1256_sources(&[-1_000_000], basis, risk) {
            let mut d = complete(FederalStatus::Single);
            add(&mut d, wage(0, Owner::Taxpayer, 20_000_000, 0));
            add(&mut d, income(source));
            let regular = crate::section1256::form_6781(&d);
            let alternative = crate::section1256::form_6781_amt(&d);
            assert_eq!((regular.short, regular.long), (-4_000, -6_000));
            assert_eq!((alternative.short, alternative.long), (-2_000, -3_000));
            let r = supported(&d);
            // Each system deducts $3,000; its own remaining capital loss carries.
            assert_eq!(value(&r, "capital_loss_carryforward"), 7_000);
            assert_eq!(value(&r, "amt_capital_loss_carryforward"), 2_000);
        }
    }
}

#[test]
fn section1256_amt_rounds_each_entity_before_return_netting_and_split() {
    for source in k1_section1256_sources(&[-10_049], [10_049, 5_050], [100_000; 2]) {
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(source));
        add(
            &mut d,
            income(FederalIncome::Section1256 {
                data: contracts_source(crate::FederalSection1256 {
                    aggregate: 2_050,
                    withholding: 0,
                    special_treatment: false,
                }),
            }),
        );
        // Source rows -100/+21 and -51/+21, then one return-wide split.
        let regular = crate::section1256::form_6781(&d);
        let alternative = crate::section1256::form_6781_amt(&d);
        assert_eq!((regular.net, regular.short, regular.long), (-79, -32, -47));
        assert_eq!((alternative.net, alternative.short, alternative.long), (-30, -12, -18));
    }
}

#[test]
fn section1256_positive_rows_cannot_hide_unmapped_negative_code_allowances() {
    for contract in [-40_000, 100_000] {
        for mut source in k1_section1256_sources(&[contract], [30_000, 20_000], [1_000_000; 2]) {
            match &mut source {
                FederalIncome::Partnership { data } => data.income.push(PartnerIncomeTotal {
                    code: PartnerIncomeCode::SpeciallyAllocatedOrdinary,
                    amount: -400_000,
                }),
                // Every S-corporation OtherLoss code has a destination once
                // code C reaches Form 6781. Partnership code R remains
                // an unmapped negative row for this residual guard.
                FederalIncome::Scorp { .. } => continue,
                _ => unreachable!(),
            }
            let mut d = complete(FederalStatus::Single);
            add(&mut d, income(source));
            let r = estimate(&d);
            assert_eq!(r.state, ResultState::Unsupported);
            assert!(r.issues.iter().any(|i| i.display.key == "ISSUE_K1_AMT_LIMIT_DIFFERS"));
        }
    }
}

#[test]
fn section1256_carryback_retains_unavailable_amt_basis_and_risk_boundary() {
    for (basis, risk) in
        [([1_000_000, 500_000], [2_000_000; 2]), ([2_000_000; 2], [1_000_000, 500_000])]
    {
        for source in k1_section1256_sources(&[-1_000_000], basis, risk) {
            let mut d = complete(FederalStatus::Single);
            add(&mut d, income(source));
            d.screening.as_mut().unwrap().section_1256_carryback = Some(100_000);
            let r = estimate(&d);
            assert_eq!(r.state, ResultState::Unsupported);
            assert!(r.issues.iter().any(|i| i.display.key == "ISSUE_K1_AMT_LIMIT_DIFFERS"));
        }
    }
}

#[test]
fn section1256_actual_foreign_credit_gross_never_counts_an_amt_loss_delta_as_gain() {
    for source in k1_section1256_sources(&[-10_000], [10_000, 5_000], [100_000; 2]) {
        let mut d = passive_fund_return(200_000);
        add(&mut d, income(source));
        let r = supported(&d);
        // $200,000 wages + $10,000 foreign dividends; a contract loss is
        // restored for line 3e in both systems, never a $50 gross gain.
        for prefix in ["foreign_passive_", "foreign_amt_passive_"] {
            let gross: Vec<_> = r
                .lines
                .iter()
                .filter(|line| {
                    line.id.starts_with(prefix) && line.id.ends_with("_gross_income_all_sources")
                })
                .collect();
            assert!(!gross.is_empty());
            assert!(gross.iter().all(|line| line.amount == 21_000_000));
        }
    }
}

#[test]
fn section1256_k3_rows_stay_out_of_the_existing_foreign_gross_projection() {
    for mut source in k1_section1256_sources(&[-10_000], [10_000, 5_000], [100_000; 2]) {
        match &mut source {
            FederalIncome::Partnership { data } => data.schedule_k3 = Some(k3_blank()),
            FederalIncome::Scorp { data } => {
                let mut source_edit = edit_scorp(data);
                let data = &mut *source_edit;
                data.schedule_k3 = Some(k3_blank())
            }
            _ => unreachable!(),
        }
        let mut d = passive_fund_return(200_000);
        add(&mut d, income(source));
        add(
            &mut d,
            income(FederalIncome::Section1256 {
                data: contracts_source(crate::FederalSection1256 {
                    aggregate: 10_000,
                    withholding: 0,
                    special_treatment: false,
                }),
            }),
        );
        // Only the broker's $100 enters gross contracts in either system.
        // The K-3 loss nets regular contracts to $0 and AMT contracts to $50;
        // that $50 net delta is not another gross gain on Form 1116 line 3e.
        assert_eq!(crate::section1256::form_6781_without_k3(&d).net, 100);
        assert_eq!(crate::section1256::form_6781_amt_without_k3(&d).net, 100);
        assert_eq!(crate::section1256::form_6781(&d).net, 0);
        assert_eq!(crate::section1256::form_6781_amt(&d).net, 50);
        let r = supported(&d);
        let gross: Vec<_> = r
            .lines
            .iter()
            .filter(|line| {
                line.id.starts_with("foreign_amt_passive_")
                    && line.id.ends_with("_gross_income_all_sources")
            })
            .collect();
        assert!(!gross.is_empty());
        assert!(gross.iter().all(|line| line.amount == 21_010_000));
    }
}

#[test]
fn section1256_native_save_reopen_restart_and_correction_keep_independent_amounts() {
    for source in k1_section1256_sources(&[-10_000], [10_000, 5_000], [100_000; 2]) {
        let mut d = empty_return(FederalStatus::Single);
        add(&mut d, wage(0, Owner::Taxpayer, 20_000_000, 0));
        add(&mut d, income(source));
        let saved =
            save_reopen_native_sources(d, &[TaxSection::Documents], &[TaxSection::Documents]);
        let result = supported(&saved);
        assert_eq!(value(&result, "agi"), 199_900);
        // AMTI before exemption: $200,000 wages less the independently allowed $50 loss.
        assert_eq!(value(&result, "alternative_minimum_taxable_income"), 199_950);
        let adjustment =
            result.lines.iter().find(|line| line.id == "amt_disposition_adjustment").unwrap();
        assert_eq!(adjustment.amount, 5_000);
        assert_eq!(adjustment.form_line.as_ref().unwrap().form, TaxForm::Form6251);
        assert_eq!(adjustment.form_line.as_ref().unwrap().line, "2k");
        let id = saved.income.last().unwrap().id;
        let mut session = restart(saved);
        let mut request = open_document_center(&mut session);
        let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
        let row = documents.common.k1.documents.iter_mut().find(|row| row.id == id).unwrap();
        match &mut row.document {
            crate::K1Document::K1Partnership(page) => page.form.income[0].amount = Some(-2_000),
            crate::K1Document::K1Scorp(page) => page.form.income[0].amount = Some(-2_000),
            _ => unreachable!(),
        }
        accepted(&send(&mut session, request));
        let r = supported(&session.draft);
        assert_eq!(session.draft.income.last().unwrap().id, id);
        assert_eq!(value(&r, "agi"), 199_980);
        assert_eq!(value(&r, "alternative_minimum_taxable_income"), 199_980);
        assert_eq!(estimate(&restart(session.draft).draft), r);
    }
}
