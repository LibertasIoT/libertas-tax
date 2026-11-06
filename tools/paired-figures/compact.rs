//! Bounded diagnostic retention with explicit figure-comparison evidence.
use crate::{Result, debug, debug::Tree, fail};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

pub const DEBUG_LIMIT: usize = 64 * 1024;
// Bound diagnostic AST memory independently of the smaller text retention limit.
pub const DEBUG_PARSE_LIMIT: usize = 64 * 1024 * 1024;
const LEGACY_CAPACITY_AUDIT_ERROR: &str = "digest-only (excluded capacity module)";
pub const SIZE_AUDIT_ERROR: &str = "digest-only (size): Debug exceeds 64 MiB parse limit";

pub fn size_limited(event: &Value) -> bool {
    event["audit_error"].as_str() == Some(SIZE_AUDIT_ERROR)
}

fn exceeds_parse_limit(bytes: u64) -> bool {
    bytes > DEBUG_PARSE_LIMIT as u64
}

pub fn oversized_debug(event: &Value) -> bool {
    event["debug"].as_str().is_some_and(|text| exceeds_parse_limit(text.len() as u64))
}

/// Reduce one complete legacy event without changing its provenance or typed figures.
/// Available fingerprints cover every numeric token and complete Currency projection.
pub fn compact_event(event: &Value) -> Result<Value> {
    if event["version"] != 1 {
        return fail("compact capture requires a version 1 event");
    }
    let text = event["debug"].as_str().ok_or("capture debug must be a complete string")?;
    event["test"].as_str().ok_or("capture test must be a string")?;
    event["index"].as_u64().ok_or("capture index must be an unsigned integer")?;
    event["kind"].as_str().ok_or("capture kind must be a string")?;
    if event.get("money").is_none() {
        return fail("capture is missing typed money");
    }
    let fingerprints = (|| -> Result<Fingerprints> {
        if exceeds_parse_limit(text.len() as u64) {
            return fail(SIZE_AUDIT_ERROR);
        }
        let mut tree = debug::Parser::parse_backend::<debug::CompactTree<'_>>(text, false)?;
        debug::normalize_tree(&mut tree)?;
        let mut fingerprints = Fingerprints::default();
        fingerprints.walk(&tree, &mut "$".to_string())?;
        Ok(fingerprints)
    })();
    let omission = if text.len() > DEBUG_LIMIT { Some("oversize") } else { None };
    // Omitted Debug may be enormous; clone only fields that survive reduction.
    let mut result = Value::Object(
        event
            .as_object()
            .ok_or("capture event must be an object")?
            .iter()
            .filter(|(key, _)| key.as_str() != "debug" || omission.is_none())
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect(),
    );
    let fields = result.as_object_mut().ok_or("capture event must be an object")?;
    fields.insert("version".into(), json!(2));
    fields.insert("debug_sha256".into(), json!(format!("{:x}", Sha256::digest(text.as_bytes()))));
    match fingerprints {
        Ok(fingerprints) => {
            fields.insert(
                "numeric_sha256".into(),
                json!(format!("{:x}", fingerprints.numeric.finalize())),
            );
            fields.insert(
                "currency_sha256".into(),
                json!(format!("{:x}", fingerprints.currency.finalize())),
            );
            fields.insert("numeric_count".into(), json!(fingerprints.numeric_count));
            fields.insert("currency_count".into(), json!(fingerprints.currency_count));
            fields.insert("audit_error".into(), Value::Null);
        }
        Err(error) => {
            // Equal complete Debug digests prove equality even when the diagnostic
            // grammar cannot parse the text. Never turn unavailable audits into zero.
            for key in ["numeric_sha256", "currency_sha256", "numeric_count", "currency_count"] {
                fields.insert(key.into(), Value::Null);
            }
            fields.insert("audit_error".into(), json!(error.to_string()));
        }
    }
    fields.insert("debug_bytes".into(), json!(text.len()));
    fields.insert("debug_retained".into(), json!(omission.is_none()));
    fields.insert("omission_reason".into(), json!(omission));
    if omission.is_some() {
        fields.insert("debug".into(), Value::Null);
    }
    Ok(result)
}

