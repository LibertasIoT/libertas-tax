use crate::tests::support::assert::k1_consolidated::k1_form4797_sum as form;
// The 2026 form's ordinary K-1 rows must join Part II, not the Part I
// capital-transfer line. Amount oracles and shared return footings are exact.
use super::*;

fn check(draft: &FederalDraft, ordinary: i64) -> FederalResult {
    let result = estimate(draft);
    assert_eq!(result.state, ResultState::EstimatedForSupportedScope, "{:?}", result.issues);
    assert_eq!(form(&result, "11"), ordinary);
    assert_eq!(form(&result, "19"), ordinary);
    assert_eq!(form(&result, "20b"), ordinary);
    assert_eq!(value(&result, "schedule_1_other_gains"), ordinary);
    assert_eq!(
        (11..=18).map(|line| form(&result, &alloc::format!("{line}"))).sum::<i64>(),
        ordinary
    );
    assert_eq!(form(&result, "10"), value(&result, "form4797_line10"));
    assert_eq!(
        form(&result, "8"),
        ["2", "3", "4", "5", "6", "7"].iter().map(|line| form(&result, line)).sum::<i64>()
    );
    result
}

#[test]
fn k1_sale_footing_code_r_uses_2026_part_two_for_gain_and_allowed_loss() {
    for amount in [8_000, -4_000] {
        let mut data = ordinary_partner_source(15_000, true);
        data.income.push(PartnerIncomeTotal {
            code: PartnerIncomeCode::SpeciallyAllocatedOrdinary,
            amount: amount * 100,
        });
        let (draft, _) =
            crate::tests::passthrough::k1_sale_return::draft(FederalIncome::Partnership { data });
        let result = check(&draft, amount);
        assert_eq!(printed_line(&result, "k1_ordinary_gains"), (TaxForm::Form4797, "11".into()));
        assert_eq!(value(&result, "agi"), 15_000 + amount);
        assert_eq!(estimate(&restart(draft).draft), result);
    }
}

#[test]
fn k1_sale_footing_ordinary_code_r_and_capital_box_ten_keep_separate_transfers() {
    let mut data = ordinary_partner_source(15_000, true);
    data.income.push(PartnerIncomeTotal {
        code: PartnerIncomeCode::SpeciallyAllocatedOrdinary,
        amount: 800_000,
    });
    data.regular.section1231 = Some(k1_section1231(5000, 0));
    data.alternative = Some(data.regular.clone());
    let (draft, _) =
        crate::tests::passthrough::k1_sale_return::draft(FederalIncome::Partnership { data });
    let result = check(&draft, 8_000);
    assert_eq!(form(&result, "2"), 5_000);
    assert_eq!(form(&result, "10"), 5_000);
    assert_eq!(value(&result, "capital_gain"), 5_000);
    assert_eq!(value(&result, "agi"), 28_000);
}

#[test]
fn k1_sale_footing_section_751_gain_joins_2026_part_two_once() {
    let mut data = ordinary_partner_source(15_000, true);
    data.final_k1 = true;
    data.regular.basis.opening = Some(0);
    data.regular.basis.opening_liabilities = 0;
    data.regular.basis.ending_liabilities = 0;
    data.alternative = Some(data.regular.clone());
    data.disposition = Some(K1Disposition::Sold { proceeds: 2_000_000 });
    data.sale = PartnerSaleParts { ordinary: 300_000, collectibles: 0, unrecaptured: 0 };
    data.interest_term = Some(GainTerm::Long);
    let (draft, _) =
        crate::tests::passthrough::k1_sale_return::draft(FederalIncome::Partnership { data });
    let result = check(&draft, 3_000);
    assert_eq!(value(&result, "k1_ordinary_gains"), 3_000);
    assert_eq!(value(&result, "capital_gain"), 2_000);
    assert_eq!(value(&result, "agi"), 20_000);
}
