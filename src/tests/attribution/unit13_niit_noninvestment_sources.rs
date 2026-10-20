use super::*;
use crate::tests::support::assert::assert_nonallocation_ordinary_unchanged as ordinary_unchanged;
use crate::tests::support::client::unit50_interview_native::NativeInterviewCase;
use crate::tests::support::client::{draft_from_native_records, native_draft_records};

fn check_noninvestment_source(mut source: FederalIncomeEntry, gross: i64, adjustment: i64) {
    let mut draft = injured_return((200_000, 100_000), (50_000, 0), 0, allocation());
    add(&mut draft, income(savings_income(5000, 0)));
    let mut interest = income(savings_income(15_000, 0));
    interest.owner = Owner::Spouse;
    add(&mut draft, interest);
    source.owner = Owner::Spouse;
    add(&mut draft, source);
    // Native ordinary-source treatment changes MAGI, not the actual interest
    // recipients. The joint ceiling exceeds the unchanged20,000 NII pool.
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        draft.injured_spouse.as_mut().unwrap().injured = injured;
        let result = supported(&draft);
        assert_eq!(value(&result, "agi"), 270_000 + gross - adjustment);
        assert_eq!(value(&result, "form_8960_net_investment_income"), 20_000);
        assert_eq!(value(&result, "net_investment_income_tax"), 760);
        ordinary_unchanged(&draft, &result);
        let owners = if injured == Owner::Taxpayer { [190, 570] } else { [570, 190] };
        assert_eq!([column(&result, "18", "b"), column(&result, "18", "c")], owners);
        let income = if injured == Owner::Taxpayer {
            [5000, 15_000 + gross]
        } else {
            [15_000 + gross, 5000]
        };
        assert_eq!([column(&result, "13b", "b"), column(&result, "13b", "c")], income);
        let deductions = if injured == Owner::Taxpayer { [0, adjustment] } else { [adjustment, 0] };
        assert_eq!([column(&result, "14", "b"), column(&result, "14", "c")], deductions);
        assert!(
            !result
                .review_notes
                .iter()
                .any(|note| note.display.key == "WARN_8379_OTHER_TAX_OWNER_UNAVAILABLE")
        );
        assert_eq!(estimate(&restart(draft.clone()).draft), result);
    }
    draft.income.reverse();
    assert_eq!(value(&supported(&draft), "form_8960_net_investment_income"), 20_000);
}

#[test]
fn unit13_niit_government_unemployment_retains_actual_investment_owners() {
    check_noninvestment_source(government_unemployment(10_000, &[2000], 0), 8000, 0);
}

#[test]
fn unit13_niit_taxable_alimony_retains_actual_investment_owners() {
    // Actual legacy instrument:18,000 paid,8,000 child support first,
    // leaving10,000 taxable alimony; no invented extra receipt question.
    check_noninvestment_source(
        income(FederalIncome::Alimony { data: alimony_record(20180101) }),
        10_000,
        0,
    );
}

#[test]
fn unit13_niit_jury_pay_and_remittance_retain_actual_investment_owners() {
    check_noninvestment_source(
        income(FederalIncome::JuryDuty {
            data: JuryPay { received: 1_000_000, remitted: 200_000 },
        }),
        10_000,
        2000,
    );
}

#[test]
fn unit13_niit_taxable_scholarship_retains_actual_investment_owners() {
    check_noninvestment_source(income(scholarship(12_000, 2000, 4000)), 8000, 0);
}

#[test]
fn unit13_niit_casual_gambling_retains_actual_investment_owners() {
    check_noninvestment_source(gambling(10_000, 0), 10_000, 0);
}

#[test]
fn unit13_niit_actual_1099g_other_boxes_are_counted_at_their_native_source_once() {
    let mut entry = government_unemployment(10_000, &[2000], 0);
    let FederalIncome::Unemployment { data } = &mut entry.income else { unreachable!() };
    data.rtaa = 100_000;
    data.taxable_grants = 300_000;
    data.grant_use = Some(FederalGrantUse::You);
    data.family_leave = 200_000;
    //8,000 actual unemployment +1,000 RTAA +3,000 personal grant +2,000 leave.
    check_noninvestment_source(entry, 14_000, 0);
}

