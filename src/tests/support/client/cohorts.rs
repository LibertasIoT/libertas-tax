use crate::tests::*;
pub(in crate::tests) fn save_existing_business(
    session: &mut Interview,
    id: i64,
    page: Business,
) -> Reply {
    let request = crate::tests::cross::interview::save_topic(
        session,
        TaxSection::SelfEmployment,
        vec![crate::tests::cross::interview::document(
            id,
            IncomeSource::Record { value: crate::IncomeRecord::Business(page) },
        )],
    );
    send(session, request)
}
