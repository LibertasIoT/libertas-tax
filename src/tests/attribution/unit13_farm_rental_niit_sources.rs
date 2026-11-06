use super::*;
use crate::tests::support::client::unit50_interview_native::NativeInterviewCase;
use crate::tests::support::client::{draft_from_native_records, native_draft_records};

fn farm(production: i64, expenses: i64) -> FederalFarmRental {
    let FederalIncome::Business { data } = raised_farm(production, expenses) else { panic!() };
    let BusinessActivity::Farming { data: mut farm } = data.activity else { panic!() };
    farm.header = None;
    FederalFarmRental {
        improvement_buildings: Vec::new(),
        interest_limit_election: None,
        farm,
        expenses: data.expenses,
        assets: vec![],
        vehicles: vec![],
        active_participation: true,
        at_risk: Some(crate::RentalAtRisk::All),
        history: RentalHistory::NONE,
        qbi_business: RentalBusiness::No,
        qbi_aggregation_group: None,
        payroll: BusinessPayroll::No,
    }
}

fn fixture() -> FederalDraft {
    let mut d = injured_return((200_000, 100_000), (50_000, 0), 0, allocation());
    add(&mut d, income(FederalIncome::FarmRental { data: farm(11_000, 1000) }));
    let mut second = income(FederalIncome::FarmRental { data: farm(20_000, 5000) });
    second.owner = crate::DocumentOwner::Spouses;
    add(&mut d, second);
    d
}

fn no_missing_niit(r: &FederalResult) {
    assert!(
        !r.review_notes.iter().any(|n| n.display.key == "WARN_8379_OTHER_TAX_OWNER_UNAVAILABLE")
    );
}

#[test]
fn unit13_farm_rental_native_allowed_sources_keep_their_niit_owners() {
    let mut d = fixture();
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        d.injured_spouse.as_mut().unwrap().injured = injured;
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), 275_000);
        assert_eq!(value(&r, "schedule_e_farm_rental_total"), 25_000);
        assert_eq!(value(&r, "form_8960_net_investment_income"), 25_000);
        assert_eq!(value(&r, "net_investment_income_tax"), 950);
        assert_eq!(value(&r, "self_employment_tax"), 0);
        let i = usize::from(injured == Owner::Spouse);
        assert_eq!(
            [column(&r, "13b", "b"), column(&r, "13b", "c")],
            [[10_000, 15_000], [15_000, 10_000]][i]
        );
        assert_injured_ordinary_unchanged(&d, &r, supported);
        assert_eq!([column(&r, "18", "b"), column(&r, "18", "c")], [[380, 570], [570, 380]][i]);
        no_missing_niit(&r);
    }
    d.income.reverse();
    let r = supported(&d);
    assert_eq!([column(&r, "18", "b"), column(&r, "18", "c")], [570, 380]);
    assert_eq!(estimate(&restart(d.clone()).draft), r);
}

#[test]
fn unit13_farm_rental_final_native_source_keeps_depreciation_and_system_history() {
    let mut d = fixture();
    let FederalIncome::FarmRental { data } = &mut d.income[2].income else { panic!() };
    let mut asset = rental_equipment(5000, BonusElection::ElectOut);
    asset.description = "Share lease equipment".into();
    data.assets.push(asset);
    data.history = RentalHistory { regular: 300_000, amt: 500_000, ..RentalHistory::NONE };
    let FederalIncome::FarmRental { data } = &mut d.income[3].income else { panic!() };
    data.expenses[0].amount = 400_000;
    data.history = RentalHistory { regular: 900_000, amt: 1_000_000, ..RentalHistory::NONE };
    for alternative in [false, true] {
        let mut result = FederalResult::empty(d.revision);
        let pool = crate::extended_rules::rentals_for(&d, 250_000, &mut result, alternative);
        assert!(pool.inputs_complete);
        assert!(result.issues.is_empty(), "{:?}", result.issues);
        // This is the existing final annual-dollar owner, not raw cents.
        assert_eq!(
            [pool.income_by_id[&3], pool.income_by_id[&4]],
            if alternative { [4000, 6000] } else { [6000, 7000] }
        );
        assert_eq!(pool.farm_rental, if alternative { 10_000 } else { 13_000 });
    }
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 263_000);
    assert_eq!(value(&r, "form_8960_net_investment_income"), 13_000);
    assert_eq!(value(&r, "net_investment_income_tax"), 494);
    assert_eq!([column(&r, "13b", "b"), column(&r, "13b", "c")], [6000, 7000]);
    assert_injured_ordinary_unchanged(&d, &r, supported);
    assert_eq!([column(&r, "18", "b"), column(&r, "18", "c")], [228, 266]);
    no_missing_niit(&r);
}

