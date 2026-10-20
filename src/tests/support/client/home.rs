use super::TEST_USER;
use crate::{FederalDraft, HomeOfficeSite, HomeOffices, TaxInterviewProtocol as P};
use alloc::{vec, vec::Vec};

pub(in crate::tests) fn save_home_offices(d: &FederalDraft, sites: Vec<HomeOfficeSite>) -> P {
    P::SaveHomeOffices {
        user: TEST_USER,
        cookie: d.cookie.clone(),
        revision: d.revision,
        page: crate::tests::cross::interview::PAGE_INCOME,
        previous: crate::tests::cross::interview::PAGE_INCOME,
        review: true,
        estimate: vec![],
        warnings: vec![],
        progress: vec![],
        mortgage_loans: vec![],
        home_activities: vec![],
        casualty_events: vec![],
        allowed_home_owners: vec![],
        value: HomeOffices { sites: sites.into_iter().map(Into::into).collect() },
    }
}
