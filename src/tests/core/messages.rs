// The calculation's labels and explanations are StringResources (locale
// independent replies): every key the source names is declared, every
// template's placeholders match its arguments, and an id without a label
// fails.
use super::*;
extern crate std;

fn resource(key: &str) -> Option<&'static str> {
    APP_STRINGS.iter().find(|(k, _)| *k == key).map(|(_, t)| *t)
}
// The highest positional argument a template reads, plus one.
fn arity(template: &str) -> usize {
    let mut count = 0;
    let mut quoted = false;
    let chars: Vec<char> = template.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '\'' if chars.get(i + 1) == Some(&'\'') => i += 1,
            '\'' if quoted || matches!(chars.get(i + 1), Some('{' | '}' | '#' | '|')) => {
                quoted = !quoted
            }
            '{' if !quoted => {
                let digits: alloc::string::String =
                    chars[i + 1..].iter().take_while(|c| c.is_ascii_digit()).collect();
                if let Ok(n) = digits.parse::<usize>() {
                    count = count.max(n + 1);
                }
            }
            _ => {}
        }
        i += 1;
    }
    count
}
fn sources() -> Vec<(std::path::PathBuf, alloc::string::String)> {
    let mut files = Vec::new();
    let mut dirs = vec![std::path::PathBuf::from("src")];
    while let Some(dir) = dirs.pop() {
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                if !path.ends_with("tests") {
                    dirs.push(path);
                }
            } else if path.extension().is_some_and(|e| e == "rs") && !path.ends_with("lib.rs") {
                let text = std::fs::read_to_string(&path).unwrap();
                files.push((path, text));
            }
        }
    }
    files
}

#[test]
fn every_message_key_is_declared_with_matching_arguments() {
    let mut keys = 0;
    for (path, text) in sources() {
        // Every authored calculation, issue, and review-note key the source names.
        let mut rest = text.as_str();
        while let Some(at) = rest.find('"') {
            rest = &rest[at + 1..];
            let Some(end) = rest.find('"') else { break };
            let word = &rest[..end];
            rest = &rest[end + 1..];
            if (word.starts_with("EXPLAIN_")
                || word.starts_with("RETURN_1040_")
                || word.starts_with("ISSUE_")
                || word.starts_with("NOTE_")
                || word.starts_with("WARN_"))
                && word.bytes().all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_')
            {
                assert!(resource(word).is_some(), "{} names undeclared {word}", path.display());
                keys += 1;
            }
            if word.starts_with("SAVING_KIND_")
                && word.bytes().all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_')
            {
                for suffix in ["_ONE", "_MANY"] {
                    let key = alloc::format!("{word}{suffix}");
                    assert!(resource(&key).is_some(), "{} names undeclared {key}", path.display());
                }
            } else if word.starts_with("SAVING_")
                && word.bytes().all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_')
            {
                assert!(resource(word).is_some(), "{} names undeclared {word}", path.display());
            }
        }
        // msg!("KEY", a, b): the template reads exactly those arguments.
        let mut rest = text.as_str();
        while let Some(at) = rest.find("msg!(\"") {
            rest = &rest[at + 6..];
            let key_end = rest.find('"').unwrap();
            let key = &rest[..key_end];
            let after = &rest[key_end + 1..];
            let mut depth = 1;
            let mut args = 0;
            let mut in_string = false;
            let mut prev = ' ';
            for c in after.chars() {
                match c {
                    '"' if prev != '\\' => in_string = !in_string,
                    '(' | '[' | '{' if !in_string => depth += 1,
                    ')' | ']' | '}' if !in_string => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    ',' if !in_string && depth == 1 => args += 1,
                    _ => {}
                }
                prev = c;
            }
            let template = resource(key).unwrap_or_else(|| panic!("undeclared {key}"));
            assert_eq!(arity(template), args, "{}: {key} {template}", path.display());
        }
    }
    assert!(keys > 1_000, "{keys}");
}

