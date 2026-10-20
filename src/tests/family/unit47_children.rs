// Unit47 Children residence locators reuse the actual candidate's return facts.
extern crate std;
use crate::tests::cross::interview::{navigate, reopened_people, row};
use super::*;

pub(super) fn session(final_return: bool) -> Interview {
    let mut d = complete(FederalStatus::Head);
    if final_return {
        // January through April inclusive: 31 + 28 + 31 + 30 = 120 days.
        d.people.as_mut().unwrap().taxpayer.died = Some(FinalReturn {
            bonds: None,
            estate_medical: 0,
            date: 20260430,
            ordinary: true,
        });
    }
    Interview::new(TEST_USER, d)
}
pub(super) fn child(first: &str, days: u32) -> Dependent {
    let mut child = row(first, "Sample", 20180214);
    child.lived = LivedWithYou::Days { days };
    child
}
pub(super) fn page(s: &mut Interview, people: Vec<Dependent>) -> P {
    let request = navigate(&s.draft, TaxSection::Children);
    let mut page = match send(s, request) {
        Reply::Edit(page @ P::SaveChildren { .. }) => page,
        // An empty accepted list opens the normal first-composition response.
        // Preserve its actual context while supplying the complete request.
        Reply::Response(P::BeginChildren {
            cookie,
            revision,
            page,
            previous,
            review,
            estimate,
            progress,
            allowed_lived,
            days_alive,
        }) => P::SaveChildren {
            user: TEST_USER,
            cookie,
            revision,
            page,
            previous,
            review,
            estimate: Some(estimate),
            warnings: vec![],
            progress: Some(progress),
            allowed_lived,
            days_alive,
            dependents: Dependents { people: vec![] },
        },
        other => panic!("Children editor: {other:?}"),
    };
    let P::SaveChildren { dependents, .. } = &mut page else {
        unreachable!()
    };
    dependents.people = people;
    page
}
fn refused_unchanged(s: &mut Interview, request: P, key: &str, path: &str) {
    let before = s.draft.clone();
    let dirty = s.dirty;
    let response = send(s, request.clone());
    if let Ok(path) = std::env::var("TAX_UNIT47_CHILDREN_REQUEST") {
        std::fs::write(path, request.to_avro()).unwrap();
        let Reply::Response(value) = &response else {
            panic!("native refusal")
        };
        std::fs::write(
            std::env::var("TAX_UNIT47_CHILDREN_RESPONSE").unwrap(),
            value.to_avro(),
        )
        .unwrap();
    }
    let Reply::Response(P::Problem { message, field }) = response else {
        panic!("expected {key}")
    };
    assert_eq!(message, libertas::libertas_formatted_text(key, &[]));
    assert_eq!(field.as_deref(), Some(path));
    assert_eq!(s.draft, before);
    assert_eq!(s.dirty, dirty);
    // Equality includes revision, accepted people, and next_id: no failed
    // candidate can allocate IDs for either new submitted child.
}

#[test]
fn unit47_children_actual_day_bound_locates_second_child_for_save_and_estimate() {
    let mut s = session(true);
    let people = vec![child("First", 61), child("Second", 121)];
    let mut request = page(&mut s, people.clone());
    let P::SaveChildren {
        allowed_lived,
        days_alive,
        ..
    } = &mut request
    else {
        unreachable!()
    };
    assert_eq!(*days_alive, 120);
    // A forged read-only echo cannot relax the actual return's 120-day bound.
    *days_alive = 366;
    *allowed_lived = vec![libertas_macros::variant_index!(LivedWithYou::Days) as i32];
    refused_unchanged(
        &mut s,
        request,
        "RESIDENCE_DAYS_ABOVE_DAYS_ALIVE",
        "[dependents/people/1/lived/days]",
    );
    refused_unchanged(
        &mut s,
        P::EstimateDependents {
            user: TEST_USER,
            allowed_lived: vec![libertas_macros::variant_index!(LivedWithYou::Days) as i32],
            days_alive: 366,
            dependents: Dependents { people },
        },
        "RESIDENCE_DAYS_ABOVE_DAYS_ALIVE",
        "[dependents/people/1/lived/days]",
    );
}

#[test]
fn unit47_children_actual_residence_choices_locate_second_child() {
    let mut s = session(true);
    let mut months = row("Second", "Sample", 20190301);
    months.lived = LivedWithYou::PartOfYear { months: 3 };
    let mut request = page(&mut s, vec![child("First", 61), months]);
    let P::SaveChildren { allowed_lived, .. } = &mut request else {
        unreachable!()
    };
    // The echoed choice catalog cannot authorize a month answer on this source.
    allowed_lived.push(libertas_macros::variant_index!(LivedWithYou::PartOfYear) as i32);
    refused_unchanged(
        &mut s,
        request,
        "RESIDENCE_ANSWER_NOT_ALLOWED",
        "[dependents/people/1/lived]",
    );
    let mut s = session(false);
    let request = page(
        &mut s,
        vec![row("First", "Sample", 20180214), child("Second", 61)],
    );
    refused_unchanged(
        &mut s,
        request,
        "RESIDENCE_ANSWER_NOT_ALLOWED",
        "[dependents/people/1/lived]",
    );
}

#[test]
fn unit47_children_day_bound_equality_saves_reopens_resaves_and_restarts() {
    let mut s = session(true);
    let request = page(&mut s, vec![child("First", 61), child("Second", 120)]);
    assert!(!matches!(
        send(&mut s, request),
        Reply::Response(P::Problem { .. })
    ));
    assert_eq!(
        s.draft.dependents[0].residency,
        DependentResidency::Days { days: 61 }
    );
    assert_eq!(
        s.draft.dependents[1].residency,
        DependentResidency::Days { days: 120 }
    );
    let records = s.draft.dependents.clone();
    assert!(records.iter().all(|person| person.id > 0));
    let reopened = reopened_people(&mut s);
    assert_eq!(reopened[0].lived, LivedWithYou::Days { days: 61 });
    assert_eq!(reopened[1].lived, LivedWithYou::Days { days: 120 });
    let request = page(&mut s, reopened);
    assert!(!matches!(
        send(&mut s, request),
        Reply::Response(P::Problem { .. })
    ));
    assert_eq!(s.draft.dependents, records);
    assert_eq!(restart(s.draft.clone()).draft, s.draft);
}
