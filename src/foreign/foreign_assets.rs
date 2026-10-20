//! Form 8938 filing-requirement check.

// Coverage: Form 8938 — Partial (T3-small, 2026-09-27): requirement check only.
//   The App does not prepare Form 8938. From the Schedule B Part III answers (a foreign
//   financial account, line 7a, or a foreign trust, line 8) and the one question they add, the
//   highest total value of the filer's foreign financial assets during 2026
//   (ForeignAccountOrTrust.highest_value, stored as FederalScreening.foreign_asset_max; client
//   rule FOREIGN_ASSETS_VALUE_NEEDED checked again on save), the review says when Form 8938 is
//   likely required. Thresholds (i8938 Rev. 11-2021, "Reporting Thresholds Applying to
//   Specified Individuals"): living in the United States, more than $50,000 on the last day of
//   the year or $75,000 at any time ($100,000 / $150,000 filing jointly); living abroad (the
//   presence abroad test: a Form 2555 exclusion claimed on this return, by either spouse on a
//   joint return, Reg. 1.6038D-2(a)(4)), $200,000 / $300,000 ($400,000 / $600,000 jointly).
//   Above the any-time threshold: likely required. Above the last-day threshold only: required
//   if the December 31 value was above it (the return does not carry the year-end value).
//   Not checked: specified foreign financial assets reported without a foreign account or trust
//   answer (foreign stock held directly, foreign partnership interests; K-1 international items
//   are refused separately), exceptions (duplicative Forms 3520/5471/8621 and others), and
//   specified domestic entities. Tests: tests/investment/investment.rs
//   form_8938_check_uses_the_united_states_thresholds, form_8938_check_uses_the_abroad_thresholds,
//   form_8938_note_needs_a_foreign_account_or_trust; tests/interview.rs
//   foreign_account_kind_asks_the_highest_value.

use crate::{FederalDraft, FederalResult, Message, MessageArg, Owner};

// The presence abroad test, as the return shows it: a claimed foreign
// earned income exclusion (Form 2555) for this filer.
fn abroad(d: &FederalDraft, owner: Owner) -> bool {
    d.screening.as_ref().is_some_and(|s| {
        s.foreign_earned.iter().any(|claim| claim.owner == owner && claim.exclusion)
    })
}

// The two Form 8938 thresholds in dollars: the last day of the year and any
// time during it.
pub(crate) fn thresholds(joint: bool, living_abroad: bool) -> (i64, i64) {
    let (last_day, any_time) = if living_abroad { (200_000, 300_000) } else { (50_000, 75_000) };
    if joint { (last_day * 2, any_time * 2) } else { (last_day, any_time) }
}

// The review note on Form 8938, if any.
pub(crate) fn check(d: &FederalDraft, joint: bool, r: &mut FederalResult) {
    let Some(screen) = d.screening.as_ref() else {
        return;
    };
    if !screen.foreign_account && !screen.foreign_trust {
        return;
    }
    let living_abroad = abroad(d, Owner::Taxpayer) || (joint && abroad(d, Owner::Spouse));
    let (last_day, any_time) = thresholds(joint, living_abroad);
    #[cfg(test)]
    let place = match (living_abroad, joint) {
        (false, false) => "living in the United States",
        (false, true) => "living in the United States and filing jointly",
        (true, false) => "living abroad",
        (true, true) => "living abroad and filing jointly",
    };
    let (unknown_key, high_key, conditional_key) = match (living_abroad, joint) {
        (false, false) => ("NOTE_8938_US_UNKNOWN", "NOTE_8938_US_HIGH", "NOTE_8938_US_CONDITIONAL"),
        (false, true) => (
            "NOTE_8938_US_JOINT_UNKNOWN",
            "NOTE_8938_US_JOINT_HIGH",
            "NOTE_8938_US_JOINT_CONDITIONAL",
        ),
        (true, false) => {
            ("NOTE_8938_ABROAD_UNKNOWN", "NOTE_8938_ABROAD_HIGH", "NOTE_8938_ABROAD_CONDITIONAL")
        }
        (true, true) => (
            "NOTE_8938_ABROAD_JOINT_UNKNOWN",
            "NOTE_8938_ABROAD_JOINT_HIGH",
            "NOTE_8938_ABROAD_JOINT_CONDITIONAL",
        ),
    };
    let note = match screen.foreign_asset_max {
        None => crate::federal_model::ReviewNote::localized(
            #[cfg(test)]
            alloc::format!(
                "Form 8938 may be required: compare the total value of your foreign financial assets with its thresholds for filers {place} (more than ${last_day} on December 31, 2026 or ${any_time} at any time during 2026). This App does not prepare Form 8938."
            ),
            Message::new(
                unknown_key,
                alloc::vec![MessageArg::usd_dollars(last_day), MessageArg::usd_dollars(any_time)],
            ),
        ),
        Some(cents) if cents > any_time * 100 => crate::federal_model::ReviewNote::localized(
            #[cfg(test)]
            alloc::format!(
                "Form 8938 is likely required: your foreign financial assets reached ${}, more than the ${any_time} any-time threshold for filers {place}. This App does not prepare Form 8938; file it with your return.",
                cents / 100
            ),
            Message::new(
                high_key,
                alloc::vec![MessageArg::usd_cents(cents), MessageArg::usd_dollars(any_time)],
            ),
        ),
        Some(cents) if cents > last_day * 100 => crate::federal_model::ReviewNote::localized(
            #[cfg(test)]
            alloc::format!(
                "Form 8938 is required if your foreign financial assets were worth more than ${last_day} on December 31, 2026 (the threshold for filers {place}); their highest value, ${}, is not above the ${any_time} any-time threshold. This App does not prepare Form 8938.",
                cents / 100
            ),
            Message::new(
                conditional_key,
                alloc::vec![
                    MessageArg::usd_dollars(last_day),
                    MessageArg::usd_cents(cents),
                    MessageArg::usd_dollars(any_time)
                ],
            ),
        ),
        Some(_) => return,
    };
    r.review_notes.push(note);
}
