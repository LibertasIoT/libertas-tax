use crate::tests::support::client::{read_native_draft, write_native_draft};
// Normal client acceptance bridge. Every incoming payload goes through the
// ordinary native handler; private artifacts retain only the accepted draft.
extern crate std;
use super::unit47_children::{child, page};
use super::unit47_coverdell::{record, topic};
use super::*;
use crate::tests::cross::interview::{open_other_taxes, other_taxes_request, save_you};

fn fixture(case: &str) -> (Interview, P, P) {
    if case.starts_with("qa-") {
        return crate::tests::cross::interview::unit47_documents::client_fixture(case);
    }
    if case == "country-format" {
        return crate::tests::cross::interview::unit47_text_size::country_fixture();
    }
    if case == "children" {
        let mut s = super::unit47_children::session(true);
        let request = page(&mut s, vec![child("Existing", 61)]);
        assert!(!matches!(send(&mut s, request), Reply::Response(P::Problem { .. })));
        let seed = page(&mut s, vec![]);
        let mut guide = seed.clone();
        let P::SaveChildren { dependents, .. } = &mut guide else { unreachable!() };
        dependents.people = vec![child("First", 61), child("Second", 120)];
        return (s, seed, guide);
    }
    assert!(matches!(case, "coverdell-cumulative" | "coverdell-loss"));
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut person = crate::interview::filer(&s.draft.people.as_ref().unwrap().taxpayer);
    person.birth_date = 20100301;
    let request = save_you(&s.draft, person);
    assert!(!matches!(send(&mut s, request), Reply::Response(P::Problem { .. })));
    let seed = open_other_taxes(&mut s);
    let mut facts = record();
    facts.corrections = [(100_000, 20261215), (45_000, 20270501)]
        .into_iter()
        .map(|(principal, withdrawn)| CoverdellCorrectionPage {
            account: "Actual ESA".into(),
            principal,
            withdrawn,
            earnings: if case == "coverdell-loss" && withdrawn == 20261215 { -100_000 } else { 0 },
            income_document: None,
        })
        .collect();
    let guide = other_taxes_request(
        &seed,
        &s.draft,
        vec![OtherTax::AlternativeMinimumTax, topic(vec![facts])],
    );
    (s, seed, guide)
}

fn reopened(s: &mut Interview, case: &str) -> P {
    if case.starts_with("qa-") {
        return crate::tests::cross::interview::unit47_documents::client_reopened(s);
    }
    if case == "country-format" {
        return crate::tests::cross::interview::unit47_text_size::country_reopened(s);
    }
    if case == "children" {
        let people = crate::tests::cross::interview::reopened_people(s);
        page(s, people)
    } else {
        open_other_taxes(s)
    }
}

fn clear_outputs(request: &mut P) {
    match request {
        P::SaveChildren { estimate, progress, .. }
        | P::SaveOtherTaxes { estimate, progress, .. }
        | P::SaveMoreCredits { estimate, progress, .. }
        | P::SaveIncomes { estimate, progress, .. } => {
            *estimate = None;
            *progress = None;
        }
        _ => unreachable!(),
    }
}

