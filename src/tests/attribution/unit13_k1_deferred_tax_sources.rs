use crate::tests::support::client::{
    draft_from_native_records, native_draft_records, write_native_draft_records,
};
// Native K-1 box20AI source amounts; no section409A tax re-calculation.
use super::*;

fn deferred_return(cents: [i64; 2]) -> FederalDraft {
    let mut d = injured_return((60_000, 10_000), (20_000, 5_000), 0, allocation());
    for (owner, amount) in [Owner::Taxpayer, Owner::Spouse].into_iter().zip(cents) {
        let mut data = ordinary_partner_source(0, false);
        data.deferred_compensation_tax = amount;
        let mut entry = income(FederalIncome::Partnership { data });
        entry.owner = owner;
        // Identical labels cannot substitute for the accepted source owner.
        entry.label = "Synthetic same-name partnership".into();
        add(&mut d, entry);
    }
    d
}

#[test]
fn unit13_k1_deferred_native_tax_is_owned_once_without_changing_income() {
    let mut d = deferred_return([10_000, 30_000]);
    let joint = supported(&d);
    assert_eq!(value(&joint, "agi"), 80_000);
    assert_eq!(value(&joint, "k1_deferred_compensation_tax"), 400);
    assert_eq!(return_line(&joint, "23") / 100, 400);
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        d.injured_spouse.as_mut().unwrap().injured = owner;
        let r = supported(&d);
        let first = usize::from(owner == Owner::Spouse);
        assert_eq!(column(&r, "18", "a"), 400);
        assert_eq!(column(&r, "18", "b"), [100, 300][first]);
        assert_eq!(column(&r, "18", "c"), [100, 300][1 - first]);
        assert!(
            !r.review_notes
                .iter()
                .any(|n| n.display.key == "WARN_8379_OTHER_TAX_OWNER_UNAVAILABLE")
        );
        let _ = share_cents(&r);
        assert_eq!((r.tax, r.refund), (joint.tax, joint.refund));
        assert_eq!(supported(&restart(d.clone()).draft), r);
        d.income.reverse();
        let reordered = supported(&d);
        assert_eq!((reordered.tax, reordered.refund), (r.tax, r.refund));
        assert_eq!(
            [column(&reordered, "18", "b"), column(&reordered, "18", "c")],
            [column(&r, "18", "b"), column(&r, "18", "c")]
        );
    }
}

#[test]
fn unit13_k1_deferred_native_joint_rounding_uses_actual_injured_first() {
    // Literal independent whole-dollar oracles: [first-selected owner][TP,SP].
    for (cents, joint, expected) in [
        ([49, 49], 1, [[0, 1], [1, 0]]),
        ([49, 50], 1, [[0, 1], [0, 1]]),
        ([50, 49], 1, [[1, 0], [1, 0]]),
        ([0, 50], 1, [[0, 1], [0, 1]]),
        ([50, 0], 1, [[1, 0], [1, 0]]),
        ([149, 151], 3, [[1, 2], [1, 2]]),
    ] {
        let mut d = deferred_return(cents);
        for first in 0..2 {
            d.injured_spouse.as_mut().unwrap().injured = [Owner::Taxpayer, Owner::Spouse][first];
            let r = supported(&d);
            let selected = expected[first][first];
            assert_eq!(value(&r, "k1_deferred_compensation_tax"), joint);
            assert_eq!(column(&r, "18", "a"), joint);
            assert_eq!(column(&r, "18", "b"), selected, "{cents:?}, first{first}");
            assert_eq!(column(&r, "18", "c"), joint - selected);
            assert_eq!(column(&r, "13b", "a"), 0);
            let _ = share_cents(&r);
        }
    }
}

#[test]
fn unit13_k1_deferred_typed_projection_rejects_bad_footing() {
    use crate::injured_spouse::DeferredCompensationTaxSources as Sources;
    let source = Sources { joint_dollars: 1, cents: [49, 50] };
    assert_eq!(source.owners(0), Some([0, 1]));
    assert_eq!(source.owners(1), Some([0, 1]));
    assert_eq!(source.owners(2), None);
    assert_eq!(Sources { joint_dollars: 2, ..source }.owners(0), None);
    assert_eq!(Sources { joint_dollars: 1, cents: [49, 49] }.owners(0), Some([0, 1]));
    assert_eq!(Sources { joint_dollars: 1, cents: [49, 49] }.owners(1), Some([1, 0]));
}

