use super::*;
use crate::tests::support::client::unit50_interview_native::NativeInterviewCase;
extern crate std;

fn draft(owner: Owner, filer: CarryFiler) -> FederalDraft {
    let mut d = r4_portfolio_draft();
    history(&mut d).carryforwards[0].filer = filer;
    let mut entry = crate::tests::credits::investment_credit::business_return(
        crate::tests::credits::investment_credit::earlier_energy_sale_asset(),
    )
    .income
    .pop()
    .unwrap();
    entry.owner = crate::DocumentOwner::from(owner);
    let FederalIncome::Business { data } = &mut entry.income else { panic!() };
    data.activity = BusinessActivity::Ordinary {
        returns: 0,
        receipts: 0,
        accounting: BusinessAccounting::Cash,
    };
    add(&mut d, entry);
    d.screening.as_mut().unwrap().section1231 =
        Some(Section1231History { regular: vec![], amt: vec![] });
    d
}

// IRS2026 Publication1040 draft Tax Table: MFS TI37500 ->4255,
// TI3900 ->393, TI43900 ->5023. The actual source has1958 business
// depreciation and4442 ordinary property loss, total6400. Native4255
// recapture3600 belongs to that same actual property owner.
// TP owner: liabilities[7855,393], ratio[9524,476]/10000.
// SP owner: liabilities[5023,3600], ratio[5825,4175]/10000.
// Joint TI41400 ->4475. Earlier source-energy credit800 leaves3675
// globally; individual caps are4262/213 or2607/1868 respectively.
fn expected(owner: Owner, filer: CarryFiler) -> i64 {
    match (owner, filer) {
        (_, CarryFiler::SameFilers) | (Owner::Taxpayer, CarryFiler::You) => 3675,
        (Owner::Taxpayer, CarryFiler::Spouse) => 213,
        (Owner::Spouse, CarryFiler::You) => 2607,
        (Owner::Spouse, CarryFiler::Spouse) => 1868,
    }
}
fn check(d: &FederalDraft, owner: Owner, filer: CarryFiler) -> FederalResult {
    assert!(d.injured_spouse.is_none());
    let r = supported(d);
    let used = expected(owner, filer);
    for (id, amount) in [
        ("agi", 73600),
        ("taxable_income", 41400),
        ("regular_income_tax", 4475),
        ("schedule_1_business_income", -1958),
        ("form4797_line19", -4442),
        ("investment_credit_recapture_tax", 3600),
        ("form_3800_line_16", 4475),
        ("business_credit_energy_2023_used", 800),
        ("general_business_credit", 800 + used),
    ] {
        assert_eq!(value(&r, id), amount, "{id}");
    }
    let prefix = if filer == CarryFiler::SameFilers {
        alloc::string::String::from("business_credit_work_opportunity_2025")
    } else {
        alloc::format!("business_credit_work_opportunity_source_none_filer_{}_2025", filer as i32)
    };
    assert_eq!(value(&r, &alloc::format!("{prefix}_used")), used);
    assert_eq!(value(&r, &alloc::format!("{prefix}_to_next_year")), 10000 - used);
    assert_eq!(r.tax, Some((4475 - 800 - used + 3600) * 100));
    assert!(!noted(&r, "WARN_BUSINESS_CREDIT_CARRYFORWARDS_LEFT_OUT"));
    r
}
#[test]
fn unit16_r4_delivered_property_and_recapture_drive_both_caps_without_claim() {
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        for filer in [CarryFiler::You, CarryFiler::Spouse, CarryFiler::SameFilers] {
            let mut d = draft(owner, filer);
            let r = check(&d, owner, filer);
            assert_eq!(supported(&restart(d.clone()).draft), r);
            d.income.reverse();
            let reordered = check(&d, owner, filer);
            assert_eq!(
                (r.tax, r.refund, r.balance),
                (reordered.tax, reordered.refund, reordered.balance)
            );
        }
    }
    for filer in [CarryFiler::You, CarryFiler::Spouse] {
        let mut d = draft(Owner::Spouse, filer);
        own(&mut d, BusinessCredit::WorkOpportunity, 1000);
        let r = supported(&d);
        let prefix = alloc::format!(
            "business_credit_work_opportunity_source_none_filer_{}_2025",
            filer as i32
        );
        assert_eq!(value(&r, &alloc::format!("{prefix}_used")), expected(Owner::Spouse, filer));
        assert_eq!(value(&r, "general_business_credit"), 1800 + expected(Owner::Spouse, filer));
        assert_eq!(value(&r, "investment_credit_recapture_tax"), 3600);
    }
}
#[test]
fn unit16_r4_property_owner_editor_save_reopen_restart() {
    use crate::tests::cross::interview::{accepted, open_topic};
    let profile = std::env::var("TAX_UNIT16_PROPERTY_PROFILE").unwrap_or_else(|_| "you".into());
    let filer = match profile.as_str() {
        "you" => CarryFiler::You,
        "spouse" => CarryFiler::Spouse,
        _ => panic!(),
    };
    let d = draft(Owner::Taxpayer, filer);
    let before = check(&d, Owner::Taxpayer, filer);
    let mut expected = d.income.last().unwrap().clone();
    let mut session = Interview::new(TEST_USER, d);
    // Establish the source's durable asset identity before changing its owner.
    let unchanged = open_topic(&mut session, TaxSection::SelfEmployment);
    accepted(&send(&mut session, unchanged));
    let original = session.draft.income.last().unwrap().clone();
    let (
        FederalIncome::Business { data: accepted_data },
        FederalIncome::Business { data: expected_data },
    ) = (&original.income, &mut expected.income)
    else {
        unreachable!()
    };
    assert_eq!(accepted_data.assets.len(), 1);
    assert_eq!(expected_data.assets.len(), 1);
    assert!(accepted_data.assets[0].id > 0);
    expected_data.assets[0].id = accepted_data.assets[0].id;
    assert_eq!(original, expected);
    check(&session.draft, Owner::Taxpayer, filer);
    let mut page = open_topic(&mut session, TaxSection::SelfEmployment);
    let native = NativeInterviewCase("TAX_UNIT16_PROPERTY");
    if native.required_checked_calculation(&mut session) {
        return;
    }
    if let Ok(dir) = std::env::var("TAX_UNIT16_PROPERTY_SEED_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(std::path::Path::new(&dir).join("seed.avro"), page.to_avro()).unwrap();
        std::fs::write(
            std::path::Path::new(&dir).join("target-id.json"),
            alloc::format!("{}", original.id),
        )
        .unwrap();
        return;
    }
    let request = if let Some(request) = native.checked_input("PAYLOAD") {
        request
    } else {
        let P::SaveIncomes { documents, .. } = &mut page else { panic!() };
        let target = documents.documents.iter_mut().find(|v| v.id == original.id).unwrap();
        target.owner = Owner::Spouse;
        page
    };
    accepted(&send(&mut session, request));
    let changed = session.draft.income.last().unwrap();
    assert_eq!(changed.id, original.id);
    assert_eq!(changed.income, original.income);
    assert_eq!(changed.owner, crate::DocumentOwner::Spouses);
    let after = check(&session.draft, Owner::Spouse, filer);
    assert_ne!(value(&before, "general_business_credit"), value(&after, "general_business_credit"));
    let saved = session.draft.clone();
    session = restart(saved.clone());
    assert_eq!(session.draft, saved);
    let reopened = open_topic(&mut session, TaxSection::SelfEmployment);
    native.output(&reopened, &native_draft_records(&saved));
    accepted(&send(&mut session, reopened));
    assert_eq!(session.draft, saved);
    let resaved = check(&session.draft, Owner::Spouse, filer);
    assert_eq!(
        (resaved.tax, resaved.refund, resaved.balance),
        (after.tax, after.refund, after.balance)
    );
}