// Keep disjoint native controls and file-driven client paths in separate
// frames: neither needs the other path's large Request/Draft temporaries.
fn native_boundary_lifecycle(case: &str) {
    let (mut s, _, mut guide) = fixture(case);
    // Empty derived displays are valid submitted values; the native
    // candidate still owns the refusal key and locator.
    clear_outputs(&mut guide);
    let mut invalid = guide.clone();
    let (key, field) = if case == "children" {
        let P::SaveChildren { dependents, .. } = &mut invalid else { unreachable!() };
        dependents.people[1].lived = LivedWithYou::Days { days: 121 };
        ("RESIDENCE_DAYS_ABOVE_DAYS_ALIVE", "[dependents/people/1/lived/days]")
    } else {
        let P::SaveOtherTaxes { value, .. } = &mut invalid else { unreachable!() };
        let OtherTax::CoverdellExcess { beneficiaries } = &mut value.taxes[1] else {
            unreachable!()
        };
        if case == "coverdell-loss" {
            beneficiaries[0].corrections[0].earnings = -100_001;
            (
                "COVERDELL_LOSS_ABOVE_PRINCIPAL",
                "[value/taxes/1/beneficiaries/0/corrections/0/earnings]",
            )
        } else {
            beneficiaries[0].corrections[1].principal = 45_001;
            (
                "COVERDELL_RETURN_ABOVE_CONTRIBUTIONS",
                "[value/taxes/1/beneficiaries/0/corrections/1/principal]",
            )
        }
    };
    let before = s.draft.clone();
    let dirty = s.dirty;
    let Reply::Response(P::Problem { message, field: actual }) = send(&mut s, invalid) else {
        panic!("native refusal")
    };
    assert_eq!(message, libertas::libertas_formatted_text(key, &[]));
    assert_eq!(actual.as_deref(), Some(field));
    assert_eq!(s.draft, before);
    assert_eq!(s.dirty, dirty);
    assert!(!matches!(send(&mut s, guide), Reply::Response(P::Problem { .. })));
    let stored = s.draft.clone();
    let request = reopened(&mut s, case);
    assert!(!matches!(send(&mut s, request), Reply::Response(P::Problem { .. })));
    assert_eq!(s.draft, stored);
    assert_eq!(restart(stored.clone()).draft, stored);
}
fn client_submit_bridge(case: &str) {
    let state = std::env::var("TAX_UNIT47_CLIENT_STATE").unwrap();
    if let Ok(input) = std::env::var("TAX_UNIT47_CLIENT_INPUT") {
        let draft = read_native_draft(&state);
        let mut s = restart(draft);
        let before = s.draft.clone();
        let dirty = s.dirty;
        let request = P::from_avro(&std::fs::read(input).unwrap()).unwrap();
        let save = matches!(
            request,
            P::SaveOtherTaxes { .. }
                | P::SaveChildren { .. }
                | P::SaveMoreCredits { .. }
                | P::SaveIncomes { .. }
        );
        let reply = send(&mut s, request);
        if let Ok(key) = std::env::var("TAX_UNIT47_CLIENT_EXPECT_KEY") {
            let Reply::Response(P::Problem { message, field }) = &reply else {
                panic!("expected {key}: {reply:?}")
            };
            assert_eq!(*message, libertas::libertas_formatted_text(&key, &[]));
            assert_eq!(
                field.as_deref(),
                Some(std::env::var("TAX_UNIT47_CLIENT_EXPECT_FIELD").unwrap().as_str())
            );
            assert_eq!(s.draft, before);
            assert_eq!(s.dirty, dirty);
        } else if save {
            assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
            if case == "children" {
                assert_eq!(s.draft.dependents.len(), 2);
                assert_eq!(s.draft.dependents[0].residency, DependentResidency::Days { days: 61 });
                assert_eq!(s.draft.dependents[1].residency, DependentResidency::Days { days: 120 });
                assert!(s.draft.dependents.iter().all(|p| p.id > 0));
            } else if case.starts_with("qa-") {
                crate::tests::cross::interview::unit47_documents::assert_client_sources(&s, case);
            } else if case == "country-format" {
                crate::tests::cross::interview::unit47_text_size::assert_country_source(&s);
            } else {
                assert_eq!(s.draft.coverdell_excess.len(), 1);
                let P::SaveOtherTaxes { value: expected, .. } = fixture(case).2 else {
                    unreachable!()
                };
                let P::SaveOtherTaxes { value: actual, .. } = open_other_taxes(&mut s) else {
                    unreachable!()
                };
                // OtherTaxes reopens in the existing canonical topic order,
                // independent of the submitted order used to prove row1 paths.
                assert_eq!(expected.taxes[0], OtherTax::AlternativeMinimumTax);
                assert_eq!(
                    actual.taxes,
                    vec![expected.taxes[1].clone(), OtherTax::AlternativeMinimumTax]
                );
            }
            let accepted = s.draft.clone();
            let shown = reopened(&mut s, case);
            std::fs::write(std::env::var("TAX_UNIT47_CLIENT_REOPEN").unwrap(), shown.to_avro())
                .unwrap();
            assert!(!matches!(send(&mut s, shown), Reply::Response(P::Problem { .. })));
            assert_eq!(s.draft, accepted);
            assert_eq!(restart(accepted.clone()).draft, accepted);
        } else {
            // Navigation and normal calculations never mutate accepted answers.
            assert_eq!(s.draft, before);
            assert_eq!(s.dirty, dirty);
        }
        let (kind, response) = match reply {
            Reply::Edit(p) => ("edit", p),
            Reply::Response(p) => ("response", p),
        };
        std::fs::write(std::env::var("TAX_UNIT47_CLIENT_OUTPUT").unwrap(), response.to_avro())
            .unwrap();
        std::fs::write(std::env::var("TAX_UNIT47_CLIENT_KIND").unwrap(), kind).unwrap();
        write_native_draft(state, &s.draft);
    } else {
        let (s, seed, guide) = fixture(case);
        std::fs::write(std::env::var("TAX_UNIT47_CLIENT_SEED").unwrap(), seed.to_avro()).unwrap();
        std::fs::write(std::env::var("TAX_UNIT47_CLIENT_GUIDE").unwrap(), guide.to_avro()).unwrap();
        write_native_draft(state, &s.draft);
    }
}
#[test]
fn unit47_normal_client_submit_bridge_and_boundary_lifecycle() {
    if let Ok(case) = std::env::var("TAX_UNIT47_CLIENT_CASE") {
        client_submit_bridge(&case);
    } else {
        for case in ["coverdell-cumulative", "coverdell-loss", "children"] {
            native_boundary_lifecycle(case);
        }
    }
}
