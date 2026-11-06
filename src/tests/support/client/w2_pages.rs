// w2 items of src/tests/support/client/income_pages.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(in crate::tests) fn save_wage_fixture(session: &mut Interview, id: i64, form: W2) -> Reply {
    let request = save_documents(
        session,
        vec![crate::tests::support::builders::native_income::document(
            id,
            IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(form)) },
        )],
    );
    save_source_fixture_request(session, request)
}

pub(in crate::tests) fn reopened_wage_fixture(session: &mut Interview) -> (i64, W2) {
    let page = open_document_center(session);
    let row = page_documents(&page)
        .into_iter()
        .find(|row| {
            matches!(row.document, IncomeSource::Wage { value: crate::WageDocument::W2(_) })
        })
        .expect("accepted W-2 source");
    let IncomeSource::Wage { value: crate::WageDocument::W2(form) } = row.document else {
        unreachable!()
    };
    let source = session.draft.income.iter().find(|entry| entry.id == row.id).unwrap();
    let FederalIncome::Wage { data: WageSource::W2(source) } = &source.income else {
        unreachable!()
    };
    assert_eq!(source.form, form);
    (row.id, source.native())
}