#[test]
fn unit13_k1_deferred_shared_liability_has_no_claim_gate_or_late_row_dependency() {
    let baseline = injured_return((60_000, 10_000), (20_000, 5_000), 0, allocation());
    let mut facts = liability_portfolio_facts(&baseline, Some([0; 2]));
    let mut d = deferred_return([10_000, 30_000]);
    d.injured_spouse = None;
    d.checklist.whole_return &= !WholeReturnTopics::INJURED_SPOUSE.0;
    let mut r = supported(&d);
    facts.deferred_compensation_tax = crate::injured_spouse::DeferredCompensationTaxSources {
        joint_dollars: value(&r, "k1_deferred_compensation_tax"),
        cents: [10_000, 30_000],
    };
    // Both actual native K-1 income components are explicitly zero in this
    // fixture; their separate additional-tax statement is not income.
    for entry in &d.income {
        if let FederalIncome::Partnership { data } = &entry.income {
            assert_eq!(data.regular.ordinary, 0);
            assert!(data.income.is_empty());
            facts.other_sources.insert(entry.id, Some(crate::kernel::ratio::exact::whole(0)));
        }
    }
    let totals = actual_liability_totals(&r);
    // The mandatory actual native component is sufficient before late row emission.
    r.lines.retain(|v| v.id != "k1_deferred_compensation_tax");
    let mut spouse_claim = allocation();
    spouse_claim.injured = Owner::Spouse;
    for claim in [None, Some(allocation()), Some(spouse_claim)] {
        let projection = crate::injured_spouse::owners::project_sources(
            &d,
            &r,
            claim.as_ref(),
            &facts,
            Some(&totals),
        )
        .unwrap();
        assert_eq!(crate::injured_spouse::separate_taxes(&d, &projection), Some([5123, 693]));
    }
    facts.deferred_compensation_tax.cents = [0; 2];
    let projection =
        crate::injured_spouse::owners::project_sources(&d, &r, None, &facts, Some(&totals))
            .unwrap();
    assert_eq!(crate::injured_spouse::separate_taxes(&d, &projection), None);
}

