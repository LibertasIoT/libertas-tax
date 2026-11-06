use super::*;
use crate::document_name_matcher::{
    RecipientDomain, eligible_recipients, match_eligible_recipient,
};

#[test]
fn printed_names_normalize_order_case_punctuation_and_compatible_middle_initials() {
    let draft = document_recipient_household(FederalStatus::Single);
    for printed in [
        "O'NEIL-SMITH, ALEX M.",
        "Alex Morgan O'Neil-Smith",
        " alex  m.  O'Neil-Smith ",
        "Alex ONeilSmith",
        "Alex Morgan ONeil Smith",
        "Alex Morgan ONeil - Smith",
        "O'Neil-Smith Alex",
    ] {
        assert_eq!(
            match_eligible_recipient(&draft, printed, RecipientDomain::Earner),
            Some(-1),
            "{printed}"
        );
    }
    for printed in [
        "",
        "A. O'Neil-Smith",
        "Alex T. O'Neil-Smith",
        "Alex Taylor O'Neil-Smith",
        "Alex Morgan Smith",
        "Unknown Person",
        "A Morgan ONeilSmith",
        "Alec Morgan ONeilSmith",
        "Alex Morgan Morgan ONeilSmith",
    ] {
        assert_eq!(
            match_eligible_recipient(&draft, printed, RecipientDomain::Earner),
            None,
            "{printed}"
        );
    }
}

#[test]
fn matching_ambiguity_is_decided_within_the_document_eligible_people() {
    let mut draft = document_recipient_household(FederalStatus::Joint);
    draft.dependents[0].first_name = "Alex Morgan".into();
    let printed = "Alex M. O'Neil-Smith";
    assert_eq!(match_eligible_recipient(&draft, printed, RecipientDomain::Earner), Some(-1));
    assert_eq!(match_eligible_recipient(&draft, printed, RecipientDomain::Student), None);
    draft.dependents[0].first_name = "Casey".into();
    assert_eq!(
        match_eligible_recipient(&draft, "Casey O'Neil-Smith", RecipientDomain::Student),
        Some(draft.dependents[0].id)
    );
    assert_eq!(
        match_eligible_recipient(&draft, "Casey O'Neil-Smith", RecipientDomain::Earner),
        None
    );
}

#[test]
fn joint_printed_names_require_a_joint_capable_document() {
    let mut draft = document_recipient_household(FederalStatus::Joint);
    for printed in [
        "Alex Morgan O'Neil-Smith & Jordan Lee O'Neil-Smith",
        "Jordan O'Neil-Smith and Alex O'Neil-Smith",
        "Alex and Jordan O'Neil-Smith",
        "Alex\tAND  Jordan ONeil Smith",
    ] {
        assert_eq!(
            match_eligible_recipient(&draft, printed, RecipientDomain::Account),
            Some(-3),
            "{printed}"
        );
        assert_eq!(
            match_eligible_recipient(&draft, printed, RecipientDomain::Earner),
            None,
            "{printed}"
        );
    }
    draft.people.as_mut().unwrap().spouse.as_mut().unwrap().last_name = "ONeil Smith".into();
    assert_eq!(
        match_eligible_recipient(&draft, "Alex and Jordan ONeil Smith", RecipientDomain::Account),
        Some(-3)
    );
}

#[test]
fn recipient_catalogs_follow_document_kind_and_reportable_spouse_items() {
    for (status, domain, expected) in [
        (FederalStatus::Single, RecipientDomain::Earner, vec![-1]),
        (FederalStatus::Single, RecipientDomain::Account, vec![-1]),
        (FederalStatus::Joint, RecipientDomain::Earner, vec![-1, -2]),
        (FederalStatus::Joint, RecipientDomain::Account, vec![-1, -2, -3]),
        (FederalStatus::Separate, RecipientDomain::Earner, vec![-1]),
    ] {
        let draft = document_recipient_household(status);
        assert_eq!(eligible_recipients(&draft, domain), expected);
    }
    let mut draft = document_recipient_household(FederalStatus::Separate);
    assert_eq!(
        match_eligible_recipient(&draft, "Jordan O'Neil-Smith", RecipientDomain::Earner),
        None
    );
    draft.people.as_mut().unwrap().spouse = None.into();
    draft.setup.as_mut().unwrap().community_property =
        CommunityProperty::Yes { state: CommunityState::California, rule: CommunityRule::Halves };
    assert_eq!(eligible_recipients(&draft, RecipientDomain::Earner), vec![-1, -2]);
    assert_eq!(
        match_eligible_recipient(&draft, "Jordan O'Neil-Smith", RecipientDomain::Earner),
        None
    );
    assert_eq!(
        eligible_recipients(&draft, RecipientDomain::Student),
        vec![-1, -2, draft.dependents[0].id]
    );
}

#[test]
fn joint_names_do_not_discard_unknown_people_or_infer_both_missing_surnames() {
    let draft = document_recipient_household(FederalStatus::Joint);
    for printed in [
        "Alex Morgan ONeilSmith & Unknown Person",
        "Alex Morgan ONeilSmith & Alex Morgan ONeilSmith",
        "Alex & Jordan",
        "Alex ONeilSmith & Jordan ONeilSmith & Casey ONeilSmith",
    ] {
        assert_eq!(
            match_eligible_recipient(&draft, printed, RecipientDomain::Account),
            None,
            "{printed}"
        );
    }
    for domain in [RecipientDomain::Earner, RecipientDomain::Student] {
        assert_eq!(
            match_eligible_recipient(&draft, "Alex ONeilSmith & Jordan ONeilSmith", domain),
            None
        );
    }
}

#[test]
fn unknown_middle_name_accepts_only_an_initial_and_initial_first_names_never_match() {
    let mut draft = document_recipient_household(FederalStatus::Single);
    draft.people.as_mut().unwrap().taxpayer.first_name = "Alex".into();
    assert_eq!(
        match_eligible_recipient(&draft, "Alex M. ONeilSmith", RecipientDomain::Earner),
        Some(-1)
    );
    assert_eq!(
        match_eligible_recipient(&draft, "Alex Morgan ONeilSmith", RecipientDomain::Earner),
        None
    );
    draft.people.as_mut().unwrap().taxpayer.first_name = "A".into();
    assert_eq!(match_eligible_recipient(&draft, "A ONeilSmith", RecipientDomain::Earner), None);
}
