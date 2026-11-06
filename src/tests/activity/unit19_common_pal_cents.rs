use super::*;
use crate::tests::support::client::unit50_interview_native::NativeInterviewCase;
use crate::tests::support::client::{
    draft_from_native_records, native_draft_records, write_native_draft_records,
};

fn fixture() -> FederalDraft {
    let mut draft = complete(FederalStatus::Single);
    let mut source = ordinary_partner_source(-1, false);
    source.participation = Some(crate::PartnerParticipation::Limited {
        minutes: 0,
        prior_ten: 0,
        personal_service: false,
        service_years: 0,
    });
    source.regular.basis.opening = Some(10_000);
    source.regular.risk.start =
        crate::ScorpRiskStart::Detailed { capacity: 10_000, profit: 0, loss: 0 };
    source.alternative = Some(source.regular.clone());
    add(&mut draft, income(FederalIncome::Partnership { data: source }));
    draft
}

fn equipment(receipts: i64, expenses: i64, systems: bool) -> EquipmentRent {
    EquipmentRent {
        related: crate::RelatedForms::default(),
        receipts,
        expenses,
        basis: 1_000_000,
        depreciable_basis: 1_000_000,
        assets: vec![],
        history: PersonalRentalHistory {
            regular: if systems { 51 } else { 0 },
            amt: if systems { 151 } else { 0 },
        },
    }
}

fn pool(draft: &FederalDraft, alternative: bool) -> crate::extended_rules::RentalResult {
    let mut result = FederalResult::empty(draft.revision);
    let pool = crate::extended_rules::rentals_for(draft, 60_000, &mut result, alternative);
    assert!(pool.inputs_complete && result.issues.is_empty(), "{:?}", result.issues);
    pool
}

#[test]
fn unit19_common_pal_actual_client_save_reopen_restart() {
    extern crate std;
    let systems = std::env::var("TAX_UNIT19_COMMON_PAL_KIND").is_ok_and(|kind| kind == "systems");
    let mut session = Interview::new(TEST_USER, fixture());
    let seed = open_topic(&mut session, TaxSection::OtherIncome);
    let native = NativeInterviewCase("TAX_UNIT19_COMMON_PAL");
    if native.required_checked_calculation(&mut session) {
        return;
    }
    let mut answers = seed.clone();
    let P::SaveIncomes { documents, .. } = &mut answers else { unreachable!() };
    documents.documents = vec![
        document(
            0,
            IncomeSource::Record {
                value: crate::IncomeRecord::EquipmentRental(equipment(50, 0, false)),
            },
        ),
        document(
            0,
            IncomeSource::Record {
                value: crate::IncomeRecord::EquipmentRental(equipment(0, 149, systems)),
            },
        ),
    ]
    .into_iter()
    .map(|row| crate::IncomeRecordItem::from_income(row).expect("record fixture"))
    .collect();
    if let Ok(directory) = std::env::var("TAX_UNIT19_COMMON_PAL_DIR") {
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(std::path::Path::new(&directory).join("seed.avro"), seed.to_avro()).unwrap();
        std::fs::write(std::path::Path::new(&directory).join("answers.avro"), answers.to_avro())
            .unwrap();
    }
    if std::env::var("TAX_UNIT19_COMMON_PAL_SEED_ONLY").is_ok() {
        return;
    }
    let request = native.input("PAYLOAD").unwrap_or(answers);
    accepted(&send(&mut session, request));
    let saved = session.draft.clone();
    let ids: Vec<_> = saved
        .income
        .iter()
        .filter(|entry| matches!(entry.income, FederalIncome::NonbusinessProperty { .. }))
        .map(|entry| entry.id)
        .collect();
    assert_eq!(ids.len(), 2);
    for alternative in [false, true] {
        let calculated = pool(&saved, alternative);
        let packet = calculated.post_pal.as_ref().unwrap();
        let source = |id| {
            packet
                .sources
                .iter()
                .find(|source| {
                    source.income == id
                        && source.component
                            == crate::extended_rules::PostPalComponent::PersonalProperty
                })
                .unwrap()
        };
        let cost = source(ids[1]);
        assert_eq!(source(ids[0]).suspended_cents, 0);
        assert_eq!(calculated.personal_inputs[&ids[1]].expenses_cents, 149);
        assert_eq!(cost.prior_cents, if systems { if alternative { 151 } else { 51 } } else { 0 });
        assert_eq!(
            cost.suspended_cents,
            if systems { if alternative { 263 } else { 167 } } else { 119 }
        );
        assert_eq!(packet.allowed_loss_cents, 50);
    }
    let result = supported(&saved);
    let records = native_draft_records(&saved);
    let draft = draft_from_native_records(saved.tax_year, &records);
    assert_eq!(native_draft_records(&draft), records);
    assert_eq!(draft, saved);
    let mut session = restart(draft);
    let reopened = open_topic(&mut session, TaxSection::OtherIncome);
    if let Ok(directory) = std::env::var("TAX_UNIT19_COMMON_PAL_DIR") {
        write_native_draft_records(
            std::path::Path::new(&directory).join("accepted-record.avro"),
            &records,
        );
        std::fs::write(std::path::Path::new(&directory).join("reopened.avro"), reopened.to_avro())
            .unwrap();
        for (label, alternative) in [("regular", false), ("alternative", true)] {
            std::fs::write(
                std::path::Path::new(&directory).join(std::format!("{label}-post-pal.txt")),
                std::format!("{:?}", pool(&saved, alternative).post_pal),
            )
            .unwrap();
        }
    }
    let resave = native.input("REOPEN_PAYLOAD").unwrap_or(reopened);
    accepted(&send(&mut session, resave));
    assert_eq!(session.draft, saved);
    assert_eq!(supported(&session.draft), result);
    let mut correction = open_topic(&mut session, TaxSection::OtherIncome);
    let P::SaveIncomes { documents, .. } = &mut correction else { unreachable!() };
    let crate::IncomeRecord::EquipmentRental(page) = &mut documents.documents[1].document else {
        unreachable!()
    };
    page.expenses += 2;
    documents.documents.reverse();
    accepted(&send(&mut session, correction));
    assert_eq!(pool(&session.draft, false).personal_inputs[&ids[1]].expenses_cents, 151);
    assert_eq!(pool(&session.draft, false).personal_inputs[&ids[0]].receipts_cents, 50);
    assert_eq!(restart(session.draft.clone()).draft, session.draft);
}