#[test]
fn unit13_farm_rental_niit_uses_the_existing_native_annual_boundary() {
    let mut d = injured_return((200_000, 100_000), (50_000, 0), 0, allocation());
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let mut data = farm(1000, 0);
        let FarmAccounting::Cash { receipts } = &mut data.farm.accounting else { panic!() };
        let FarmReceipt::Raised { amount } = &mut receipts[0] else { panic!() };
        *amount += 49;
        let mut entry = income(FederalIncome::FarmRental { data });
        entry.owner = crate::DocumentOwner::from(owner);
        add(&mut d, entry);
    }
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        d.injured_spouse.as_mut().unwrap().injured = injured;
        let r = supported(&d);
        // Each separate native Form4835 result is annual1000, not a raw
        //100049-cent source. Preserve that existing filing boundary.
        assert_eq!(value(&r, "schedule_e_farm_rental_total"), 2000);
        assert_eq!(value(&r, "form_8960_net_investment_income"), 2000);
        assert_eq!(value(&r, "net_investment_income_tax"), 76);
        assert_eq!([column(&r, "13b", "b"), column(&r, "13b", "c")], [1000, 1000]);
        assert_eq!([column(&r, "18", "b"), column(&r, "18", "c")], [38, 38]);
        no_missing_niit(&r);
        assert_injured_ordinary_unchanged(&d, &r, supported);
    }
}