#[test]
fn unit13_k1_deferred_actual_client_save_reopen_restart_correction() {
    extern crate std;
    use crate::tests::cross::interview::{accepted, document, open_topic};
    let initial_draft = injured_return((60_000, 10_000), (20_000, 5_000), 0, allocation());
    let mut session = Interview::new(TEST_USER, initial_draft);
    let initial_seed = open_topic(&mut session, TaxSection::PassThrough);
    let mut initial_answers = initial_seed.clone();
    let P::SaveIncomes { documents, .. } = &mut initial_answers else { panic!() };
    for (owner, cents) in [(Owner::Taxpayer, 10_000), (Owner::Spouse, 30_000)] {
        let mut data = ordinary_partner_source(0, false);
        data.deferred_compensation_tax = cents;
        let page =
            crate::K1Partnership::from_record("Synthetic same-name partnership", &data).unwrap();
        let mut row = document(0, IncomeDocument::K1Partnership(page));
        row.owner = owner;
        documents.documents.push(row);
    }
    let correction = std::env::var("TAX_UNIT13_DEFERRED_CORRECTION").as_deref() == Ok("1");
    let read = |file: alloc::string::String| P::from_avro(&std::fs::read(file).unwrap()).unwrap();
    let write = |name: &str, request: &P| {
        if let Ok(dir) = std::env::var("TAX_UNIT13_DEFERRED_DIR") {
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(std::path::Path::new(&dir).join(name), request.to_avro()).unwrap();
        }
    };
    if correction {
        let first = std::env::var("TAX_UNIT13_DEFERRED_INITIAL_PAYLOAD")
            .map_or(initial_answers.clone(), read);
        accepted(&send(&mut session, first));
    }
    let seed =
        if correction { open_topic(&mut session, TaxSection::PassThrough) } else { initial_seed };
    if let Ok(input) = std::env::var("TAX_UNIT13_DEFERRED_CALC_INPUT") {
        let Reply::Response(response) = send(&mut session, read(input)) else {
            panic!("normal K-1 calculation response")
        };
        std::fs::write(
            std::env::var("TAX_UNIT13_DEFERRED_CALC_OUTPUT").unwrap(),
            response.to_avro(),
        )
        .unwrap();
        return;
    }
    write("seed.avro", &seed);
    let mut answers = if correction { seed } else { initial_answers };
    if correction {
        let P::SaveIncomes { documents, .. } = &mut answers else { panic!() };
        documents.documents[0].owner = Owner::Spouse;
        let IncomeDocument::K1Partnership(page) = &mut documents.documents[0].document else {
            panic!()
        };
        page.more.deferred_compensation_tax = 20_000;
    }
    write("answers.avro", &answers);
    if std::env::var("TAX_UNIT13_DEFERRED_SEED_ONLY").as_deref() == Ok("1") {
        return;
    }
    let payload = std::env::var("TAX_UNIT13_DEFERRED_PAYLOAD").map_or(answers, read);
    accepted(&send(&mut session, payload));
    let ids = session
        .draft
        .income
        .iter()
        .filter(|e| matches!(e.income, FederalIncome::Partnership { .. }))
        .map(|e| e.id)
        .collect::<Vec<_>>();
    assert_eq!(ids, vec![3, 4]);
    let r = supported(&session.draft);
    assert_eq!(value(&r, "agi"), 80_000);
    assert_eq!(column(&r, "18", "b"), if correction { 0 } else { 100 });
    assert_eq!(column(&r, "18", "c"), if correction { 500 } else { 300 });
    assert_eq!(value(&r, "k1_deferred_compensation_tax"), if correction { 500 } else { 400 });
    let _ = share_cents(&r);
    let tax_year = session.draft.tax_year;
    let stored = native_draft_records(&session.draft);
    let decoded = draft_from_native_records(tax_year, &stored);
    assert_eq!(native_draft_records(&decoded), stored);
    assert_eq!(decoded, session.draft);
    if let Ok(dir) = std::env::var("TAX_UNIT13_DEFERRED_DIR") {
        write_native_draft_records(std::path::Path::new(&dir).join("persisted.avro"), &stored);
    }
    session = restart(session.draft);
    let reopened = open_topic(&mut session, TaxSection::PassThrough);
    write("reopened.avro", &reopened);
    accepted(&send(&mut session, reopened));
    assert_eq!(supported(&session.draft), r);
    // The ordinary native control also exercises correction and another restart.
    if !correction {
        let mut next = open_topic(&mut session, TaxSection::PassThrough);
        let P::SaveIncomes { documents, .. } = &mut next else { panic!() };
        documents.documents[0].owner = Owner::Spouse;
        let IncomeDocument::K1Partnership(page) = &mut documents.documents[0].document else {
            panic!()
        };
        page.more.deferred_compensation_tax = 20_000;
        accepted(&send(&mut session, next));
        let changed = supported(&session.draft);
        assert_eq!(value(&changed, "k1_deferred_compensation_tax"), 500);
        assert_eq!([column(&changed, "18", "b"), column(&changed, "18", "c")], [0, 500]);
        assert_eq!(supported(&restart(session.draft).draft), changed);
    }
}

#[test]
fn unit13_k1_deferred_tax_survives_actual_basis_suspension() {
    let mut d = deferred_return([10_000, 30_000]);
    let FederalIncome::Partnership { data } = &mut d.income[2].income else { panic!() };
    data.regular.ordinary = -100_000;
    data.regular.basis.opening = 0;
    data.alternative = Some(data.regular.clone());
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 80_000);
    assert_eq!([column(&r, "18", "b"), column(&r, "18", "c")], [100, 300]);
    assert_eq!(return_line(&r, "23") / 100, 400);
    let _ = share_cents(&r);
}

#[test]
fn unit13_k1_deferred_community_input_is_divided_only_at_native_owner() {
    let mut d = community_w2_draft(6_000_000, 2_000_000, 1_000_000, 500_000);
    // Existing admitted MFS source: two records of the filing taxpayer.
    // A nonfiling spouse's K-1 currently has a separate native SE refusal.
    for owner in [Owner::Taxpayer; 2] {
        let mut data = ordinary_partner_source(0, false);
        data.deferred_compensation_tax = 10_050;
        let mut entry = income(FederalIncome::Partnership { data });
        entry.owner = owner;
        entry.community = CommunityShare::Community;
        add(&mut d, entry);
    }
    // Each native K-1 input contributes5,025c, not a second division of
    // the already-divided total. dollars(5,025+5,025)=101, not50/102.
    let r = supported(&d);
    assert_eq!(value(&r, "k1_deferred_compensation_tax"), 101);
    assert_eq!(return_line(&r, "23") / 100, 101);
    assert_eq!(supported(&restart(d.clone()).draft), r);
}
