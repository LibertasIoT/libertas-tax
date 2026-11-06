use super::*;
use crate::tests::support::client::unit50_interview_native::NativeInterviewCase;
use crate::tests::support::client::{
    draft_from_native_records, native_draft_records, write_native_draft_records,
};

fn page(rent: i64) -> Rental {
    let mut page = rental_page(rent);
    page.address = "Synthetic retained rental source".into();
    page.kind = crate::RentalPropertyType::Land;
    page.building.cost = 100_000;
    page.building.land = 100_000;
    page.building.in_service = 20200101;
    page
}
fn pool(draft: &FederalDraft, alternative: bool) -> crate::extended_rules::RentalResult {
    crate::extended_rules::rentals_for_home_with_care(
        draft,
        200_000,
        &mut FederalResult::empty(draft.revision),
        alternative,
        [200_000, 200_000],
        &crate::care::CareDeductions::default(),
    )
}

#[test]
fn unit19_rental_stages_actual_client_save_reopen_restart() {
    extern crate std;
    let prior = std::env::var("TAX_UNIT19_RENTAL_STAGE_KIND").is_ok_and(|kind| kind == "prior");
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let seed = open_topic(&mut session, TaxSection::Rentals);
    let native = NativeInterviewCase("TAX_UNIT19_RENTAL_STAGE");
    if native.required_calculation(&mut session) {
        return;
    }
    let mut first = page(if prior { 0 } else { 149 });
    if prior {
        first.expenses.repairs = 249;
        first.more.at_risk = simplified_rental_risk(149, 0, 0, [50, 75, 0, 0]);
        first.more.suspended = SuspendedLoss { regular: 100, amt: 200, active_in_loss_year: true };
    }
    let mut answers = seed.clone();
    let P::SaveIncomes { documents, .. } = &mut answers else { unreachable!() };
    documents.documents = vec![
        document(0, IncomeSource::Record { value: crate::IncomeRecord::Rental(first) }),
        document(0, IncomeSource::Record { value: crate::IncomeRecord::Rental(page(49)) }),
    ]
    .into_iter()
    .map(|row| crate::IncomeRecordItem::from_income(row).expect("record fixture"))
    .collect();
    if let Ok(directory) = std::env::var("TAX_UNIT19_RENTAL_STAGE_DIR") {
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(std::path::Path::new(&directory).join("seed.avro"), seed.to_avro()).unwrap();
        std::fs::write(std::path::Path::new(&directory).join("answers.avro"), answers.to_avro())
            .unwrap();
    }
    if std::env::var("TAX_UNIT19_RENTAL_STAGE_SEED_ONLY").is_ok() {
        return;
    }
    let request = native.input("PAYLOAD").unwrap_or(answers);
    accepted(&send(&mut session, request));
    let saved = session.draft.clone();
    let actual: Vec<_> = saved
        .income
        .iter()
        .filter(|entry| matches!(entry.income, FederalIncome::Rental { .. }))
        .collect();
    assert_eq!(actual.len(), 2);
    let ids = [actual[0].id, actual[1].id];
    for alternative in [false, true] {
        let calculated = pool(&saved, alternative);
        let first = &calculated.operating_sources[&ids[0]];
        let second = &calculated.operating_sources[&ids[1]];
        assert_eq!(first.alternative, alternative);
        assert_eq!(
            first.before_at_risk_cents,
            if prior { if alternative { -324 } else { -299 } } else { 149 }
        );
        assert_eq!(first.after_at_risk_cents, if prior { -149 } else { 149 });
        assert_eq!(
            first.prior_at_risk_cents,
            if prior { if alternative { 75 } else { 50 } } else { 0 }
        );
        assert_eq!(
            first.prior_passive_cents,
            if prior { if alternative { 200 } else { 100 } } else { 0 }
        );
        assert_eq!(second.before_at_risk_cents, 49);
        assert_eq!(second.after_at_risk_cents, 49);
        assert_eq!(first.filed_ordinary_dollars, calculated.income_by_id.get(&ids[0]).copied());
        assert_eq!(second.filed_ordinary_dollars, Some(0));
    }
    let result = supported(&saved);
    let records = native_draft_records(&saved);
    let draft = draft_from_native_records(saved.tax_year, &records);
    assert_eq!(native_draft_records(&draft), records);
    assert_eq!(draft, saved);
    let mut session = restart(draft);
    let reopened = open_topic(&mut session, TaxSection::Rentals);
    if let Ok(directory) = std::env::var("TAX_UNIT19_RENTAL_STAGE_DIR") {
        write_native_draft_records(
            std::path::Path::new(&directory).join("accepted-record.avro"),
            &records,
        );
        std::fs::write(std::path::Path::new(&directory).join("reopened.avro"), reopened.to_avro())
            .unwrap();
        std::fs::write(
            std::path::Path::new(&directory).join("regular-source.txt"),
            std::format!("{:?}", pool(&saved, false).operating_sources),
        )
        .unwrap();
        std::fs::write(
            std::path::Path::new(&directory).join("amt-source.txt"),
            std::format!("{:?}", pool(&saved, true).operating_sources),
        )
        .unwrap();
    }
    let resave = native.input("REOPEN_PAYLOAD").unwrap_or(reopened);
    accepted(&send(&mut session, resave));
    assert_eq!(session.draft, saved);
    assert_eq!(supported(&session.draft), result);
    let mut correction = open_topic(&mut session, TaxSection::Rentals);
    let P::SaveIncomes { documents, .. } = &mut correction else { unreachable!() };
    let crate::IncomeRecord::Rental(page) = &mut documents.documents[0].document else {
        unreachable!()
    };
    page.rent += 2;
    documents.documents.reverse();
    accepted(&send(&mut session, correction));
    let corrected = pool(&session.draft, false);
    assert_eq!(
        corrected.operating_sources[&ids[0]].before_at_risk_cents,
        if prior { -297 } else { 151 }
    );
    assert_eq!(corrected.operating_sources[&ids[1]].before_at_risk_cents, 49);
    assert_eq!(restart(session.draft.clone()).draft, session.draft);
}
