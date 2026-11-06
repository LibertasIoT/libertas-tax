// The two native source statements share these scenarios, while each caller
// supplies its actual fields, identities, dates, K-1 routing and page choices.
// Expand in the topic module to reuse Form 3800's existing own/history fixtures
// and retain the original test names and external native-client codec hooks.
macro_rules! origin_credit_scenarios {
    (
        credit: $credit:expr, code: $code:ident, native_line: $native_line:expr,
        editor_line: $editor_line:expr, native_profile: $native_profile:literal,
        origin: $origin_type:ident, component: $component_type:ident,
        components: $components:ident, field: $origin_field:ident,
        other_field: $other_origin_field:ident, allowed: $allowed_field:ident,
        dates: ($pre_date:literal, $post_date:literal),
        identities: ($first_identity:literal, $second_identity:literal),
        projected: ($post_identity:literal, $pre_identity:literal, $suspended_identity:literal),
        env: $env:literal, line_prefix: $line_prefix:literal, profile_error: $profile_error:literal,
        issues: ($mixed_issue:literal, $date_warning:literal, $invalid_issue:literal),
        tests: ($test0:ident, $test1:ident, $test2:ident, $test3:ident,
                $test4:ident, $test5:ident, $test6:ident),
        choices: $choices:block $(,)?
    ) => {
        use super::*;
        extern crate std;

        fn origin(parts: &[(&str, Option<u32>, i64)]) -> $origin_type {
            $origin_type::Statement {
                $components: parts
                    .iter()
                    .map(|(identity, date, amount)| $component_type {
                        identity: (*identity).into(),
                        original_placed_on: *date,
                        amount: *amount,
                    })
                    .collect(),
            }
        }
        fn origin_draft(date: Option<u32>, year: u32) -> FederalDraft {
            let mut d = complete(FederalStatus::Joint);
            d.income[0] = wage(1, Owner::Taxpayer, 5_716_000, 0);
            own(&mut d, $credit, 6000);
            history(&mut d).credits[0].$origin_field =
                Some(origin(&[($first_identity, date, 600_000)]));
            history(&mut d).carrybacks.push(BusinessCreditCarryback {
                credit: $credit,
                year,
                amount: 300_000,
                research_source: None,
            });
            d
        }
        fn assert_origin(d: &FederalDraft, year: u32, included: bool) {
            let r = supported(d);
            assert_eq!(value(&r, "taxable_income"), 24960);
            assert_eq!(value(&r, "regular_income_tax"), 2501);
            assert_eq!(value(&r, "general_business_credit"), 2501);
            assert_eq!(
                value(&r, concat!($line_prefix, "_current_year_to_next_year")),
                if included { 499 } else { 3499 }
            );
            if included {
                assert_eq!(
                    value(
                        &r,
                        &alloc::format!(
                            concat!($line_prefix, "_current_year_carryback_{year}"),
                            year = year
                        )
                    ),
                    3000
                );
            } else {
                assert!(!r.lines.iter().any(|l| l.id
                    == alloc::format!(
                        concat!($line_prefix, "_current_year_carryback_{year}"),
                        year = year
                    )));
            }
            assert_eq!(noted(&r, $date_warning), !included);
            assert_eq!(supported(&restart(d.clone()).draft), r);
        }
        // The completed native credit owner, after PAL and current use, retains the
        // unresolved older exception without guessing a component use ordering.
        fn assert_mixed_unused(d: &FederalDraft) {
            let r = estimate(d);
            assert!(r.issues.iter().any(|issue| issue.display.key == $mixed_issue));
            // Unsupported results deliberately withhold financial rows. Prove the
            // actual current-use/unused pool with identical source facts and no older
            // request, then check that the unresolved request emits no invented result.
            let mut baseline = d.clone();
            history(&mut baseline).carrybacks.clear();
            let native = supported(&baseline);
            assert_eq!(value(&native, "general_business_credit"), 2501);
            assert_eq!(value(&native, concat!($line_prefix, "_current_year_to_next_year")), 3499);
            assert!(r.lines.is_empty());
            assert!(
                !r.lines
                    .iter()
                    .any(|line| line.id == concat!($line_prefix, "_current_year_carryback_2023"))
            );
            assert_eq!(estimate(&restart(d.clone()).draft), r);
        }
        #[test]
        fn $test0() {
            let pre = origin_draft(Some($pre_date), 2023);
            assert_eq!(
                pre.credits.as_ref().unwrap().business_credits.as_ref().unwrap().source_rule(&pre),
                Some("BUSINESS_CARRYBACK_YEAR_NOT_ALLOWED")
            );
            assert!(
                estimate(&pre)
                    .issues
                    .iter()
                    .any(|issue| issue.display.key == "BUSINESS_CARRYBACK_YEAR_NOT_ALLOWED")
            );
            assert_origin(&origin_draft(Some($post_date), 2023), 2023, true);
            for date in [None, Some($pre_date), Some($post_date)] {
                assert_origin(&origin_draft(date, 2025), 2025, true);
            }
            let unknown_date = origin_draft(None, 2023);
            assert_origin(&unknown_date, 2023, false);
            let mut unknown_origin = unknown_date.clone();
            history(&mut unknown_origin).credits[0].$origin_field = None;
            assert_origin(&unknown_origin, 2023, false);
            let mut mixed = origin_draft(None, 2023);
            history(&mut mixed).credits[0].$origin_field = Some(origin(&[
                ($first_identity, Some($pre_date), 300_000),
                ($second_identity, Some($post_date), 300_000),
            ]));
            assert_eq!(history(&mut mixed).clone().source_rule(&mixed), None);
            assert_mixed_unused(&mixed);
            history(&mut mixed).carrybacks[0].year = 2025;
            assert_origin(&mixed, 2025, true);
        }
        #[test]
        fn $test1() {
            let valid = origin(&[
                ($first_identity, Some($post_date), 300_001),
                ($second_identity, None, 299_999),
            ]);
            assert!(valid.valid(600_000));
            assert!(!origin(&[]).valid(600_000));
            assert!(!valid.valid(599_999));
            assert!(
                !origin(&[("same", Some($post_date), 300_000), ("same", Some($pre_date), 300_000)])
                    .valid(600_000)
            );
            // Preserve complete source identifiers.
            let exact_bytes = "é".repeat(100);
            assert_eq!(exact_bytes.len(), 200);
            assert!(origin(&[(&exact_bytes, Some($post_date), 600_000)]).valid(600_000));
            let over_bytes = alloc::format!("{exact_bytes}A");
            assert_eq!(over_bytes.len(), 201);
            assert!(origin(&[(&over_bytes, Some($post_date), 600_000)]).valid(600_000));
            let mut k1 = K1Credit {
                code: K1CreditCode::$code,
                line: $native_line,
                amount: 600_000,
                $other_origin_field: None,
                $origin_field: Some(valid),
            };
            assert!(K1Credit::valid_list(&[k1.clone()]));
            k1.code = K1CreditCode::DisabledAccess;
            assert!(!K1Credit::valid_list(&[k1]));
            $choices
        }
        #[test]
        fn $test2() {
            use crate::community::Divide;
            let mut d = complete(FederalStatus::Separate);
            d.setup.as_mut().unwrap().community_property = CommunityProperty::Yes {
                state: CommunityState::Arizona,
                rule: CommunityRule::Halves,
            };
            let income = FederalIncome::Partnership { data: ordinary_partner_source(0, true) };
            let mut source = K1Credit {
                code: K1CreditCode::$code,
                line: $native_line,
                amount: 100,
                $other_origin_field: None,
                $origin_field: Some(origin(&[("A", Some($post_date), 49), ("B", None, 51)])),
            };
            let shares = [Owner::Taxpayer, Owner::Spouse].map(|owner| {
                crate::community::share(
                    d.setup.as_ref().unwrap(),
                    owner,
                    CommunityShare::Community,
                    &income,
                )
                .unwrap()
            });
            let before = source.clone();
            let divided = shares.map(|share| source.divided(share));
            assert_eq!(source, before);
            assert_eq!(divided.each_ref().map(|s| s.amount), [50, 50]);
            let parts = divided.each_ref().map(|s| {
                assert!(K1Credit::valid_list(core::slice::from_ref(s)));
                let $origin_type::Statement { $components } = s.$origin_field.as_ref().unwrap();
                $components
                    .iter()
                    .map(|p| (p.identity.clone(), p.amount, p.original_placed_on))
                    .collect::<Vec<_>>()
            });
            assert_eq!(parts[0].iter().map(|p| p.1).collect::<Vec<_>>(), vec![25, 25]);
            assert_eq!(parts[1].iter().map(|p| p.1).collect::<Vec<_>>(), vec![24, 26]);
            for ((filer, counterpart), original) in parts[0].iter().zip(&parts[1]).zip([49, 51]) {
                assert_eq!(filer.0, counterpart.0);
                assert_eq!(filer.1 + counterpart.1, original);
                assert_eq!(filer.2, counterpart.2);
            }
            let $origin_type::Statement { $components } = source.$origin_field.as_mut().unwrap();
            $components.reverse();
            for (i, share) in shares.into_iter().enumerate() {
                let reversed = source.divided(share);
                let $origin_type::Statement { $components } = reversed.$origin_field.unwrap();
                for part in $components {
                    assert_eq!(
                        parts[i].iter().find(|p| p.0 == part.identity).unwrap().1,
                        part.amount
                    );
                }
            }
        }

        #[test]
        fn $test3() {
            use crate::tests::cross::interview::{accepted, navigate, rejected_with};
            use libertas::{AvroDecode, AvroEncode};
            let selected = std::env::var(concat!($env, "_PROFILE")).ok();
            let profiles = selected
                .as_deref()
                .map_or_else(|| vec!["post", "pre", "unknown", "missing", "mixed"], |p| vec![p]);
            for profile in profiles {
                let (date, year) = match profile {
                    "post" => (Some($post_date), 2023),
                    "pre" => (Some($pre_date), 2025),
                    "unknown" | "missing" => (None, 2023),
                    "mixed" => (None, 2025),
                    _ => panic!($profile_error),
                };
                let mut d = origin_draft(date, year);
                let mut expected = history(&mut d).credits[0].clone();
                if profile == "missing" {
                    expected.$origin_field = None;
                } else if profile == "mixed" {
                    expected.$origin_field = Some(origin(&[
                        ($first_identity, Some($pre_date), 300_000),
                        ($second_identity, Some($post_date), 300_000),
                    ]));
                }
                history(&mut d).credits.clear();
                history(&mut d).carrybacks.clear();
                let mut session = Interview::new(TEST_USER, d);
                let decode = |path: &str| {
                    let bytes = std::fs::read(path).unwrap();
                    let mut offset = 0;
                    let p = P::avro_decode(&bytes, &mut offset).unwrap();
                    assert_eq!(offset, bytes.len());
                    p
                };
                let write = |name: &str, p: &P| {
                    if let Ok(path) = std::env::var(name) {
                        let mut bytes = vec![];
                        p.avro_encode(&mut bytes);
                        std::fs::write(path, bytes).unwrap();
                    }
                };
                if let Ok(path) = std::env::var(concat!($env, "_CALC_INPUT")) {
                    let Reply::Response(reply) = send(&mut session, decode(&path)) else {
                        panic!()
                    };
                    write(concat!($env, "_CALC_OUTPUT"), &reply);
                    return;
                }
                let open = |session: &mut Interview| {
                    let d = session.draft.clone();
                    match send(session, navigate(&d, TaxSection::MoreCredits)) {
                        Reply::Edit(p @ P::SaveMoreCredits { .. }) => p,
                        other => panic!("expected More credits: {other:?}"),
                    }
                };
                let seed = open(&mut session);
                write(concat!($env, "_SEED"), &seed);
                let mut answers = seed;
                let P::SaveMoreCredits { value, .. } = &mut answers else { panic!() };
                let MoreCredit::BusinessCredits(h) = value
                    .credits
                    .iter_mut()
                    .find(|v| matches!(v, MoreCredit::BusinessCredits(_)))
                    .unwrap()
                else {
                    panic!()
                };
                h.credits.push(expected.clone().into());
                h.carrybacks.push(
                    BusinessCreditCarryback {
                        credit: $credit,
                        year,
                        amount: 300_000,
                        research_source: None,
                    }
                    .into(),
                );
                write(concat!($env, "_ANSWERS"), &answers);
                if std::env::var(concat!($env, "_SEED_ONLY")).as_deref() == Ok("1") {
                    return;
                }
                let payload = std::env::var(concat!($env, "_PAYLOAD")).ok();
                accepted(&send(&mut session, payload.as_deref().map_or(answers, decode)));
                assert_eq!(history(&mut session.draft).credits, vec![expected.clone()]);
                assert_origin(&session.draft, year, !matches!(profile, "unknown" | "missing"));
                let saved = session.draft.clone();
                session = restart(saved.clone());
                assert_eq!(session.draft, saved);
                let reopened = open(&mut session);
                write(concat!($env, "_REOPENED"), &reopened);
                let P::SaveMoreCredits { value, .. } = &reopened else { panic!() };
                let MoreCredit::BusinessCredits(h) = value
                    .credits
                    .iter()
                    .find(|v| matches!(v, MoreCredit::BusinessCredits(_)))
                    .unwrap()
                else {
                    panic!()
                };
                assert_eq!(h.credits[0].$allowed_field, vec![0]);
                assert_eq!(h.credits[0].$origin_field, expected.$origin_field);
                accepted(&send(&mut session, reopened));
                assert_eq!(session.draft, saved);
                // A forged source statement cannot alter the accepted native draft.
                for bad in [
                    origin(&[]),
                    origin(&[("A", Some($post_date), 599_999)]),
                    origin(&[("A", None, 300_000), ("A", None, 300_000)]),
                ] {
                    let mut invalid = open(&mut session);
                    let P::SaveMoreCredits { value, .. } = &mut invalid else { panic!() };
                    let MoreCredit::BusinessCredits(h) = value
                        .credits
                        .iter_mut()
                        .find(|v| matches!(v, MoreCredit::BusinessCredits(_)))
                        .unwrap()
                    else {
                        panic!()
                    };
                    h.credits[0].$origin_field = Some(bad);
                    rejected_with(send(&mut session, invalid), $invalid_issue);
                    assert_eq!(session.draft, saved);
                }
                // A structurally complete mixed statement may Save. Its unresolved
                // older exception belongs to the final credit owner, not Save validation.
                if profile == "mixed" {
                    for changed_year in [2023, 2025] {
                        let mut correction = open(&mut session);
                        let P::SaveMoreCredits { value, .. } = &mut correction else { panic!() };
                        let MoreCredit::BusinessCredits(h) = value
                            .credits
                            .iter_mut()
                            .find(|v| matches!(v, MoreCredit::BusinessCredits(_)))
                            .unwrap()
                        else {
                            panic!()
                        };
                        h.carrybacks[0].year = changed_year;
                        accepted(&send(&mut session, correction));
                        assert_eq!(history(&mut session.draft).credits, vec![expected.clone()]);
                        assert_eq!(
                            history(&mut session.draft).clone().source_rule(&session.draft),
                            None
                        );
                        if changed_year == 2023 {
                            assert_mixed_unused(&session.draft);
                        } else {
                            assert_origin(&session.draft, 2025, true);
                        }
                        session = restart(session.draft.clone());
                        let retained = open(&mut session);
                        accepted(&send(&mut session, retained));
                    }
                }
                // Only an actual date correction resolves the extended carryback.
                if matches!(profile, "post" | "unknown" | "missing") {
                    for (date, included) in [(None, false), (Some($post_date), true)] {
                        let mut correction = open(&mut session);
                        let P::SaveMoreCredits { value, .. } = &mut correction else { panic!() };
                        let MoreCredit::BusinessCredits(h) = value
                            .credits
                            .iter_mut()
                            .find(|v| matches!(v, MoreCredit::BusinessCredits(_)))
                            .unwrap()
                        else {
                            panic!()
                        };
                        h.credits[0].$origin_field =
                            Some(origin(&[($first_identity, date, 600_000)]));
                        accepted(&send(&mut session, correction));
                        assert_origin(&session.draft, year, included);
                        session = restart(session.draft.clone());
                    }
                }
            }
        }

        #[test]
        fn $test4() {
            use crate::tests::cross::interview::{accepted, navigate};
            use libertas::{AvroDecode, AvroEncode};
            let selected = std::env::var(concat!($env, "_K1_PROFILE")).ok();
            let profiles =
                selected.as_deref().map_or_else(|| vec![$native_profile, "other"], |p| vec![p]);
            for profile in profiles {
                let mut d = complete(FederalStatus::Joint);
                d.income[0] = wage(1, Owner::Taxpayer, 5_716_000, 0);
                add(
                    &mut d,
                    income(FederalIncome::Partnership { data: ordinary_partner_source(0, true) }),
                );
                history(&mut d);
                let id = d.income.last().unwrap().id;
                name_accepted_income_documents(&mut d);
                let d = crate::tests::support::builders::known_fact_partnership_draft(d);
                let mut session = Interview::new(TEST_USER, d);
                let decode = |path: &str| {
                    let bytes = std::fs::read(path).unwrap();
                    let mut offset = 0;
                    let p = P::avro_decode(&bytes, &mut offset).unwrap();
                    assert_eq!(offset, bytes.len());
                    p
                };
                let write = |name: &str, p: &P| {
                    if let Ok(dir) = std::env::var(concat!($env, "_K1_DIR")) {
                        let mut bytes = vec![];
                        p.avro_encode(&mut bytes);
                        std::fs::write(std::path::Path::new(&dir).join(name), bytes).unwrap();
                    }
                };
                if let Ok(path) = std::env::var(concat!($env, "_K1_CALC_INPUT")) {
                    let Reply::Response(reply) = send(&mut session, decode(&path)) else {
                        panic!()
                    };
                    if let Ok(out) = std::env::var(concat!($env, "_K1_CALC_OUTPUT")) {
                        let mut bytes = vec![];
                        reply.avro_encode(&mut bytes);
                        std::fs::write(out, bytes).unwrap();
                    }
                    return;
                }
                let seed =
                    crate::tests::support::client::income_pages::open_document_center(&mut session);
                write("seed.avro", &seed);
                let mut answers = seed;
                let P::SaveDocuments { documents, .. } = &mut answers else { panic!() };
                let crate::K1Document::K1Partnership(page) = &mut documents
                    .common
                    .k1
                    .documents
                    .iter_mut()
                    .find(|e| e.id == id)
                    .unwrap()
                    .document
                else {
                    panic!()
                };
                let expected = K1Credit {
                    code: if profile == $native_profile {
                        K1CreditCode::$code
                    } else {
                        K1CreditCode::Other
                    },
                    line: ($editor_line)(profile),
                    amount: 600_000,
                    $other_origin_field: None,
                    $origin_field: Some(origin(&[($first_identity, Some($post_date), 600_000)])),
                };
                crate::tests::support::builders::push_partnership_credit(page, expected.clone());
                write("answers.avro", &answers);
                if std::env::var(concat!($env, "_K1_SEED_ONLY")).as_deref() == Ok("1") {
                    return;
                }
                let payload = std::env::var(concat!($env, "_K1_PAYLOAD")).ok();
                accepted(
                    &crate::tests::support::client::income_pages::save_source_fixture_request(
                        &mut session,
                        payload.as_deref().map_or(answers, decode),
                    ),
                );
                let FederalIncome::Partnership { data } =
                    &session.draft.income.iter().find(|e| e.id == id).unwrap().income
                else {
                    panic!()
                };
                assert_eq!(data.credits, vec![expected.clone()]);
                assert_eq!(value(&supported(&session.draft), "general_business_credit"), 2501);
                let saved = session.draft.clone();
                session = restart(saved.clone());
                assert_eq!(session.draft, saved);
                let reopened =
                    crate::tests::support::client::income_pages::open_document_center(&mut session);
                write("reopened.avro", &reopened);
                let P::SaveDocuments { documents, .. } = &reopened else { panic!() };
                let crate::K1Document::K1Partnership(page) =
                    &documents.common.k1.documents.iter().find(|e| e.id == id).unwrap().document
                else {
                    panic!()
                };
                assert_eq!(page.calculation_page().more.credits[0].$allowed_field, vec![0]);
                assert_eq!(
                    page.personal.as_ref().unwrap().credit_origins[0].$origin_field,
                    expected.$origin_field
                );
                accepted(&send(&mut session, reopened));
                assert_eq!(session.draft, saved);
                let d = session.draft.clone();
                let Reply::Edit(mut back) =
                    send(&mut session, navigate(&d, TaxSection::MoreCredits))
                else {
                    panic!()
                };
                let P::SaveMoreCredits { value, .. } = &mut back else { panic!() };
                let MoreCredit::BusinessCredits(h) = value
                    .credits
                    .iter_mut()
                    .find(|v| matches!(v, MoreCredit::BusinessCredits(_)))
                    .unwrap()
                else {
                    panic!()
                };
                h.carrybacks.push(
                    BusinessCreditCarryback {
                        credit: $credit,
                        year: 2023,
                        amount: 300_000,
                        research_source: None,
                    }
                    .into(),
                );
                accepted(&send(&mut session, back));
                assert_origin(&session.draft, 2023, true);
                // Correcting only the actual K1 source date changes eligibility and is
                // retained through the same parent Save, not inferred from its name.
                for (date, included) in [(None, false), (Some($post_date), true)] {
                    let mut corrected =
                        crate::tests::support::client::income_pages::open_document_center(
                            &mut session,
                        );
                    let P::SaveDocuments { documents, .. } = &mut corrected else { panic!() };
                    let crate::K1Document::K1Partnership(page) = &mut documents
                        .common
                        .k1
                        .documents
                        .iter_mut()
                        .find(|e| e.id == id)
                        .unwrap()
                        .document
                    else {
                        panic!()
                    };
                    page.personal.as_mut().unwrap().credit_origins[0].$origin_field =
                        Some(origin(&[($first_identity, date, 600_000)]));
                    accepted(
                        &crate::tests::support::client::income_pages::save_source_fixture_request(
                            &mut session,
                            corrected,
                        ),
                    );
                    assert_origin(&session.draft, 2023, included);
                    session = restart(session.draft.clone());
                }
            }
        }

        #[test]
        fn $test5() {
            for (owner, projected_cents, native_dollars) in
                [(Owner::Spouse, 49, 0), (Owner::Taxpayer, 50, 1)]
            {
                let mut d = complete(FederalStatus::Separate);
                d.setup.as_mut().unwrap().community_property = CommunityProperty::Yes {
                    state: CommunityState::Arizona,
                    rule: CommunityRule::Halves,
                };
                own(&mut d, $credit, 6000);
                history(&mut d).credits[0].$origin_field =
                    Some(origin(&[($post_identity, Some($post_date), 600_000)]));
                history(&mut d).carrybacks.push(BusinessCreditCarryback {
                    credit: $credit,
                    year: 2023,
                    amount: 300_000,
                    research_source: None,
                });
                let mut k1 = ordinary_partner_source(0, true);
                k1.credits.push(K1Credit {
                    code: K1CreditCode::$code,
                    line: $native_line,
                    amount: 99,
                    $other_origin_field: None,
                    $origin_field: Some(origin(&[($pre_identity, Some($pre_date), 99)])),
                });
                let mut source = income(FederalIncome::Partnership { data: k1 });
                source.owner = crate::DocumentOwner::from(owner);
                source.community = CommunityShare::Community;
                add(&mut d, source);
                let id = d.income.last().unwrap().id;
                let original = d.income.last().unwrap().clone();
                let projected = crate::community::entries(&d);
                let FederalIncome::Partnership { data } =
                    &projected.iter().find(|e| e.id == id).unwrap().income
                else {
                    panic!()
                };
                assert_eq!(data.credits[0].amount, projected_cents);
                assert_eq!(crate::kernel::ratio::dollars(data.credits[0].amount), native_dollars);
                // The native credit engine filters whole-credit $0 rows before it
                // merges origins. This does not round individual source components.
                assert_eq!(history(&mut d).clone().source_rule(&d), None);
                assert_eq!(d.income.last().unwrap(), &original);
                assert_eq!(
                    estimate(&d).issues.iter().any(|i| i.display.key == $mixed_issue),
                    native_dollars > 0
                );
            }
        }

        #[test]
        fn $test6() {
            let mut d = origin_draft(Some($post_date), 2023);
            let mut k1 = ordinary_partner_source(0, false);
            k1.credits.push(K1Credit {
                code: K1CreditCode::$code,
                line: $native_line,
                amount: 600_000,
                $other_origin_field: None,
                $origin_field: Some(origin(&[($suspended_identity, Some($pre_date), 600_000)])),
            });
            add(&mut d, income(FederalIncome::Partnership { data: k1 }));
            let original = d.clone();
            let mut before = d.clone();
            history(&mut before).carrybacks.clear();
            let baseline = supported(&before);
            assert_eq!(value(&baseline, "general_business_credit"), 2501);
            assert_eq!(
                value(&baseline, concat!($line_prefix, "_current_year_passive_to_next_year")),
                6000
            );
            assert_eq!(value(&baseline, concat!($line_prefix, "_current_year_to_next_year")), 3499);
            // These are genuine final native PAL/current-use amounts. Source Save
            // must not invent an earlier eligibility classifier from source dates.
            assert_eq!(history(&mut d).clone().source_rule(&d), None);
            assert_origin(&d, 2023, true);
            let result = supported(&d);
            assert_eq!(
                value(&result, concat!($line_prefix, "_current_year_passive_to_next_year")),
                6000
            );
            assert_eq!(d, original);
        }
    };
}
pub(in crate::tests) use origin_credit_scenarios;