#[test]
fn unit13_farm_rental_signed_niit_keeps_the_named_unavailable_boundary() {
    let mut d = fixture();
    let FederalIncome::FarmRental { data } = &mut d.income[2].income else { panic!() };
    data.expenses[0].amount = 3_100_000;
    add(
        &mut d,
        income(FederalIncome::Interest {
            data: interest_source(FederalInterest { taxable: 500_000, ..Default::default() }),
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "schedule_e_farm_rental_total"), 0);
    assert_eq!(value(&r, "farm_rental_3_suspended_loss"), 5000);
    assert_eq!(value(&r, "form_8960_net_investment_income"), 5000);
    assert_eq!(value(&r, "net_investment_income_tax"), 190);
    assert_eq!([column(&r, "13b", "b"), column(&r, "13b", "c")], [-10_000, 15_000]);
    assert!(
        r.review_notes.iter().any(|n| n.display.key == "WARN_8379_OTHER_TAX_OWNER_UNAVAILABLE")
    );
    assert!(!r.lines.iter().any(|l| l.id == "form_8379_line_18_b"));
    assert_injured_ordinary_unchanged(&d, &r, supported);
}

#[test]
fn unit13_farm_rental_source_is_shared_with_changed_filer_credit_without_a_claim() {
    for filer in [CarryFiler::You, CarryFiler::Spouse, CarryFiler::SameFilers] {
        let mut d = fixture();
        d.injured_spouse = None;
        d.credits.as_mut().unwrap().business_credits = Some(BusinessCreditHistory {
            credits: vec![],
            carryforwards: vec![BusinessCreditCarryforward {
                credit: BusinessCredit::WorkOpportunity,
                origin_year: 2025,
                amount: 1_000_000,
                passive: None,
                filer,
                research_source: None,
            }],
            carrybacks: vec![],
            spouse_without_credits: false,
        });
        let ordinary = supported(&d);
        assert_eq!(value(&ordinary, "agi"), 275_000);
        assert_eq!(value(&ordinary, "net_investment_income_tax"), 950);
        assert!(!noted(&ordinary, "WARN_BUSINESS_CREDIT_CARRYFORWARDS_LEFT_OUT"));
        // This is a coupled-source availability/claim-independence control,
        // not an independent numerical oracle for the complete R4 credit cap.
        assert!((1..=10_000).contains(&value(&ordinary, "general_business_credit")));
        d.set_situations(SpouseSituations::INJURED_SPOUSE.0, true);
        d.injured_spouse = Some(allocation());
        let claimed = supported(&d);
        assert_injured_ordinary_unchanged(&d, &claimed, supported);
        assert_eq!(
            (claimed.tax, claimed.refund, claimed.balance),
            (ordinary.tax, ordinary.refund, ordinary.balance)
        );
        no_missing_niit(&claimed);
        assert_eq!(estimate(&restart(d.clone()).draft), claimed);
    }
}

#[test]
fn unit13_farm_rental_actual_client_save_reopen_restart() {
    extern crate std;
    use crate::tests::cross::interview::{accepted, open_topic};
    let selected = std::env::var("TAX_UNIT13_FARM_NII_PROFILE").ok();
    for profile in ["receipt", "expense"] {
        if selected.as_deref().is_some_and(|p| p != profile) {
            continue;
        }
        let mut session = Interview::new(TEST_USER, fixture());
        let mut page = open_topic(&mut session, TaxSection::Farming);
        let native = NativeInterviewCase("TAX_UNIT13_FARM_NII");
        if native.required_checked_calculation(&mut session) {
            return;
        }
        if native.seed(&page) {
            return;
        }
        let request = if let Some(request) = native.checked_input("PAYLOAD") {
            request
        } else {
            let P::SaveIncomes { documents, .. } = &mut page else { panic!() };
            let row = &mut documents.documents[if profile == "receipt" { 0 } else { 1 }];
            row.owner = if profile == "receipt" { Owner::Spouse } else { Owner::Taxpayer };
            let crate::IncomeRecord::FarmRental(value) = &mut row.document else { panic!() };
            if profile == "receipt" {
                value.production = 1_200_000;
            } else {
                value.expenses.other[0].amount = 600_000;
            }
            page
        };
        accepted(&send(&mut session, request));
        let check = |d: &FederalDraft| {
            let r = supported(d);
            let (net, tax, owners) = if profile == "receipt" {
                (26_000, 988, [0, 988])
            } else {
                (24_000, 912, [912, 0])
            };
            assert_eq!(value(&r, "agi"), 250_000 + net);
            assert_eq!(value(&r, "schedule_e_farm_rental_total"), net);
            assert_eq!(value(&r, "form_8960_net_investment_income"), net);
            assert_eq!(value(&r, "net_investment_income_tax"), tax);
            assert_eq!(
                [column(&r, "13b", "b"), column(&r, "13b", "c")],
                if profile == "receipt" { [0, net] } else { [net, 0] }
            );
            assert_eq!([column(&r, "18", "b"), column(&r, "18", "c")], owners);
            assert_injured_ordinary_unchanged(d, &r, supported);
            no_missing_niit(&r);
        };
        check(&session.draft);
        let tax_year = session.draft.tax_year;
        let stored = native_draft_records(&session.draft);
        let draft = draft_from_native_records(tax_year, &stored);
        assert_eq!(draft, session.draft);
        let mut reopened = restart(draft);
        let shown = open_topic(&mut reopened, TaxSection::Farming);
        native.output(&shown, &stored);
        let accepted_draft = reopened.draft.clone();
        accepted(&send(&mut reopened, shown));
        assert_eq!(reopened.draft, accepted_draft);
        check(&reopened.draft);
    }
}
