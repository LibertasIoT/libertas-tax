// Native durable property identities use the one accepted-parent Save lifecycle.
use super::*;
use crate::investment_credit::{k1_property_credit_sources, k1_property_ids};

fn properties() -> Vec<crate::K1InvestmentProperty> {
    [1_000_001, 2_000_001]
        .into_iter()
        .map(|basis| crate::K1InvestmentProperty::Energy {
            own_form: None,
            basis,
            rate: crate::EnergyRate {
                increased_rate: true,
                domestic_content: false,
                energy_community: false,
            },
            id: 0,
        })
        .collect()
}
fn page(kind: &str) -> IncomeDocument {
    match kind {
        "partner" => {
            let mut page = k1_1065("Same synthetic parent");
            page.more.investment_property = properties();
            page.more.investment_confirmed = Some(crate::K1InvestmentConfirmation::Yes);
            IncomeDocument::K1Partnership(page)
        }
        "scorp" => {
            let mut page = k1_1120s("Same synthetic parent");
            page.more.investment_property = properties();
            page.more.investment_confirmed = Some(crate::K1InvestmentConfirmation::Yes);
            IncomeDocument::K1Scorp(page)
        }
        "estate" => {
            let mut page = k1_1041("Same synthetic parent");
            page.investment_property = properties()
                .into_iter()
                .map(|property| crate::EstateInvestmentPropertyPage {
                    property,
                    activities: vec![],
                })
                .collect();
            IncomeDocument::K1EstateTrust(page)
        }
        _ => unreachable!(),
    }
}
fn properties_mut(page: &mut IncomeDocument) -> Vec<&mut crate::K1InvestmentProperty> {
    match page {
        IncomeDocument::K1Partnership(page) => page.more.investment_property.iter_mut().collect(),
        IncomeDocument::K1Scorp(page) => page.more.investment_property.iter_mut().collect(),
        IncomeDocument::K1EstateTrust(page) => page
            .investment_property
            .iter_mut()
            .map(|v| &mut v.property)
            .collect(),
        _ => unreachable!(),
    }
}
fn reverse(page: &mut IncomeDocument) {
    match page {
        IncomeDocument::K1Partnership(page) => page.more.investment_property.reverse(),
        IncomeDocument::K1Scorp(page) => page.more.investment_property.reverse(),
        IncomeDocument::K1EstateTrust(page) => page.investment_property.reverse(),
        _ => unreachable!(),
    }
}
fn source(session: &Interview) -> &FederalIncomeEntry {
    session
        .draft
        .income
        .iter()
        .find(|e| !k1_property_ids(&e.income).is_empty())
        .unwrap()
}
fn save(session: &mut Interview, rows: Vec<IncomeDocumentItem>) -> Reply {
    let request = save_topic(&session.draft.clone(), TaxSection::PassThrough, rows);
    send(session, request)
}
#[test]
fn unit16_k1_property_ids_assign_once_and_conserve_native_basis_output() {
    for kind in ["partner", "scorp", "estate"] {
        let d = complete(FederalStatus::Single);
        let mut session = Interview::new(TEST_USER, d);
        accepted(&save(&mut session, vec![document(0, page(kind))]));
        let source = source(&session);
        let ids = k1_property_ids(&source.income);
        assert_eq!(ids, vec![3, 4]); // parent2, then the actual two properties.
        let outputs = k1_property_credit_sources(&source.income).unwrap();
        assert_eq!(
            outputs.iter().map(|v| v.property_id).collect::<Vec<_>>(),
            ids
        );
        // Native credits round3000.003/6000.003 to3000/6000; exactly half
        // of the corresponding native-dollar credits reduces basis in cents.
        assert_eq!(
            outputs
                .iter()
                .map(|v| v.basis_decrease_cents)
                .collect::<Vec<_>>(),
            vec![150_000, 300_000]
        );
        let canonical = match &source.income {
            FederalIncome::Partnership { data } => {
                crate::investment_credit::k1_basis_decrease(&data.investment_property)
            }
            FederalIncome::Scorp { data } => {
                crate::investment_credit::k1_basis_decrease(&data.investment_property)
            }
            FederalIncome::EstateTrustPortfolio { data } => data
                .investment_property
                .iter()
                .map(|v| v.property.basis_decrease())
                .sum(),
            _ => unreachable!(),
        };
        assert_eq!(
            outputs.iter().map(|v| v.basis_decrease_cents).sum::<i64>(),
            canonical
        );
        let result = estimate(&session.draft);
        let saved = session.draft.clone();
        let rows = page_documents(&open_topic(&mut session, TaxSection::PassThrough)).to_vec();
        accepted(&save(&mut session, rows));
        assert_eq!(session.draft, saved);
        assert_eq!(estimate(&restart(saved).draft), result);
    }
}
#[test]
fn unit16_k1_property_reorder_correction_and_replacement_keep_actual_identity() {
    for kind in ["partner", "scorp", "estate"] {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        accepted(&save(&mut session, vec![document(0, page(kind))]));
        let mut rows = page_documents(&open_topic(&mut session, TaxSection::PassThrough)).to_vec();
        reverse(&mut rows[0].document);
        if let crate::K1InvestmentProperty::Energy { basis, .. } =
            properties_mut(&mut rows[0].document)
                .into_iter()
                .next()
                .unwrap()
        {
            *basis = 3_000_001;
        }
        accepted(&save(&mut session, rows));
        assert_eq!(k1_property_ids(&source(&session).income), vec![4, 3]);
        assert_eq!(
            k1_property_credit_sources(&source(&session).income).unwrap()[0].basis_decrease_cents,
            450_000
        );
        let mut rows = page_documents(&open_topic(&mut session, TaxSection::PassThrough)).to_vec();
        // Imported/replaced unkeyed rows are genuinely new sources. Neither
        // matching amount nor position may silently reuse an old property ID.
        for p in properties_mut(&mut rows[0].document) {
            *p.id_mut() = 0;
        }
        accepted(&save(&mut session, rows));
        assert_eq!(k1_property_ids(&source(&session).income), vec![5, 6]);
    }
}
#[test]
fn unit16_k1_property_foreign_duplicate_and_guessed_ids_reject_atomically() {
    for kind in ["partner", "scorp", "estate"] {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        accepted(&save(
            &mut session,
            vec![document(0, page(kind)), document(0, page(kind))],
        ));
        let rows = page_documents(&open_topic(&mut session, TaxSection::PassThrough)).to_vec();
        let first = k1_property_ids(
            &session
                .draft
                .income
                .iter()
                .find(|e| e.id == rows[0].id)
                .unwrap()
                .income,
        );
        for bad in [first[0], session.draft.next_id, rows[0].id] {
            let mut forged = rows.clone();
            *properties_mut(&mut forged[1].document)[0].id_mut() = bad;
            let before = session.draft.clone();
            rejected_with(
                save(&mut session, forged),
                "ISSUE_INTERVIEW_A_RECORD_NO_LONGER_BELONGS_TO_51E52C2C",
            );
            assert_eq!(session.draft, before);
        }
        let mut forged = rows;
        let first = properties_mut(&mut forged[0].document)[0].id();
        *properties_mut(&mut forged[0].document)[1].id_mut() = first;
        let before = session.draft.clone();
        rejected_with(
            save(&mut session, forged),
            "ISSUE_INTERVIEW_A_RECORD_IDENTITY_IS_REPEATED_IN_EE6DC425",
        );
        assert_eq!(session.draft, before);
    }
}
#[test]
fn unit16_k1_property_actual_client_save_reopen_restart() {
    extern crate std;
    let kind = std::env::var("TAX_UNIT16_ID_KIND").unwrap_or_else(|_| "partner".into());
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let seed = open_topic(&mut session, TaxSection::PassThrough);
    if let Ok(input) = std::env::var("TAX_UNIT16_ID_CALC_INPUT") {
        // A correction calculation belongs to the genuinely accepted first
        // Save. Restore that exact payload through the public writer so its
        // parent/property identities have the same native authorization.
        if let Ok(path) = std::env::var("TAX_UNIT16_ID_ACCEPTED") {
            let request = P::from_avro(&std::fs::read(path).unwrap()).unwrap();
            accepted(&send(&mut session, request));
            assert_eq!(k1_property_ids(&source(&session).income), vec![3, 4]);
        }
        let request = P::from_avro(&std::fs::read(input).unwrap()).unwrap();
        let Reply::Response(response) = send(&mut session, request) else {
            panic!("normal calculation response")
        };
        std::fs::write(
            std::env::var("TAX_UNIT16_ID_CALC_OUTPUT").unwrap(),
            response.to_avro(),
        )
        .unwrap();
        return;
    }
    let mut answers = seed.clone();
    let P::SaveIncomes { documents, .. } = &mut answers else {
        panic!()
    };
    documents.documents.push(document(0, page(&kind)));
    let dir = std::env::var("TAX_UNIT16_ID_DIR")
        .ok()
        .map(std::path::PathBuf::from);
    if std::env::var("TAX_UNIT16_ID_SEED_ONLY").as_deref() == Ok("1") {
        let dir = dir.unwrap();
        std::fs::write(dir.join("seed.avro"), seed.to_avro()).unwrap();
        std::fs::write(dir.join("answers.avro"), answers.to_avro()).unwrap();
        return;
    }
    let request = if let Some(dir) = &dir {
        P::from_avro(&std::fs::read(dir.join("payload.avro")).unwrap()).unwrap()
    } else {
        answers
    };
    accepted(&send(&mut session, request));
    assert_eq!(k1_property_ids(&source(&session).income), vec![3, 4]);
    let result = estimate(&session.draft);
    let saved = session.draft.clone();
    session = restart(saved.clone());
    assert_eq!(session.draft, saved);
    assert_eq!(estimate(&session.draft), result);
    let reopened = open_topic(&mut session, TaxSection::PassThrough);
    if let Some(dir) = &dir {
        std::fs::write(dir.join("reopened.avro"), reopened.to_avro()).unwrap();
    }
    accepted(&send(&mut session, reopened));
    assert_eq!(session.draft, saved);
    if let Ok(path) = std::env::var("TAX_UNIT16_ID_RESAVE_PAYLOAD") {
        let request = P::from_avro(&std::fs::read(path).unwrap()).unwrap();
        accepted(&send(&mut session, request));
        assert_eq!(k1_property_ids(&source(&session).income), vec![4, 3]);
        assert_eq!(session.draft.next_id, saved.next_id);
        assert_eq!(
            k1_property_credit_sources(&source(&session).income).unwrap()[0].basis_decrease_cents,
            450_000
        );
        let corrected = session.draft.clone();
        session = restart(corrected.clone());
        assert_eq!(session.draft, corrected);
        let reopened = open_topic(&mut session, TaxSection::PassThrough);
        if let Some(dir) = &dir {
            std::fs::write(dir.join("corrected-reopened.avro"), reopened.to_avro()).unwrap();
        }
        accepted(&send(&mut session, reopened));
        assert_eq!(session.draft, corrected);
    }
}

