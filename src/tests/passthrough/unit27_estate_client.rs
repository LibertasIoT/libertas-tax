use crate::tests::cross::interview::{accepted, document, open_topic};
use super::*;

// Optional Avro transport uses the ordinary editor, callbacks and Save path.
// The native expected source is only a list of visible synthetic answers;
// the actual client must construct its row from the empty editor's template.
#[test]
fn unit27_estate_fresh_client_saves_whole_source_and_reopens_after_restart() {
    extern crate std;
    let owner = if std::env::var("TAX_B11_ESTATE_OWNER").as_deref() == Ok("spouse") {
        Owner::Spouse
    } else {
        Owner::Taxpayer
    };
    let source = super::unit27_estate_community::source();
    let mut draft = super::unit27_estate_community::draft(owner);
    draft
        .income
        .retain(|entry| !matches!(entry.income, FederalIncome::EstateTrustPortfolio { .. }));
    let mut session = Interview::new(TEST_USER, draft);
    let seed = open_topic(&mut session, TaxSection::PassThrough);
    let mut request = seed.clone();
    let P::SaveIncomes {
        documents,
        allowed_community,
        ..
    } = &mut request
    else {
        unreachable!()
    };
    assert!(documents.documents.is_empty());
    let mut row = document(
        0,
        IncomeDocument::K1EstateTrust(
            K1EstateTrust::from_record("Synthetic componentized trust", &source).unwrap(),
        ),
    );
    row.owner = owner;
    row.community = CommunityShare::Separate;
    row.allowed_community = allowed_community.clone();
    documents.documents.push(row);
    if let Ok(path) = std::env::var("TAX_B11_ESTATE_CALC_INPUT") {
        let calculation = P::from_avro(&std::fs::read(path).unwrap()).unwrap();
        assert!(matches!(
            calculation,
            P::EstimateDocuments { .. }
                | P::DocumentPropertyOptions { .. }
                | P::PreviewDocument { .. }
        ));
        let before = session.draft.clone();
        let Reply::Response(reply) = send(&mut session, calculation) else {
            panic!("expected ordinary document calculation response")
        };
        assert_eq!(session.draft, before);
        std::fs::write(
            std::env::var("TAX_B11_ESTATE_CALC_OUTPUT").unwrap(),
            reply.to_avro(),
        )
        .unwrap();
        return;
    }
    for (name, value) in [("SEED", &seed), ("ANSWERS", &request)] {
        if let Ok(path) = std::env::var(alloc::format!("TAX_B11_ESTATE_{name}")) {
            std::fs::write(path, value.to_avro()).unwrap();
        }
    }
    if let Ok(path) = std::env::var("TAX_B11_ESTATE_PAYLOAD") {
        request = P::from_avro(&std::fs::read(path).unwrap()).unwrap();
        assert!(matches!(request, P::SaveIncomes { .. }));
    }
    accepted(&send(&mut session, request));
    let entry = session
        .draft
        .income
        .iter()
        .find(|entry| matches!(entry.income, FederalIncome::EstateTrustPortfolio { .. }))
        .unwrap();
    assert!(entry.id > 0);
    assert_eq!(entry.owner, owner);
    assert_eq!(entry.community, CommunityShare::Separate);
    assert_eq!(
        entry.income,
        FederalIncome::EstateTrustPortfolio {
            data: source.clone()
        }
    );
    let original = entry.clone();
    let projected = crate::community::record(&session.draft, entry);
    let FederalIncome::EstateTrustPortfolio { data } = projected.as_ref() else {
        unreachable!()
    };
    assert_eq!(
        crate::estate_trust::investment_adjustment(data),
        Some([0; 2])
    );
    assert_eq!(data.interest, 50_000);
    assert_eq!(data.ordinary_dividends, 20_000);
    assert_eq!(data.qualified_dividends, 15_000);
    assert_eq!(
        (data.short_gain, data.long_gain),
        if owner == Owner::Taxpayer {
            (20_000, 100_000)
        } else {
            (0, 0)
        }
    );
    let report = supported(&session.draft);
    let reopened = open_topic(&mut session, TaxSection::PassThrough);
    if let Ok(path) = std::env::var("TAX_B11_ESTATE_REOPEN") {
        std::fs::write(path, reopened.to_avro()).unwrap();
    }
    let P::SaveIncomes { documents, .. } = reopened else {
        unreachable!()
    };
    let IncomeDocument::K1EstateTrust(page) = &documents.documents[0].document else {
        unreachable!()
    };
    assert_eq!(page.record(), source);
    let mut restarted = restart(session.draft.clone());
    assert!(restarted.draft.income.contains(&original));
    assert_eq!(supported(&restarted.draft), report);
    let P::SaveIncomes { documents, .. } = open_topic(&mut restarted, TaxSection::PassThrough)
    else {
        unreachable!()
    };
    let IncomeDocument::K1EstateTrust(page) = &documents.documents[0].document else {
        unreachable!()
    };
    assert_eq!(page.record(), source);
}