#[test]
fn save_warning_templates_keep_typed_count_and_identifying_name_slots() {
    for (key, expected) in [
        ("SAVING_REMOVES_COUNT", 3),
        ("SAVING_REMOVES_ONE_NAMED", 3),
        ("SAVING_REMOVES_NAMED", 2),
        ("SAVING_REOPENS", 1),
    ] {
        assert_eq!(arity(resource(key).unwrap()), expected, "{key}");
    }
}

#[test]
fn every_calculation_label_has_its_resources() {
    // Every LINE_ resource names one pattern (the registry of patterns), N
    // standing for a numbered or named part.
    let patterns: Vec<alloc::string::String> = APP_STRINGS
        .iter()
        .filter_map(|(key, _)| key.strip_prefix("LINE_"))
        .map(|rest| {
            rest.to_lowercase()
                .split('_')
                .map(|p| if p == "n" { "N" } else { p })
                .collect::<Vec<_>>()
                .join("_")
        })
        .collect();
    for pattern in &patterns {
        // The keys come from the one id helper; a pattern with N parts is a
        // valid id whose parts are the arguments.
        // The rows of a general business credit share one pattern per kind,
        // with the credit as the first argument.
        let id = pattern
            .replace('N', "7")
            .replace("business_credit_credit_", "business_credit_work_opportunity_");
        let keys = crate::federal_model::IdKeys::new(&id);
        assert_eq!(&keys.pattern, pattern);
        let parts = keys.args.len();
        let credit = usize::from(pattern.starts_with("business_credit_credit_"));
        assert_eq!(parts, credit + pattern.split('_').filter(|p| *p == "N").count());
        let label = resource(&keys.line).unwrap();
        assert_eq!(arity(label), parts, "{pattern}");
        // A return row is the form, the line and the label, unless the
        // pattern has its own row wording.
        if keys.row_override() {
            assert_eq!(arity(resource(&keys.row).unwrap()), parts + 2, "{pattern}");
        } else {
            assert!(resource(&keys.row).is_none(), "{pattern} has a needless row");
        }
        // An explanation is optional; one under this pattern's key has text.
        if let Some(explain) = resource(&keys.explain) {
            assert!(!explain.is_empty(), "{pattern}");
        }
    }
    // And back: every ROW_ resource belongs to a LINE_ pattern listed in
    // ROW_OVERRIDES, which is sorted for binary search and names only
    // patterns that have a label.
    for (key, _) in APP_STRINGS {
        if let Some(rest) = key.strip_prefix("ROW_").filter(|rest| *rest != "LABEL") {
            assert!(resource(&alloc::format!("LINE_{rest}")).is_some(), "{key} has no pattern");
        }
    }
    assert!(
        crate::line_labels::ROW_OVERRIDES.windows(2).all(|w| w[0] < w[1]),
        "ROW_OVERRIDES is sorted for binary search"
    );
    for pattern in crate::line_labels::ROW_OVERRIDES {
        assert!(patterns.iter().any(|p| p == pattern), "{pattern} is no pattern");
    }
    // Numbered and named parts are arguments.
    let label = crate::line_label("business_12_asset_3_bonus");
    assert_eq!(label.key, "LINE_BUSINESS_N_ASSET_N_BONUS");
    assert_eq!(label.english(), "Business 12 asset 3 bonus");
    assert_eq!(
        crate::line_label("qcd_taxpayer_exclusion").english(),
        "QCD exclusion, taxpayer (qualified charitable distributions)"
    );
    // Form 5329 Part II lines name their filer in words (the digit of the old
    // account_0_taxable became a "0" argument): no arguments, owner words.
    let label = crate::line_label("account_spouse_taxable");
    assert_eq!(label.key, "LINE_ACCOUNT_SPOUSE_TAXABLE");
    assert!(label.args.is_empty());
    assert_eq!(label.english(), "Account distributions, spouse, included in income");
    // Part I the same way (it was "Early 0 taxable").
    let label = crate::line_label("early_spouse_taxable");
    assert!(label.args.is_empty());
    assert_eq!(label.english(), "Early distributions, spouse, taxable");
    let row = crate::FormLine::new(crate::TaxForm::Form5329, "5").row_label("account_taxpayer_tax");
    assert_eq!(row.key, "ROW_LABEL");
    assert_eq!(row.english(), "Form 5329 5 — Account distributions, taxpayer, additional tax");
}

