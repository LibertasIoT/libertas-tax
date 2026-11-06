//! Explicit one-to-one source transformations used only for strict comparison.
use crate::{Result, fail};
use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

// serde_json normally accepts repeated keys. A declaration must reject them at every depth.
struct Unique(Value);
impl<'de> Deserialize<'de> for Unique {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        struct UniqueVisitor;
        impl<'de> Visitor<'de> for UniqueVisitor {
            type Value = Unique;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("JSON with unique object keys")
            }
            fn visit_map<M: MapAccess<'de>>(
                self,
                mut access: M,
            ) -> std::result::Result<Unique, M::Error> {
                let mut map = serde_json::Map::new();
                while let Some((key, Unique(value))) = access.next_entry::<String, Unique>()? {
                    if map.insert(key.clone(), value).is_some() {
                        return Err(de::Error::custom(format!(
                            "duplicate structural mapping key: {key}"
                        )));
                    }
                }
                Ok(Unique(map.into()))
            }
            fn visit_seq<S: SeqAccess<'de>>(
                self,
                mut access: S,
            ) -> std::result::Result<Unique, S::Error> {
                let mut values = Vec::new();
                while let Some(Unique(value)) = access.next_element()? {
                    values.push(value);
                }
                Ok(Unique(values.into()))
            }
            fn visit_bool<E: de::Error>(self, v: bool) -> std::result::Result<Unique, E> {
                Ok(Unique(v.into()))
            }
            fn visit_i64<E: de::Error>(self, v: i64) -> std::result::Result<Unique, E> {
                Ok(Unique(v.into()))
            }
            fn visit_u64<E: de::Error>(self, v: u64) -> std::result::Result<Unique, E> {
                Ok(Unique(v.into()))
            }
            fn visit_f64<E: de::Error>(self, v: f64) -> std::result::Result<Unique, E> {
                Ok(Unique(json!(v)))
            }
            fn visit_str<E: de::Error>(self, v: &str) -> std::result::Result<Unique, E> {
                Ok(Unique(v.into()))
            }
            fn visit_string<E: de::Error>(self, v: String) -> std::result::Result<Unique, E> {
                Ok(Unique(v.into()))
            }
            fn visit_unit<E: de::Error>(self) -> std::result::Result<Unique, E> {
                Ok(Unique(Value::Null))
            }
        }
        deserializer.deserialize_any(UniqueVisitor)
    }
}
fn keys(value: &Value, expected: &[&str]) -> bool {
    value.as_object().is_some_and(|map| {
        map.len() == expected.len() && expected.iter().all(|key| map.contains_key(*key))
    })
}
#[derive(Clone)]
pub struct Structure {
    paths: BTreeMap<String, Vec<Value>>,
    fields: BTreeSet<(String, String)>,
    catalogs: Vec<Value>,
    field_paths: Vec<Value>,
    pub applied: Vec<(String, u64)>,
}
impl Structure {
    pub fn read(path: &str) -> Result<Self> {
        let mut decoder = serde_json::Deserializer::from_reader(std::fs::File::open(path)?);
        let Unique(spec) = Unique::deserialize(&mut decoder)?;
        decoder.end()?;
        if !keys(&spec, &["version", "variant_paths", "source_fields", "catalogs", "field_paths"])
            || spec["version"] != 1
        {
            return fail("unsupported structural mapping");
        }
        let mut paths: BTreeMap<String, Vec<Value>> = BTreeMap::new();
        for (name, path) in spec["variant_paths"].as_object().ok_or("invalid variant paths")? {
            let path = path.as_array().ok_or("variant paths must be nonempty and one-to-one")?;
            if path.is_empty()
                || path.iter().any(|wrapper| {
                    !(wrapper.as_str().is_some_and(|s| !s.is_empty())
                        || (keys(wrapper, &["tag", "field"])
                            && wrapper
                                .as_object()
                                .unwrap()
                                .values()
                                .all(|v| v.as_str().is_some_and(|s| !s.is_empty()))))
                })
            {
                return fail("variant paths must be nonempty and one-to-one");
            }
            paths.insert(name.clone(), path.clone());
        }
        if paths.is_empty() {
            return fail("variant paths must be nonempty and one-to-one");
        }
        let all: Vec<_> = paths.values().collect();
        for i in 0..all.len() {
            for j in i + 1..all.len() {
                if all[i] == all[j] {
                    return fail("variant paths must be nonempty and one-to-one");
                }
                if all[i].starts_with(all[j]) || all[j].starts_with(all[i]) {
                    return fail("variant paths must not overlap by prefix");
                }
            }
        }
        let mut fields = BTreeSet::new();
        for rule in spec["source_fields"].as_array().ok_or("invalid source fields")? {
            if !keys(rule, &["type", "field"])
                || !fields.insert((
                    rule["type"].as_str().ok_or("invalid source type")?.into(),
                    rule["field"].as_str().ok_or("invalid source field")?.into(),
                ))
            {
                return fail("duplicate or invalid source-field mapping");
            }
        }
        Ok(Self {
            paths,
            fields,
            catalogs: spec["catalogs"].as_array().ok_or("invalid catalogs")?.clone(),
            field_paths: spec["field_paths"].as_array().ok_or("invalid field paths")?.clone(),
            applied: Vec::new(),
        })
    }
    fn count(&mut self, key: String) {
        if let Some((_, count)) = self.applied.iter_mut().find(|(name, _)| name == &key) {
            *count += 1;
        } else {
            self.applied.push((key, 1));
        }
    }
    pub fn apply(&mut self, node: &mut Value, old: bool) -> Result<()> {
        match node {
            Value::Array(values) => {
                for value in values {
                    self.apply(value, old)?;
                }
            }
            Value::Object(map) => {
                let field_order: Vec<String> = map
                    .get("$field_order")
                    .and_then(Value::as_array)
                    .map(|values| {
                        values.iter().filter_map(Value::as_str).map(str::to_string).collect()
                    })
                    .unwrap_or_default();
                // Debug declarations retain source field order for mapping counters.
                // This private metadata is excluded from every comparison view.
                for (key, value) in map.iter_mut() {
                    if key == "$fields" && !field_order.is_empty() {
                        for field in &field_order {
                            if let Some(child) = value.get_mut(field) {
                                self.apply(child, old)?;
                            }
                        }
                    } else if key != "$field_order" {
                        self.apply(value, old)?;
                    }
                }
                let tag = map.get("$tag").and_then(Value::as_str).unwrap_or("").to_string();
                if let Some(fields) = map.get_mut("$fields").and_then(Value::as_object_mut) {
                    if old {
                        let ordered_fields = if field_order.is_empty() {
                            fields.keys().cloned().collect()
                        } else {
                            field_order
                        };
                        for field in &ordered_fields {
                            let Some(value) = fields.get_mut(field) else {
                                continue;
                            };
                            if !self.fields.contains(&(tag.clone(), field.clone())) {
                                continue;
                            }
                            let path = value["$tag"].as_str().and_then(|name| self.paths.get(name));
                            if !keys(value, &["$tag", "$items"])
                                || value["$items"].as_array().is_none_or(|items| items.len() != 1)
                                || path.is_none()
                            {
                                return fail(format!("unknown source variant at {tag}.{field}"));
                            }
                            let mut mapped = value["$items"][0].clone();
                            for wrapper in path.unwrap().iter().rev() {
                                mapped = if let Some(name) = wrapper.as_str() {
                                    json!({"$tag":name,"$items":[mapped]})
                                } else {
                                    let mut fs = serde_json::Map::new();
                                    fs.insert(wrapper["field"].as_str().unwrap().into(), mapped);
                                    json!({"$tag":wrapper["tag"],"$fields":fs})
                                };
                            }
                            *value = mapped;
                            self.count(format!("{tag}.{field}"));
                        }
                        for rule in self.catalogs.clone() {
                            if rule["type"] == tag {
                                self.catalog(fields, &rule)?;
                            }
                        }
                    } else {
                        for rule in self.field_paths.clone() {
                            if rule["type"] == tag
                                && let Some(value) = fields.get_mut(
                                    rule["field"].as_str().ok_or("invalid field-path mapping")?,
                                )
                            {
                                self.field_path(value, &rule)?;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
    fn catalog(&mut self, fields: &mut serde_json::Map<String, Value>, rule: &Value) -> Result<()> {
        if !keys(rule, &["type", "source", "target", "tag", "kinds", "fields"]) {
            return fail("invalid catalog mapping");
        }
        let source = rule["source"].as_str().ok_or("invalid catalog source")?;
        let target = rule["target"].as_str().ok_or("invalid catalog target")?;
        let original = fields.get(source).ok_or("missing original kind catalog")?;
        if !keys(original, &["$items"]) {
            return fail("missing original kind catalog");
        }
        if fields.contains_key(target) {
            return fail("catalog mapping would overwrite a field");
        }
        let mut outputs: BTreeMap<String, BTreeSet<u64>> = BTreeMap::new();
        for field in rule["fields"].as_array().ok_or("invalid catalog fields")? {
            outputs.insert(field.as_str().ok_or("invalid catalog field")?.into(), BTreeSet::new());
        }
        for value in original["$items"].as_array().ok_or("missing original kind catalog")? {
            let kind = value["$number"].as_str().and_then(|n| rule["kinds"].get(n));
            if !keys(value, &["$number"]) || kind.is_none() {
                return fail("unknown original kind catalog index");
            }
            let kind = kind.unwrap().as_object().ok_or("invalid kind catalog mapping")?;
            if kind.is_empty() || kind.keys().any(|key| !outputs.contains_key(key)) {
                return fail("invalid kind catalog mapping");
            }
            for (field, index) in kind {
                if !index.is_null() {
                    outputs
                        .get_mut(field)
                        .unwrap()
                        .insert(index.as_u64().ok_or("invalid kind catalog index")?);
                }
            }
        }
        let mapped: BTreeMap<_,_> = outputs.into_iter().map(|(field,values)| (field,json!({"$items":values.into_iter().map(|n| json!({"$number":n.to_string()})).collect::<Vec<_>>()}))).collect();
        fields.insert(target.into(), json!({"$tag":rule["tag"],"$fields":mapped}));
        self.count(format!("{}.{}->{}", rule["type"].as_str().unwrap_or(""), source, target));
        Ok(())
    }
    fn field_path(&mut self, value: &mut Value, rule: &Value) -> Result<()> {
        if !keys(rule, &["type", "field", "prefixes", "inserted_field"]) {
            return fail("invalid field-path mapping");
        }
        if *value == json!({"$tag":"None"}) {
            return Ok(());
        }
        if !keys(value, &["$tag", "$items"])
            || value["$tag"] != "Some"
            || value["$items"].as_array().is_none_or(|a| a.len() != 1)
            || !keys(&value["$items"][0], &["$text"])
        {
            return fail("expected optional field-path text");
        }
        let text =
            value["$items"][0]["$text"].as_str().ok_or("expected optional field-path text")?;
        let path: String = serde_json::from_str(text)?;
        if !path.starts_with('[') || !path.ends_with(']') {
            return fail("expected square field path");
        }
        let mut segments: Vec<_> = path[1..path.len() - 1].split('/').collect();
        for prefix in rule["prefixes"].as_array().ok_or("invalid field-path prefixes")? {
            let prefix = prefix.as_array().ok_or("invalid field-path prefix")?;
            if segments.len() <= prefix.len()
                || !segments.iter().zip(prefix).all(|(actual, expected)| {
                    if expected == "#" {
                        !actual.is_empty() && actual.bytes().all(|b| b.is_ascii_digit())
                    } else {
                        expected == *actual
                    }
                })
            {
                continue;
            }
            if rule["inserted_field"] == segments[prefix.len()] {
                segments.remove(prefix.len());
                value["$items"][0]["$text"] =
                    crate::json::ascii_string(&format!("[{}]", segments.join("/"))).into();
                self.count(format!(
                    "{}.{}.path.{}",
                    rule["type"].as_str().unwrap_or(""),
                    rule["field"].as_str().unwrap_or(""),
                    rule["inserted_field"].as_str().unwrap_or("")
                ));
                break;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn spec() -> Value {
        json!({"version":1,"variant_paths":{"W2":[{"tag":"Wage","field":"value"},"W2"]},"source_fields":[{"type":"IncomeDocumentItem","field":"document"}],"catalogs":[],"field_paths":[{"type":"Problem","field":"field","prefixes":[["document"],["documents","documents","#","document"]],"inserted_field":"value"}]})
    }
    fn mapping(value: Value, temp: &crate::TempDir) -> Result<Structure> {
        let path = temp.0.join(format!("mapping-{}.json", crate::unique()));
        std::fs::write(&path, value.to_string())?;
        Structure::read(path.to_str().unwrap())
    }
    #[test]
    fn declared_source_wrap_preserves_payload_and_unrelated_text() {
        let temp = crate::TempDir::new(&std::env::temp_dir()).unwrap();
        let mut map = mapping(spec(), &temp).unwrap();
        let mut old=crate::debug::Parser::parse(r#"IncomeDocumentItem { id: 123, document: W2(Source { cents: -10001, text: "same", values: [] }) }"#).unwrap();
        let new=crate::debug::Parser::parse(r#"IncomeDocumentItem { id: 123, document: Wage { value: W2(Source { cents: -10001, text: "same", values: [] }) } }"#).unwrap();
        map.apply(&mut old, true).unwrap();
        assert_eq!(
            crate::debug::views(&old, true).unwrap().structural,
            crate::debug::views(&new, true).unwrap().structural
        );
        assert_eq!(map.applied, vec![("IncomeDocumentItem.document".into(), 1)]);
        let mut path=crate::debug::Parser::parse(r#"Problem { field: Some("[documents/documents/123/document/value/Business/assets/17/basis]") }"#).unwrap();
        map.apply(&mut path, false).unwrap();
        assert_eq!(
            path["$fields"]["field"]["$items"][0]["$text"],
            r#""[documents/documents/123/document/Business/assets/17/basis]""#
        );
    }
    #[test]
    fn duplicate_keys_colliding_paths_and_unknown_variants_fail() {
        let temp = crate::TempDir::new(&std::env::temp_dir()).unwrap();
        let path = temp.0.join("duplicate.json");
        std::fs::write(&path, r#"{"version":1,"version":1}"#).unwrap();
        assert!(
            Structure::read(path.to_str().unwrap())
                .err()
                .unwrap()
                .to_string()
                .contains("duplicate structural mapping key")
        );
        let mut invalid = spec();
        invalid["variant_paths"]["Other"] = invalid["variant_paths"]["W2"].clone();
        assert!(mapping(invalid, &temp).err().unwrap().to_string().contains("one-to-one"));
        let mut map = mapping(spec(), &temp).unwrap();
        let mut tree =
            crate::debug::Parser::parse("IncomeDocumentItem { document: Unknown(1) }").unwrap();
        assert!(
            map.apply(&mut tree, true).unwrap_err().to_string().contains("unknown source variant")
        );
    }
    #[test]
    fn mapping_counts_follow_source_field_traversal_order() {
        let temp = crate::TempDir::new(&std::env::temp_dir()).unwrap();
        let mut declaration = spec();
        declaration["source_fields"] =
            json!([{"type":"Row","field":"z"},{"type":"Row","field":"a"}]);
        let mut map = mapping(declaration, &temp).unwrap();
        let mut tree = crate::debug::Parser::parse("Row { z: W2(1), a: W2(2) }").unwrap();
        map.apply(&mut tree, true).unwrap();
        assert_eq!(map.applied, vec![("Row.z".into(), 1), ("Row.a".into(), 1)]);
    }
}