/// Reject malformed fingerprint records before they can establish equality.
pub fn validate_compact_event(event: &Value) -> Result<()> {
    if event["version"] != 2 {
        return fail("expected compact capture version 2");
    }
    event["test"].as_str().ok_or("compact capture test must be a string")?;
    event["kind"].as_str().ok_or("compact capture kind must be a string")?;
    for key in ["index", "debug_bytes"] {
        event[key]
            .as_u64()
            .ok_or_else(|| format!("compact capture {key} must be an unsigned integer"))?;
    }
    if event.get("money").is_none() {
        return fail("compact capture is missing typed money");
    }
    let unavailable = match event.get("audit_error") {
        None | Some(Value::Null) => false,
        Some(Value::String(error)) if !error.trim().is_empty() => true,
        _ => return fail("compact audit_error must be null or a nonempty diagnostic"),
    };
    if unavailable {
        for key in ["numeric_sha256", "currency_sha256", "numeric_count", "currency_count"] {
            if event.get(key) != Some(&Value::Null) {
                return fail("unavailable compact audits must be explicit nulls");
            }
        }
    } else {
        for key in ["numeric_count", "currency_count"] {
            event[key]
                .as_u64()
                .ok_or_else(|| format!("compact capture {key} must be an unsigned integer"))?;
        }
    }
    let hashes: &[&str] = if unavailable {
        &["debug_sha256"]
    } else {
        &["debug_sha256", "numeric_sha256", "currency_sha256"]
    };
    for &key in hashes {
        let text =
            event[key].as_str().ok_or_else(|| format!("compact capture {key} must be a string"))?;
        if text.len() != 64
            || !text.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return fail(format!("compact capture {key} must be a lowercase SHA256"));
        }
    }
    let retained = event["debug_retained"]
        .as_bool()
        .ok_or("compact capture debug_retained must be Boolean")?;
    let length = event["debug_bytes"].as_u64().unwrap();
    // Older compact captures omitted this module before byte budgets were available.
    // Read that evidence as unavailable; new captures use only the generic size guard.
    let legacy_capacity = event["audit_error"].as_str() == Some(LEGACY_CAPACITY_AUDIT_ERROR);
    if legacy_capacity
        && (retained
            || event["omission_reason"] != "capacity"
            || !event["test"].as_str().unwrap().starts_with("tests::core_tests::unit53_capacity::"))
    {
        return fail("invalid legacy capacity omission");
    }
    if size_limited(event) != (exceeds_parse_limit(length) && !legacy_capacity) {
        return fail(
            "size-limited compact audit requires Debug above the parse limit and its size marker",
        );
    }
    if retained {
        let text =
            event["debug"].as_str().ok_or("retained compact Debug must be a complete string")?;
        if event.get("omission_reason") != Some(&Value::Null)
            || text.len() > DEBUG_LIMIT
            || text.len() as u64 != length
            || event["debug_sha256"] != format!("{:x}", Sha256::digest(text.as_bytes()))
        {
            return fail("inconsistent retained compact Debug metadata");
        }
    } else {
        if event.get("debug") != Some(&Value::Null) {
            return fail("omitted compact Debug must be null");
        }
        match event["omission_reason"].as_str() {
            Some("capacity") if legacy_capacity => {}
            Some("oversize") if length > DEBUG_LIMIT as u64 => {}
            _ => return fail("invalid compact Debug omission reason or byte count"),
        }
    }
    Ok(())
}

#[derive(Default)]
struct Fingerprints {
    numeric: Sha256,
    currency: Sha256,
    numeric_count: u64,
    currency_count: u64,
}

// Length framing makes paths and token spellings unambiguous without storing a
// second map of all paths. Only the current traversal path remains allocated.
fn token(hash: &mut Sha256, bytes: &[u8]) {
    hash.update((bytes.len() as u64).to_le_bytes());
    hash.update(bytes);
}

/// Hash the complete typed value with canonical object order and no source order.
fn typed<'a, T: Tree<'a>>(hash: &mut Sha256, node: &T) {
    if let Some(fields) = node.fields() {
        hash.update(b"{");
        for (key, value) in fields {
            if key != "$field_order" {
                token(hash, key.as_bytes());
                typed(hash, value);
            }
        }
        hash.update(b"}");
    } else if let Some(items) = node.items() {
        hash.update(b"[");
        for item in items {
            typed(hash, item);
        }
        hash.update(b"]");
    } else if let Some(value) = node.text() {
        hash.update(b"s");
        token(hash, value.as_bytes());
    } else {
        // Debug AST leaves are strings. Preserve any other JSON scalar's type.
        hash.update(b"v");
        token(hash, node.to_value().to_string().as_bytes());
    }
}