#[test]
#[should_panic(expected = "no label resource for calculation id")]
fn a_calculation_id_without_a_label_fails() {
    crate::line_label("no_such_calculation_line");
}

#[test]
fn typed_nested_messages_keep_their_arguments_on_the_wire() {
    use libertas::{AvroDecode, LibertasMessageArgumentDecode as A};

    let inner = crate::Message::new("REVIEW_NOT_CALCULATED", vec![crate::MessageArg::Count(2)]);
    let message = crate::Message::new(
        "NOTE_WITH_LABEL",
        vec![
            crate::MessageArg::Nested(alloc::boxed::Box::new(inner.clone())),
            crate::MessageArg::Number(7),
        ],
    );
    let encoded = message.encode();
    let mut offset = 0;
    let decoded = crate::common::FormattedMessage::avro_decode(&encoded, &mut offset).unwrap();
    assert_eq!(offset, encoded.len());
    assert_eq!(decoded.0, "NOTE_WITH_LABEL");
    assert_eq!(decoded.1, vec![A::FormattedText(inner.encode()), A::Signed(7)]);
    assert_eq!(status_text(&encoded), "2 is not calculated yet: 7");
}

#[test]
fn exact_currency_message_keeps_cents_and_usd() {
    use libertas::{AvroDecode, LibertasMessageArgumentDecode as A};

    let message = crate::Message::new(
        "ESTIMATED_REFUND",
        vec![crate::MessageArg::Currency {
            currency_code: "USD".into(),
            scale: 2,
            minor_units: -5,
        }],
    );
    let encoded = message.encode();
    let mut offset = 0;
    let decoded = crate::common::FormattedMessage::avro_decode(&encoded, &mut offset).unwrap();
    assert_eq!(offset, encoded.len());
    assert_eq!(
        decoded.1,
        vec![A::Currency { currency_code: "USD".into(), scale: 2, minor_units: -5 }]
    );
}

#[test]
fn attention_keeps_a_message_with_typed_arguments_nested() {
    use libertas::{AvroDecode, LibertasMessageArgumentDecode as A};

    let issue = crate::Message::new(
        "ESTIMATED_BALANCE",
        vec![crate::MessageArg::Currency {
            currency_code: "USD".into(),
            scale: 2,
            minor_units: 12345,
        }],
    )
    .encode();
    let attention = crate::interview::attention_status(&issue);
    let mut offset = 0;
    let decoded = crate::common::FormattedMessage::avro_decode(&attention, &mut offset).unwrap();
    assert_eq!(offset, attention.len());
    assert_eq!(decoded.0, "REVIEW_ATTENTION");
    assert_eq!(decoded.1, vec![A::FormattedText(issue)]);
}

#[test]
fn the_review_breakdown_and_return_lines_are_keyed() {
    // No reply prose: every label and explanation is a declared resource
    // with its arguments; none is the TEXT literal fallback.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Joint));
    let Reply::Response(P::Review { result, .. }) =
        send(&mut s, P::ReviewAccepted { user: TEST_USER })
    else {
        panic!("expected the review")
    };
    assert!(!result.breakdown.is_empty() && !result.return_lines.is_empty());
    for row in result.breakdown.iter().chain(&result.return_lines) {
        for value in [&row.label, &row.explanation] {
            let mut offset = 0;
            let name = <crate::common::FormattedMessage as libertas::AvroDecode>::avro_decode(
                value,
                &mut offset,
            )
            .unwrap();
            assert_ne!(name.0, "TEXT");
            assert!(resource(&name.0).is_some(), "{}", name.0);
            assert!(!status_text(value).is_empty());
        }
    }
    let rows = rows(&result.return_lines);
    assert!(rows.iter().any(|r| r.label == "1040 1a — W-2 wages"));
}

