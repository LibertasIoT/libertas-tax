extern crate std;

use crate::tests::cross::interview::{accepted, navigate};
use crate::tests::support::builders::injured_original::fixture;
use crate::tests::*;
use std::{fs, path::Path};

fn open(directory: &Path, session: &mut Interview) {
    // Supply the real Review reference context for its ordinary OpenSection
    // link; a raw Navigate request alone has no allowed section catalog.
    let entry = P::ReviewAccepted { user: TEST_USER };
    fs::write(directory.join("entry.avro"), entry.to_avro()).unwrap();
    let Reply::Response(review @ P::Review { .. }) = send(session, entry) else {
        panic!("expected the ordinary accepted-return review")
    };
    fs::write(directory.join("review.avro"), review.to_avro()).unwrap();
    let request = navigate(&session.draft, TaxSection::InjuredSpouse);
    fs::write(directory.join("request.avro"), request.to_avro()).unwrap();
    let Reply::Response(begin @ P::BeginInjuredSpouse { .. }) = send(session, request) else {
        panic!("the real opening must construct its required editor")
    };
    fs::write(directory.join("begin.avro"), begin.to_avro()).unwrap();
}

fn result(directory: &Path, draft: &FederalDraft) {
    let r = supported(draft);
    let mut plain = draft.clone();
    plain.injured_spouse = None;
    let joint = supported(&plain);
    assert_eq!(r.return_lines, joint.return_lines);
    assert_eq!(
        (r.tax, r.payments, r.refund, r.balance),
        (joint.tax, joint.payments, joint.refund, joint.balance)
    );
    let lines: std::collections::BTreeMap<_, _> = r
        .lines
        .iter()
        .filter(|l| l.id.starts_with("form_8379_"))
        .map(|l| (l.id.clone(), l.amount))
        .collect();
    let profile = std::env::var("TAX_UNIT13_ORIGINAL_PROFILE").unwrap();
    let (case, _) = profile.rsplit_once('-').unwrap();
    let injured = draft.injured_spouse.as_ref().unwrap().injured;
    let line = |row: &str, column: &str| {
        *lines.get(&alloc::format!("form_8379_line_{row}_{column}")).unwrap()
    };
    let protected = match case {
        "a" | "e" => Some(12500),
        "b" | "d-nv" | "d-nm" | "d-wa" | "f-zero" => Some(50000),
        "b-family" | "c-ca" | "c-id" | "c-la" => Some(0),
        "f" => Some(37500),
        _ => None,
    };
    if let Some(expected) = protected {
        assert_eq!(r.refund, Some(100000));
        assert_eq!(lines.get("form_8379_injured_refund_estimate"), Some(&expected));
    }
    match case {
        "g" => {
            assert_eq!(line("13b", "a"), 1200000);
            assert_eq!(line("13b", "b"), if injured == Owner::Taxpayer { 1200000 } else { 0 });
            let pension = draft
                .income
                .iter()
                .find(|e| matches!(e.income, FederalIncome::Retirement { .. }))
                .unwrap();
            assert_eq!(pension.community, CommunityShare::Partly { percent: 50 });
            let community =
                crate::community::refund_community_percent(&pension.income, pension.community, 5)
                    .unwrap();
            let FederalIncome::Retirement { data } = &pension.income else { unreachable!() };
            assert_eq!(
                crate::kernel::ratio::fraction(data.gross, i64::from(community), 100),
                600000
            );
            // P555's hypothetical separate reporting explains source character;
            // it is never used to replace the joint recipient's Part III income.
            let mut separate = complete(FederalStatus::Separate).setup.unwrap();
            separate.community_property = CommunityProperty::Yes {
                state: CommunityState::Nevada,
                rule: CommunityRule::Halves,
            };
            assert_eq!(
                crate::community::share(
                    &separate,
                    Owner::Taxpayer,
                    pension.community,
                    &pension.income
                )
                .unwrap()
                .filer(data.gross),
                900000
            );
            assert_eq!(
                crate::community::share(
                    &separate,
                    Owner::Spouse,
                    pension.community,
                    &pension.income
                )
                .unwrap()
                .filer(data.gross),
                300000
            );
        }
        "h" if draft
            .injured_spouse
            .as_ref()
            .unwrap()
            .allocations
            .iter()
            .any(|a| a.component == "schedule_2_premium_credit_repayment") =>
        {
            assert_eq!(line("18", "a"), 100000);
            assert_eq!(line("18", "b"), if injured == Owner::Taxpayer { 70000 } else { 30000 });
            assert_eq!(line("18", "b") + line("18", "c"), 100000);
        }
        "i-standard" | "i-qbi" => {
            assert_eq!(line("15", "a"), 3385000);
            assert_eq!(line("15", "b"), if injured == Owner::Spouse { 1775000 } else { 1610000 });
            if case == "i-qbi" {
                assert_eq!(value(&r, "qbi_deduction"), 2000);
                assert_eq!(
                    line("15", "c"),
                    if injured == Owner::Spouse { 1610000 } else { 1775000 }
                );
            }
        }
        "i-actc" => {
            assert_eq!(line("17", "a"), 170000);
            assert_eq!(line("17", "b"), if injured == Owner::Spouse { 170000 } else { 0 });
        }
        _ => {}
    }
    let value = serde_json::json!({
        "tax": r.tax, "payments": r.payments, "refund": r.refund,
        "balance": r.balance, "lines": lines,
        "source_ids": draft.income.iter().map(|e| e.id).collect::<Vec<_>>()
    });
    fs::write(directory.join("result.json"), serde_json::to_vec_pretty(&value).unwrap()).unwrap();
}