#[test]
fn unit16_k1_property_all_variants_keep_identity_when_community_amounts_divide() {
    let mut properties = properties();
    properties.extend([
        crate::K1InvestmentProperty::RehabilitationRental {
            own_form: None,
            expenditures: 10_000_001,
            placed_year: 2026,
            id: 0,
        },
        crate::K1InvestmentProperty::RehabilitationOther {
            own_form: None,
            expenditures: 10_000_001,
            placed_year: 2025,
            id: 0,
        },
        crate::K1InvestmentProperty::CleanElectricity {
            own_form: None,
            basis: 1_000_001,
            storage: true,
            rate: crate::EnergyRate {
                increased_rate: false,
                domestic_content: false,
                energy_community: false,
            },
            id: 0,
        },
    ]);
    let mut page = k1_1065("Synthetic divided source");
    page.more.investment_property = properties;
    let mut original = FederalIncome::Partnership {
        data: page.record(),
    };
    assert!(k1_property_credit_sources(&original).is_none());
    for (i, id) in crate::investment_credit::k1_property_ids_mut(&mut original)
        .into_iter()
        .enumerate()
    {
        *id = i as i64 + 10;
    }
    let mut draft = complete(FederalStatus::Separate);
    draft.setup.as_mut().unwrap().community_property = crate::CommunityProperty::Yes {
        state: crate::CommunityState::Texas,
        rule: crate::CommunityRule::Halves,
    };
    let share = crate::community::share(
        draft.setup.as_ref().unwrap(),
        Owner::Taxpayer,
        CommunityShare::Community,
        &original,
    )
    .unwrap();
    let divided = crate::community::divided(share, &original);
    assert_eq!(k1_property_ids(&original), vec![10, 11, 12, 13, 14]);
    assert_eq!(k1_property_ids(&divided), k1_property_ids(&original));
    let amounts = k1_property_credit_sources(&original)
        .unwrap()
        .into_iter()
        .map(|v| v.basis_decrease_cents)
        .collect::<Vec<_>>();
    assert_eq!(amounts, vec![150_000, 300_000, 2_000_000, 0, 30_000]);
    let FederalIncome::Partnership { data } = divided.as_ref() else {
        unreachable!()
    };
    let crate::K1InvestmentProperty::RehabilitationRental {
        own_form: None,
        expenditures,
        placed_year,
        id,
    } = &data.investment_property[2]
    else {
        unreachable!()
    };
    assert_eq!((*expenditures, *placed_year, *id), (5_000_001, 2026, 12));
}
