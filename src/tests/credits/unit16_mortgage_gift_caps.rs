use super::*;
use crate::tests::support::client::unit50_interview_native::NativeInterviewCase;
extern crate std;

fn owner_slot(d: &mut FederalDraft) -> &mut Option<MortgageSubsidyGiftOwners> {
    let home = &mut d.credits.as_mut().unwrap().mortgage_subsidies[0];
    let MortgageSubsidyDisposition::Ordinary {
        transfer: MortgageSubsidyTransfer::GiftToOther { owners, .. },
        ..
    } = &mut home.disposition
    else {
        panic!()
    };
    owners
}
fn draft(owner: Option<MortgageSubsidyGiftOwners>, filer: CarryFiler) -> FederalDraft {
    let mut d = r4_portfolio_draft();
    history(&mut d).carryforwards[0].filer = filer;
    d.credits.as_mut().unwrap().mortgage_subsidies =
        vec![crate::tests::federal::other_taxes::gift(20200601, 300000, 200000, 0, 60)];
    *owner_slot(&mut d) = owner;
    d
}
// Independent2026 MFS Table liabilities without recapture are5023/393.
// Actual Form8828 tax7500 gives12523/393 (TP) or5023/7893 (SP).
// Native ratios9696/304 or3889/6111 apply to the same joint limit5243.
fn expected(owner: MortgageSubsidyGiftOwners, filer: CarryFiler) -> i64 {
    match (owner, filer) {
        (_, CarryFiler::SameFilers) => 5243,
        (MortgageSubsidyGiftOwners::Taxpayer, CarryFiler::You) => 5084,
        (MortgageSubsidyGiftOwners::Taxpayer, CarryFiler::Spouse) => 159,
        (MortgageSubsidyGiftOwners::Spouse, CarryFiler::You) => 2039,
        (MortgageSubsidyGiftOwners::Spouse, CarryFiler::Spouse) => 3204,
        (MortgageSubsidyGiftOwners::Both, _) => 0,
    }
}
fn check(d: &FederalDraft, used: i64) -> FederalResult {
    assert!(d.injured_spouse.is_none());
    let r = supported(d);
    for (id, amount) in [
        ("agi", 80000),
        ("taxable_income", 47800),
        ("regular_income_tax", 5243),
        ("form_3800_line_16", 5243),
        ("mortgage_subsidy_recapture_tax", 7500),
        ("general_business_credit", used),
    ] {
        assert_eq!(value(&r, id), amount, "{id}");
    }
    assert_eq!(r.tax, Some((5243 - used + 7500) * 100));
    r
}
#[test]
fn unit16_r4_mortgage_gift_actual_owners_drive_caps_without_claim_or_guess() {
    for owner in [MortgageSubsidyGiftOwners::Taxpayer, MortgageSubsidyGiftOwners::Spouse] {
        for filer in [CarryFiler::You, CarryFiler::Spouse, CarryFiler::SameFilers] {
            let d = draft(Some(owner), filer);
            let used = expected(owner, filer);
            let r = check(&d, used);
            assert!(!noted(&r, "WARN_BUSINESS_CREDIT_CARRYFORWARDS_LEFT_OUT"));
            assert_eq!(supported(&restart(d.clone()).draft), r);
            let prefix = if filer == CarryFiler::SameFilers {
                alloc::string::String::from("business_credit_work_opportunity_2025")
            } else {
                alloc::format!(
                    "business_credit_work_opportunity_source_none_filer_{}_2025",
                    filer as i32
                )
            };
            assert_eq!(value(&r, &alloc::format!("{prefix}_to_next_year")), 10000 - used);
        }
    }
    for owner in [None, Some(MortgageSubsidyGiftOwners::Both)] {
        for filer in [CarryFiler::You, CarryFiler::Spouse] {
            let r = check(&draft(owner, filer), 0);
            assert!(noted(&r, "WARN_BUSINESS_CREDIT_CARRYFORWARDS_LEFT_OUT"));
        }
        let r = check(&draft(owner, CarryFiler::SameFilers), 5243);
        assert!(!noted(&r, "WARN_BUSINESS_CREDIT_CARRYFORWARDS_LEFT_OUT"));
    }
    for owner in [MortgageSubsidyGiftOwners::Taxpayer, MortgageSubsidyGiftOwners::Spouse] {
        let mut d = draft(Some(owner), CarryFiler::You);
        own(&mut d, BusinessCredit::WorkOpportunity, 1000);
        let r = supported(&d);
        assert_eq!(
            value(&r, "business_credit_work_opportunity_source_none_filer_1_2025_used"),
            expected(owner, CarryFiler::You)
        );
        assert_eq!(value(&r, "mortgage_subsidy_recapture_tax"), 7500);
    }
}
#[test]
fn unit16_r4_mortgage_gift_owner_editor_save_reopen_restart() {
    use crate::tests::cross::interview::{accepted, open_other_taxes, rejected_with};
    let profile = std::env::var("TAX_UNIT16_GIFT_PROFILE").unwrap_or_else(|_| "tp-you".into());
    let owner = if profile.starts_with("tp-") {
        MortgageSubsidyGiftOwners::Taxpayer
    } else {
        MortgageSubsidyGiftOwners::Spouse
    };
    let filer = if profile.ends_with("-you") { CarryFiler::You } else { CarryFiler::Spouse };
    let d = draft(None, filer);
    let before = check(&d, 0);
    assert!(noted(&before, "WARN_BUSINESS_CREDIT_CARRYFORWARDS_LEFT_OUT"));
    let original = d.credits.as_ref().unwrap().mortgage_subsidies[0].clone();
    let mut session = Interview::new(TEST_USER, d);
    let mut page = open_other_taxes(&mut session);
    let native = NativeInterviewCase("TAX_UNIT16_GIFT");
    if native.required_checked_calculation(&mut session) {
        return;
    }
    if native.seed(&page) {
        return;
    }
    let request = if let Some(request) = native.checked_input("PAYLOAD") {
        request
    } else {
        let P::SaveOtherTaxes { value, .. } = &mut page else { panic!() };
        let OtherTax::MortgageSubsidyRecapture { homes } = &mut value.taxes[0] else { panic!() };
        let MortgageSubsidyDisposition::Ordinary {
            transfer: MortgageSubsidyTransfer::GiftToOther { owners, .. },
            ..
        } = &mut homes[0].disposition
        else {
            panic!()
        };
        *owners = Some(owner);
        page
    };
    accepted(&send(&mut session, request));
    let saved = session.draft.clone();
    check(&saved, expected(owner, filer));
    let mut no_answer = saved.clone();
    *owner_slot(&mut no_answer) = None;
    assert_eq!(no_answer.credits.as_ref().unwrap().mortgage_subsidies[0], original);
    session = restart(saved.clone());
    assert_eq!(session.draft, saved);
    let reopened = open_other_taxes(&mut session);
    native.output(&reopened, &native_draft_records(&saved));
    accepted(&send(&mut session, reopened));
    assert_eq!(session.draft, saved);
    check(&session.draft, expected(owner, filer));
    let mut invalid = open_other_taxes(&mut session);
    let P::SaveOtherTaxes { value, .. } = &mut invalid else { panic!() };
    let OtherTax::MortgageSubsidyRecapture { homes } = &mut value.taxes[0] else { panic!() };
    let MortgageSubsidyDisposition::Ordinary {
        transfer: MortgageSubsidyTransfer::GiftToOther { owners, .. },
        ..
    } = &mut homes[0].disposition
    else {
        panic!()
    };
    *owners = None;
    rejected_with(send(&mut session, invalid), "MORTGAGE_GIFT_OWNER_REQUIRED");
    assert_eq!(session.draft, saved);
}