impl Fingerprints {
    fn walk<'a, T: Tree<'a>>(&mut self, node: &T, path: &mut String) -> Result<()> {
        if let Some(fields) = node.fields() {
            if let Some(number) = node.get("$number") {
                let number = number.text().ok_or("numeric Debug token must be a string")?;
                token(&mut self.numeric, path.as_bytes());
                token(&mut self.numeric, number.as_bytes());
                self.numeric_count += 1;
            }
            if node
                .get("$tag")
                .and_then(Tree::text)
                .is_some_and(|tag| tag.rsplit("::").next() == Some("Currency"))
            {
                token(&mut self.currency, path.as_bytes());
                for key in ["currency_code", "minor_units", "scale"] {
                    let value = node
                        .get("$fields")
                        .and_then(|fields| fields.get(key))
                        .ok_or("incomplete Currency Debug fields")?;
                    token(&mut self.currency, key.as_bytes());
                    typed(&mut self.currency, value);
                }
                self.currency_count += 1;
            }
            for (key, value) in fields {
                if key == "$text" || key == "$field_order" {
                    continue;
                }
                let length = path.len();
                path.push('.');
                path.push_str(key);
                self.walk(value, path)?;
                path.truncate(length);
            }
        } else if let Some(values) = node.items() {
            use std::fmt::Write;
            for (index, value) in values.iter().enumerate() {
                let length = path.len();
                write!(path, "[{index}]")?;
                self.walk(value, path)?;
                path.truncate(length);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn event(text: &str) -> Value {
        json!({"version":1,"test":"capture::case","index":7,"kind":"Send",
            "money":{"exact":123},"debug":text})
    }
    fn reduce(text: &str) -> Value {
        let value = compact_event(&event(text)).unwrap();
        validate_compact_event(&value).unwrap();
        value
    }
    #[test]
    fn borrowed_backend_preserves_json_fingerprints_and_typed_hashes() {
        fn fingerprint<'a, T: Tree<'a>>(mut tree: T) -> Result<(String, String, String, u64, u64)> {
            debug::normalize_tree(&mut tree)?;
            let mut fingerprints = Fingerprints::default();
            fingerprints.walk(&tree, &mut "$".into())?;
            let mut full = Sha256::new();
            typed(&mut full, &tree);
            Ok((
                format!("{:x}", fingerprints.numeric.finalize()),
                format!("{:x}", fingerprints.currency.finalize()),
                format!("{:x}", full.finalize()),
                fingerprints.numeric_count,
                fingerprints.currency_count,
            ))
        }
        for text in [
            r#"Reply { words: "7", values: [1.20e+3, -9, 0], map: {3: Some(4), 5}, tuple: (6,) }"#,
            r#"FederalResult { lines: [CalculationLine { id: "b", amount: 2 }, CalculationLine { id: "a", amount: 1 }, CalculationLine { id: "a", amount: 3 }] }"#,
            r#"Reply { cash: Currency { currency_code: Code { z: "USD", a: [Some(1), Tag {}] }, scale: Scale(2), minor_units: Units { z: Currency { currency_code: USD, scale: 2, minor_units: 99 }, a: {3: (4, 5)} } } }"#,
            "Reply { cash: Currency { scale: 2, minor_units: 99 } }",
            "FederalResult { lines: [CalculationLine { id: 1 }] }",
        ] {
            let old = fingerprint(debug::Parser::parse_numeric(text).unwrap());
            let new = fingerprint(
                debug::Parser::parse_backend::<debug::CompactTree<'_>>(text, false).unwrap(),
            );
            match (old, new) {
                (Ok(old), Ok(new)) => {
                    assert_eq!(old, new, "{text}");
                    let event = reduce(text);
                    assert_eq!(event["numeric_sha256"], old.0);
                    assert_eq!(event["currency_sha256"], old.1);
                    assert_eq!(event["numeric_count"], old.3);
                    assert_eq!(event["currency_count"], old.4);
                }
                (Err(old), Err(new)) => {
                    assert_eq!(old.to_string(), new.to_string(), "{text}");
                    assert_eq!(reduce(text)["audit_error"], old.to_string());
                }
                _ => panic!("backend result differs for {text}"),
            }
        }
    }
    #[test]
    fn complete_debug_digest_and_exact_numeric_spelling_cover_every_number() {
        let a = reduce(r#"Reply { words: "7", values: [1.20e+3, -9, 0] }"#);
        let b = reduce(r#"Reply { words: "8", values: [1.20e+3, -9, 0] }"#);
        assert_ne!(a["debug_sha256"], b["debug_sha256"]);
        assert_eq!(a["numeric_sha256"], b["numeric_sha256"]);
        assert_eq!(a["numeric_count"], 3);
        let c = reduce(r#"Reply { words: "7", values: [1200, -9, 0] }"#);
        assert_ne!(a["numeric_sha256"], c["numeric_sha256"]);
        let d = reduce(r#"Reply { words: "7", values: [1.20e+3, -9, 1] }"#);
        assert_ne!(a["numeric_sha256"], d["numeric_sha256"]);
        assert_eq!(a["kind"], "Send");
        assert_eq!(a["index"], 7);
        assert_eq!(a["money"], json!({"exact":123}));
        assert!(a["debug_retained"].as_bool().unwrap());
    }
    #[test]
    fn stable_line_ids_normalize_reordering_but_preserve_duplicate_occurrences() {
        let a = reduce(
            r#"FederalResult { lines: [CalculationLine { id: "a", amount: 1 }, CalculationLine { id: "b", amount: 2 }, CalculationLine { id: "a", amount: 3 }] }"#,
        );
        let b = reduce(
            r#"FederalResult { lines: [CalculationLine { amount: 2, id: "b" }, CalculationLine { id: "a", amount: 1 }, CalculationLine { id: "a", amount: 3 }] }"#,
        );
        assert_eq!(a["numeric_sha256"], b["numeric_sha256"]);
        let c = reduce(
            r#"FederalResult { lines: [CalculationLine { id: "a", amount: 3 }, CalculationLine { id: "b", amount: 2 }, CalculationLine { id: "a", amount: 1 }] }"#,
        );
        assert_ne!(a["numeric_sha256"], c["numeric_sha256"]);
    }
    #[test]
    fn currency_hash_preserves_typed_text_and_ignores_source_field_order() {
        let a = reduce(
            r#"Reply { cash: Currency { currency_code: Code { z: "USD", a: 1 }, scale: 2, minor_units: 99 } }"#,
        );
        let b = reduce(
            r#"Reply { cash: Currency { minor_units: 99, scale: 2, currency_code: Code { a: 1, z: "USD" } } }"#,
        );
        assert_eq!(a["currency_sha256"], b["currency_sha256"]);
        assert_eq!(a["numeric_sha256"], b["numeric_sha256"]);
        assert_eq!(a["currency_count"], 1);
        let c = reduce(
            r#"Reply { cash: Currency { currency_code: Code { z: "EUR", a: 1 }, scale: 2, minor_units: 99 } }"#,
        );
        assert_ne!(a["currency_sha256"], c["currency_sha256"]);
        assert_eq!(a["numeric_sha256"], c["numeric_sha256"]);
        let unavailable =
            compact_event(&event("Reply { cash: Currency { scale: 2, minor_units: 99 } }"))
                .unwrap();
        assert!(unavailable["audit_error"].as_str().unwrap().contains("incomplete Currency"));
        assert!(unavailable["numeric_sha256"].is_null());
        assert!(unavailable["currency_count"].is_null());
        validate_compact_event(&unavailable).unwrap();
    }
    #[test]
    fn debug_retention_is_complete_or_explicitly_absent_and_figures_are_never_omitted() {
        let text = format!("Reply {{ words: \"{}\", amount: 9 }}", "x".repeat(DEBUG_LIMIT));
        let large = reduce(&text);
        assert_eq!(large["debug"], Value::Null);
        assert_eq!(large["debug_bytes"], text.len());
        assert_eq!(large["omission_reason"], "oversize");
        assert_eq!(large["numeric_count"], 1);
        assert_eq!(large["debug_sha256"], format!("{:x}", Sha256::digest(text.as_bytes())));
        for test in [
            "tests::core_tests::unit53_capacity::lots",
            "tests::health::hsa_capacity",
            "tests::at_risk::capacity",
            "tests::credit::capacity",
            "tests::core_tests::unit53_capacity_other::lots",
        ] {
            let mut original = event("Reply { amount: 9 }");
            original["test"] = test.into();
            let compact = compact_event(&original).unwrap();
            assert_eq!(compact["numeric_count"], 1);
            assert_eq!(compact["audit_error"], Value::Null);
            validate_compact_event(&compact).unwrap();
        }
    }
    #[test]
    fn parse_limit_boundary_preserves_unknowns_without_parsing_oversized_debug() {
        assert!(!exceeds_parse_limit(DEBUG_PARSE_LIMIT as u64));
        assert!(exceeds_parse_limit(DEBUG_PARSE_LIMIT as u64 + 1));
        let original = event("@");
        let malformed = compact_event(&original).unwrap();
        assert!(!size_limited(&malformed));
        assert!(malformed["audit_error"].as_str().unwrap().contains("unknown Debug syntax"));
        validate_compact_event(&malformed).unwrap();
        // The omitted record declares its original length without retaining it.
        let mut oversized = malformed;
        oversized["debug_bytes"] = json!(DEBUG_PARSE_LIMIT + 1);
        oversized["audit_error"] = SIZE_AUDIT_ERROR.into();
        oversized["debug_retained"] = false.into();
        oversized["omission_reason"] = "oversize".into();
        oversized["debug"] = Value::Null;
        assert!(size_limited(&oversized));
        assert_eq!(oversized["money"], original["money"]);
        assert_eq!(oversized["numeric_count"], Value::Null);
        assert_eq!(oversized["currency_sha256"], Value::Null);
        validate_compact_event(&oversized).unwrap();
        for (key, invalid) in [
            ("debug_bytes", json!(DEBUG_PARSE_LIMIT)),
            ("audit_error", json!("some other error")),
            ("audit_error", Value::Null),
            ("numeric_count", json!(0)),
            ("debug_retained", json!(true)),
            ("omission_reason", json!("unexpected")),
        ] {
            let mut broken = oversized.clone();
            broken[key] = invalid;
            assert!(validate_compact_event(&broken).is_err(), "{key}");
        }
    }
    #[test]
    fn legacy_capacity_omissions_remain_readable_without_new_name_based_emission() {
        let mut original = event("opaque @");
        original["test"] = "tests::core_tests::unit53_capacity::old".into();
        let mut legacy = compact_event(&original).unwrap();
        assert_eq!(legacy["debug_retained"], true);
        legacy["debug"] = Value::Null;
        legacy["debug_retained"] = false.into();
        legacy["omission_reason"] = "capacity".into();
        legacy["audit_error"] = LEGACY_CAPACITY_AUDIT_ERROR.into();
        validate_compact_event(&legacy).unwrap();
        legacy["test"] = "other::case".into();
        assert!(validate_compact_event(&legacy).is_err());
    }

    #[test]
    fn malformed_metadata_cannot_establish_compact_equality() {
        let good = reduce("Reply { amount: 9 }");
        for (key, invalid) in [
            ("numeric_sha256", json!("bad")),
            ("currency_sha256", json!("A".repeat(64))),
            ("debug_sha256", json!("0".repeat(64))),
            ("numeric_count", json!(-1)),
            ("currency_count", Value::Null),
            ("index", json!(1.5)),
            ("debug_retained", json!("true")),
            ("debug_bytes", json!(1)),
            ("omission_reason", json!("oversize")),
            ("debug", Value::Null),
        ] {
            let mut broken = good.clone();
            broken[key] = invalid;
            assert!(validate_compact_event(&broken).is_err(), "{key}");
        }
        let mut omitted = good;
        omitted["debug"] = Value::Null;
        omitted["debug_retained"] = false.into();
        omitted["omission_reason"] = "oversize".into();
        assert!(validate_compact_event(&omitted).is_err());
        omitted["omission_reason"] = "capacity".into();
        assert!(validate_compact_event(&omitted).is_err());
        omitted = compact_event(&json!({"version":1,"test":"tests::core_tests::unit53_capacity::lots","index":0,"kind":"Send","money":null,"debug":"opaque @"})).unwrap();
        validate_compact_event(&omitted).unwrap();
        omitted.as_object_mut().unwrap().remove("debug");
        assert!(validate_compact_event(&omitted).is_err());
    }
}
