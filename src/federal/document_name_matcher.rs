use alloc::{string::String, vec::Vec};

pub(crate) use crate::document_parties::JOINT_RECIPIENT;

#[derive(Clone, Copy)]
pub(crate) enum RecipientDomain {
    Earner,
    Account,
    Student,
}

// Matching and manual assignment admission use the same eligible keys.
pub(crate) fn eligible_recipients(
    draft: &crate::FederalDraft,
    domain: RecipientDomain,
) -> Vec<i64> {
    let spouse = draft.setup.as_ref().is_some_and(|setup| {
        setup.filing_status() == crate::FederalStatus::Joint
            || crate::community::applies(setup)
            || (setup.filing_status() == crate::FederalStatus::Separate
                && matches!(domain, RecipientDomain::Account))
    });
    let mut keys: Vec<_> = crate::federal_rules::people_students(draft)
        .into_iter()
        .map(|person| person.id)
        .filter(|key| match key {
            -1 => true,
            -2 => spouse,
            id => *id > 0 && matches!(domain, RecipientDomain::Student),
        })
        .collect();
    // Separate returns can report joint accounts, and community-property
    // returns can also report a spouse's other sources, without the joint
    // return's full spouse-facts record.
    if spouse && !keys.contains(&-2) {
        keys.insert(1, -2);
    }
    if spouse && keys.contains(&-2) && matches!(domain, RecipientDomain::Account) {
        keys.push(JOINT_RECIPIENT);
    }
    keys
}

fn words(name: &str) -> Vec<String> {
    name.split(|c: char| c.is_whitespace() || c == ',')
        .map(|word| {
            word.chars()
                .filter(|c| c.is_alphanumeric())
                .flat_map(char::to_lowercase)
                .collect::<String>()
        })
        .filter(|word| !word.is_empty())
        .collect()
}

// Require the full first name and every surname part, in either printed order.
// A middle name may be omitted or replaced by its compatible initial.
fn same_name(printed: &str, first: &str, last: &str) -> bool {
    let mut printed = words(printed);
    let mut first = words(first);
    let last = words(last);
    let Some(given) = first.first() else { return false };
    if given.chars().count() <= 1 || last.is_empty() {
        return false;
    }
    let surname = last.concat();
    let Some((start, end)) = (0..printed.len()).find_map(|start| {
        let mut joined = String::new();
        for (offset, word) in printed[start..].iter().enumerate() {
            joined.push_str(word);
            if joined == surname {
                return Some((start, start + offset + 1));
            }
            if !surname.starts_with(&joined) {
                break;
            }
        }
        None
    }) else {
        return false;
    };
    printed.drain(start..end);
    let Some(position) = printed.iter().position(|word| word == given) else { return false };
    printed.remove(position);
    first.remove(0);
    if first.is_empty() {
        // No known middle name: a printed initial adds no conflicting name.
        return printed.len() <= 1 && printed.iter().all(|word| word.chars().count() == 1);
    }
    for word in printed {
        let Some(position) = first.iter().position(|middle| {
            &word == middle
                || (word.chars().count() == 1 || middle.chars().count() == 1)
                    && word.chars().next() == middle.chars().next()
        }) else {
            return false;
        };
        first.remove(position);
    }
    true
}

fn match_person(draft: &crate::FederalDraft, printed: &str, eligible: &[i64]) -> Option<i64> {
    let people = draft.people.as_ref()?;
    let filers = core::iter::once((-1, &people.taxpayer.first_name, &people.taxpayer.last_name))
        .chain(people.spouse.as_ref().map(|spouse| (-2, &spouse.first_name, &spouse.last_name)));
    let dependents =
        draft.dependents.iter().map(|person| (person.id, &person.first_name, &person.last_name));
    let mut matched = filers.chain(dependents).filter_map(|(key, first, last)| {
        (eligible.contains(&key) && same_name(printed, first, last)).then_some(key)
    });
    let key = matched.next()?;
    matched.next().is_none().then_some(key)
}

// The result is an ephemeral catalog key, never an allocation of tax amounts.
pub(crate) fn match_eligible_recipient(
    draft: &crate::FederalDraft,
    printed: &str,
    domain: RecipientDomain,
) -> Option<i64> {
    let eligible = eligible_recipients(draft, domain);
    let lower = printed.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase();
    let split = lower.split_once('&').or_else(|| lower.split_once(" and "));
    let Some((left, right)) = split else {
        return match_person(draft, printed, &eligible);
    };
    if !eligible.contains(&JOINT_RECIPIENT) {
        return None;
    }
    let people = draft.people.as_ref()?;
    let spouse = people.spouse.as_ref()?;
    if match_person(draft, left, &eligible).is_none()
        && match_person(draft, right, &eligible).is_none()
    {
        return None;
    }
    let resolve = |name: &str| {
        match_person(draft, name, &eligible).or_else(|| {
            if words(&people.taxpayer.last_name).concat() != words(&spouse.last_name).concat() {
                return None;
            }
            match_person(draft, &alloc::format!("{name} {}", spouse.last_name), &eligible)
        })
    };
    match (resolve(left), resolve(right)) {
        (Some(-1), Some(-2)) | (Some(-2), Some(-1)) => Some(JOINT_RECIPIENT),
        _ => None,
    }
}
