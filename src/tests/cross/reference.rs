//! External-reference cases: `tests/fixtures/reference/<source>/<case>.json`, schema in that
//! directory's README.md. Each case enters its pages through the native client Requests (the
//! `irs_examples` helpers), then compares every listed line and the state against figures
//! printed by a source outside the App, collecting all mismatches before failing once.
extern crate std;
use super::interview::{accepted, document, row, save_children};
use super::irs_examples::{enter, example_return, printed};
use super::*;
use alloc::string::String;

#[derive(serde::Deserialize)]
struct Case {
    filing: String,
    birth: u32,
    pages: Vec<PageSpec>,
    expected: Vec<Line>,
    expected_state: String,
    // Notice keys that must appear in the review notes (known gaps).
    #[serde(default)]
    expected_notes: Vec<String>,
    // Gaps excluded from `expected`: documentation only, each names its ledger row.
    #[serde(default)]
    #[allow(dead_code)]
    known_gaps: Vec<serde_json::Value>,
    // Printed lines the App is known to miss: reported, not failed (triage class B/C/D/E).
    #[serde(default)]
    expect_mismatch: Vec<Mismatch>,
}

#[derive(serde::Deserialize)]
#[serde(tag = "page", rename_all = "snake_case")]
enum PageSpec {
    Children { children: Vec<(String, String, u32)> },
    Wages { w2: Vec<Wage> },
}

#[derive(serde::Deserialize)]
struct Wage {
    wages: i64,
    withholding: i64,
    social_wages: i64,
    social_tax: i64,
    medicare_wages: i64,
    medicare_tax: i64,
}

#[derive(serde::Deserialize)]
struct Line {
    form: String,
    line: String,
    amount: i64,
}

#[derive(serde::Deserialize)]
struct Mismatch {
    form: String,
    line: String,
    class: String,
    triage: String,
}

fn form(name: &str) -> TaxForm {
    match name {
        "1040" => TaxForm::Form1040,
        "8812" => TaxForm::Schedule8812,
        "EIC" => TaxForm::ScheduleEic,
        other => panic!("unknown form {other}"),
    }
}

fn check(name: &str, case: &Case, out: &mut Vec<String>) {
    let filing = match case.filing.as_str() {
        "Single" => Filing::Single,
        other => panic!("{name}: unknown filing {other}"),
    };
    let mut session = example_return(filing, case.birth);
    for page in &case.pages {
        match page {
            PageSpec::Children { children } => {
                let people = children.iter().map(|(f, l, b)| row(f, l, *b)).collect();
                let request = save_children(&session.draft, people);
                accepted(&send(&mut session, request));
            }
            PageSpec::Wages { w2 } => {
                let documents = w2
                    .iter()
                    .map(|w| {
                        let c = |v: i64| v * 100;
                        document(
                            0,
                            IncomeDocument::W2(W2 {
                                wages: c(w.wages),
                                withholding: c(w.withholding),
                                social_wages: c(w.social_wages),
                                social_tax: c(w.social_tax),
                                medicare_wages: c(w.medicare_wages),
                                medicare_tax: c(w.medicare_tax),
                                ..super::interview::w2(0)
                            }),
                        )
                    })
                    .collect();
                enter(&mut session, TaxSection::Wages, documents);
            }
        }
    }
    let result = estimate(&session.draft);
    let state = alloc::format!("{:?}", result.state);
    if state != case.expected_state {
        out.push(alloc::format!("{name}: state {state}, expected {}", case.expected_state));
    }
    for key in &case.expected_notes {
        if !result.review_notes.iter().any(|n| &n.display.key == key) {
            out.push(alloc::format!("{name}: missing review note {key}"));
        }
    }
    for want in &case.expected {
        let got = printed(&result, form(&want.form), &want.line);
        let known =
            case.expect_mismatch.iter().find(|m| m.form == want.form && m.line == want.line);
        match known {
            Some(m) if got != want.amount => std::eprintln!(
                "{name}: {} line {}: App {got}, source {} (class {}, {})",
                want.form,
                want.line,
                want.amount,
                m.class,
                m.triage
            ),
            Some(m) => out.push(alloc::format!(
                "{name}: {} line {} now matches ({got}); remove expect_mismatch {}",
                want.form,
                want.line,
                m.triage
            )),
            None if got != want.amount => out.push(alloc::format!(
                "{name}: {} line {}: App {got}, source {}",
                want.form,
                want.line,
                want.amount
            )),
            None => {}
        }
    }
}

#[test]
fn reference_cases_match_their_sources() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/reference");
    let mut out = Vec::new();
    for source in std::fs::read_dir(&root).unwrap() {
        let source = source.unwrap().path();
        if !source.is_dir() {
            continue;
        }
        for file in std::fs::read_dir(&source).unwrap() {
            let path = file.unwrap().path();
            let raw = std::fs::read_to_string(&path).unwrap();
            let case: Case = serde_json::from_str(&raw).unwrap_or_else(|e| panic!("{path:?}: {e}"));
            check(&path.file_stem().unwrap().to_string_lossy(), &case, &mut out);
        }
    }
    assert!(out.is_empty(), "{}", out.join("\n"));
}