#[test]
fn every_tax_currency_message_requests_the_localized_short_symbol() {
    use crate::common::short_currency_slot;
    for (key, index) in [
        ("ESTIMATED_REFUND", 0),
        ("ESTIMATED_BALANCE", 0),
        ("SUMMARY_DOCUMENTS_WITH_AMOUNT", 1),
        ("REVIEW_USD_AMOUNT", 0),
        ("REVIEW_GAMBLING_LOSSES_FROM", 0),
        ("REVIEW_MARKETPLACE_CREDIT", 1),
        ("REVIEW_MARKETPLACE_REPAY", 1),
        ("NOTE_8938_ABROAD_JOINT_CONDITIONAL", 2),
        ("NOTE_SCORP_LOAN_PRINCIPAL_FORMER_OPEN", 3),
        ("NOTE_INJURED_HALF_OTHER_ADJUSTMENTS", 0),
    ] {
        assert!(short_currency_slot(resource(key).unwrap(), index), "{key}");
    }
    assert!(short_currency_slot(
        "{0,plural,one {Credit {1,number,currency}} other {Credits {1,number,currency}}}",
        1
    ));
    for invalid in ["{0}", "{0,number}", "{0,number,currency} and {0}", "'{0,number,currency}'"] {
        assert!(!short_currency_slot(invalid, 0), "{invalid}");
    }
    assert!(short_currency_slot("Literal '{0}' and {0,number,currency}", 0));
    for (minor_units, expected) in [
        (-5, "Estimated federal refund -$0.05"),
        (676100, "Estimated federal refund $6761.00"),
        (i64::MAX, "Estimated federal refund $92233720368547758.07"),
    ] {
        let message = crate::Message::new(
            "ESTIMATED_REFUND",
            vec![crate::MessageArg::usd_cents(minor_units)],
        );
        assert_eq!(message.english(), expected);
        let encoded = message.encode();
        let mut offset = 0;
        let decoded = <crate::common::FormattedMessage as libertas::AvroDecode>::avro_decode(
            &encoded,
            &mut offset,
        )
        .unwrap();
        assert_eq!(offset, encoded.len());
        assert!(
            matches!(&decoded.1[..], [libertas::LibertasMessageArgumentDecode::Currency { currency_code, scale: 2, minor_units: value }] if currency_code == "USD" && *value == minor_units)
        );
    }
}

// A resource key is declared once: a second entry would never be read, and a
// merge of two branches that each added the same key leaves one silently.
#[test]
fn every_resource_key_is_declared_once() {
    let mut seen = std::collections::BTreeSet::new();
    let duplicates: Vec<&str> =
        APP_STRINGS.iter().filter(|(key, _)| !seen.insert(*key)).map(|(key, _)| *key).collect();
    assert!(duplicates.is_empty(), "{duplicates:?}");
}

// Row 226: the rows of a general business credit are labeled from the credit's
// form resource and the id's kind; the longest credit key names the credit.
#[test]
fn business_credit_rows_are_labeled_from_the_credit_and_the_kind() {
    let label =
        crate::line_label("business_credit_low_income_housing_pre2008_2019_passive_to_next_year");
    assert_eq!(label.key, "LINE_BUSINESS_CREDIT_CREDIT_N_PASSIVE_TO_NEXT_YEAR");
    assert_eq!(label.english(), "Business credit Form 8586 (pre-2008) 2019 passive to 2027");
    let label = crate::line_label("business_credit_low_income_housing_current_year_used");
    assert_eq!(label.english(), "Business credit Form 8586 2026 used");
    let row = crate::FormLine::new(TaxForm::Form3800, "4d")
        .row_label("business_credit_energy_current_year_carryback_2025");
    assert_eq!(
        row.english(),
        "Form 3800 4d — Business credit Form 3468 Part VI 2026 carryback 2025"
    );
}