pub(in crate::tests) fn run() {
    let Ok(directory) = std::env::var("TAX_UNIT13_ORIGINAL_DIR") else { return };
    let directory = Path::new(&directory);
    fs::create_dir_all(directory).unwrap();
    let profile = std::env::var("TAX_UNIT13_ORIGINAL_PROFILE").unwrap();
    let step = std::env::var("TAX_UNIT13_ORIGINAL_STEP").unwrap();
    match step.as_str() {
        "seed" => {
            let mut session = restart(fixture(&profile));
            write_native_draft(directory.join("draft.avro"), &session.draft);
            let answers = if profile.starts_with("h-") {
                serde_json::json!({"schedule_2_premium_credit_repayment": if profile.ends_with("-taxpayer") {70000} else {30000}})
            } else {
                serde_json::json!({})
            };
            fs::write(
                directory.join("metadata.json"),
                serde_json::to_vec_pretty(&serde_json::json!({"answers": answers, "section": TaxSection::InjuredSpouse as i32})).unwrap(),
            )
            .unwrap();
            open(directory, &mut session);
            result(directory, &session.draft);
        }
        "open" => {
            let mut session = restart(read_native_draft(directory.join("draft.avro")));
            let request =
                P::from_avro(&fs::read(directory.join("request.payload.avro")).unwrap()).unwrap();
            assert!(matches!(request, P::OpenSection { .. }));
            let before = session.draft.clone();
            let Reply::Response(begin @ P::BeginInjuredSpouse { .. }) = send(&mut session, request)
            else {
                panic!("the actual client link must open the allocation workflow")
            };
            assert_eq!(session.draft, before);
            fs::write(directory.join("begin.avro"), begin.to_avro()).unwrap();
        }
        "calculate" => {
            let mut session = restart(read_native_draft(directory.join("draft.avro")));
            let request =
                P::from_avro(&fs::read(directory.join("request.payload.avro")).unwrap()).unwrap();
            assert!(matches!(request, P::EstimateInjuredSpouse { .. }));
            let before = session.draft.clone();
            let Reply::Response(response) = send(&mut session, request) else {
                panic!("calculation response")
            };
            assert_eq!(session.draft, before);
            fs::write(directory.join("response.avro"), response.to_avro()).unwrap();
        }
        "save" => {
            let mut session = restart(read_native_draft(directory.join("draft.avro")));
            let request =
                P::from_avro(&fs::read(directory.join("request.payload.avro")).unwrap()).unwrap();
            let P::SaveInjuredSpouse { value, .. } = &request else {
                panic!("ordinary complete Save")
            };
            let intended = InjuredSpouseAllocation::from_page(value);
            let previous_revision = session.draft.revision;
            let changed = session.draft.injured_spouse.as_ref() != Some(&intended);
            let mut candidate = session.draft.clone();
            candidate.injured_spouse = Some(intended.clone());
            candidate.revision = previous_revision + i64::from(changed);
            let expected = supported(&candidate);
            let sources = session.draft.income.clone();
            accepted(&send(&mut session, request));
            assert_eq!(session.draft.injured_spouse.as_ref(), Some(&intended));
            assert_eq!(session.draft.income, sources);
            assert_eq!(session.draft.revision, previous_revision + i64::from(changed));
            assert_eq!(supported(&session.draft), expected);
            write_native_draft(directory.join("stored.avro"), &session.draft);
            write_native_draft(directory.join("draft.avro"), &session.draft);
            result(directory, &session.draft);
            let mut reopened = restart(read_native_draft(directory.join("stored.avro")));
            assert_eq!(supported(&reopened.draft), expected);
            open(directory, &mut reopened);
            fs::copy(directory.join("begin.avro"), directory.join("reopened.avro")).unwrap();
        }
        "reopen" => {
            let mut session = restart(read_native_draft(directory.join("stored.avro")));
            write_native_draft(directory.join("draft.avro"), &session.draft);
            result(directory, &session.draft);
            open(directory, &mut session);
        }
        _ => panic!("unknown editor bridge step: {step}"),
    }
}