#[test]
fn unit13_niit_repaid_benefit_cents_retain_selected_owner_annual_rounding() {
    let mut draft = injured_return((200_000, 100_000), (50_000, 0), 0, allocation());
    add(&mut draft, income(savings_income(5000, 0)));
    let mut interest = income(savings_income(15_000, 0));
    interest.owner = Owner::Spouse;
    add(&mut draft, interest);
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let mut entry = government_unemployment(1000, &[1000], 0);
        let FederalIncome::Unemployment { data } = &mut entry.income else { unreachable!() };
        data.amount += 49;
        entry.owner = owner;
        add(&mut draft, entry);
    }
    //Two49-cent native net benefits aggregate to one annual dollar.
    //The selected first column rounds its own source once; the other keeps
    //the joint residual. No benefit or repayment is rounded per statement.
    for (injured, parts) in [(Owner::Taxpayer, [5000, 15_001]), (Owner::Spouse, [15_000, 5001])] {
        draft.injured_spouse.as_mut().unwrap().injured = injured;
        let result = supported(&draft);
        assert_eq!(value(&result, "agi"), 270_001);
        assert_eq!(value(&result, "form_8960_net_investment_income"), 20_000);
        assert_eq!(value(&result, "net_investment_income_tax"), 760);
        assert_eq!([column(&result, "13b", "b"), column(&result, "13b", "c")], parts);
        ordinary_unchanged(&draft, &result);
        assert_eq!(estimate(&restart(draft.clone()).draft), result);
    }
}

#[test]
fn unit13_niit_noninvestment_actual_client_save_reopen_restart() {
    extern crate std;
    use crate::tests::cross::interview::{accepted, open_topic};
    let selected = std::env::var("TAX_UNIT13_NII_NONINV_PROFILE").ok();
    for profile in ["unemployment", "jury"] {
        if selected.as_deref().is_some_and(|value| value != profile) {
            continue;
        }
        let mut draft = injured_return((200_000, 100_000), (50_000, 0), 0, allocation());
        add(&mut draft, income(savings_income(5000, 0)));
        let mut interest = income(savings_income(15_000, 0));
        interest.owner = Owner::Spouse;
        add(&mut draft, interest);
        let mut source = if profile == "unemployment" {
            government_unemployment(10_000, &[2000], 0)
        } else {
            income(FederalIncome::JuryDuty {
                data: JuryPay { received: 1_000_000, remitted: 200_000 },
            })
        };
        source.owner = Owner::Spouse;
        add(&mut draft, source);
        let mut session = Interview::new(TEST_USER, draft);
        let topic = if profile == "unemployment" {
            TaxSection::GovernmentBenefits
        } else {
            TaxSection::OtherIncome
        };
        let mut page = open_topic(&mut session, topic);
        let native = NativeInterviewCase("TAX_UNIT13_NII_NONINV");
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
            let row = documents.documents.iter_mut().find(|row| row.id == 5).unwrap();
            row.owner = Owner::Taxpayer;
            match &mut row.document {
                IncomeDocument::Form1099G(data) => {
                    data.amount = 1_200_000;
                    data.more.repayments[0].amount = 300_000;
                }
                IncomeDocument::JuryPay(data) => {
                    data.received = 1_200_000;
                    data.remitted = 300_000;
                }
                _ => panic!(),
            }
            page
        };
        accepted(&send(&mut session, request));
        let check = |draft: &FederalDraft| {
            let result = supported(draft);
            assert_eq!(value(&result, "agi"), 279_000);
            assert_eq!(value(&result, "form_8960_net_investment_income"), 20_000);
            assert_eq!(value(&result, "net_investment_income_tax"), 760);
            assert_eq!(
                [column(&result, "13b", "b"), column(&result, "13b", "c")],
                if profile == "unemployment" { [14_000, 15_000] } else { [17_000, 15_000] }
            );
            assert_eq!(
                [column(&result, "14", "b"), column(&result, "14", "c")],
                if profile == "unemployment" { [0, 0] } else { [3000, 0] }
            );
            assert_eq!([column(&result, "18", "b"), column(&result, "18", "c")], [190, 570]);
            ordinary_unchanged(draft, &result);
        };
        check(&session.draft);
        let tax_year = session.draft.tax_year;
        let stored = native_draft_records(&session.draft);
        let draft = draft_from_native_records(tax_year, &stored);
        assert_eq!(draft, session.draft);
        let mut reopened = restart(draft);
        let shown = open_topic(&mut reopened, topic);
        native.output(&shown, &stored);
        let accepted_draft = reopened.draft.clone();
        accepted(&send(&mut reopened, shown));
        assert_eq!(reopened.draft, accepted_draft);
        check(&reopened.draft);
    }
}
