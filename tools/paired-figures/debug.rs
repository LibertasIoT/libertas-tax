//! Lossless parsing of the Rust Debug subset used by complete figure captures.
use crate::{Result, fail};
use serde_json::Value;
#[cfg(test)]
use serde_json::json;
use std::{
    cell::RefCell,
    collections::{BTreeMap, VecDeque, hash_map::DefaultHasher},
    hash::{Hash, Hasher},
    mem::size_of,
    sync::Arc,
};

/// Backends share the lexer and grammar; large comparisons borrow tokens and
/// use packed object members instead of allocating a JSON map for every leaf.
pub(crate) trait Tree<'a>: Sized + PartialEq {
    type Key: Ord;
    fn text_key(&self) -> Option<Self::Key>;
    fn keyed_object(fields: BTreeMap<Self::Key, Self>) -> Self;
    fn string(text: &'a str) -> Self;
    fn object(fields: Vec<(&'a str, Self)>) -> Self;
    fn array(items: Vec<Self>) -> Self;
    fn text(&self) -> Option<&str>;
    fn fields(&self) -> Option<impl Iterator<Item = (&str, &Self)>>;
    fn items(&self) -> Option<&[Self]>;
    fn get(&self, key: &str) -> Option<&Self>;
    fn get_mut(&mut self, key: &str) -> Option<&mut Self>;
    fn insert(&mut self, key: &'a str, value: Self) -> Option<Self>;
    fn remove(&mut self, key: &str) -> Option<Self>;
    fn into_items(self) -> Option<Vec<Self>>;
    fn to_value(&self) -> Value;
}
impl<'a> Tree<'a> for Value {
    type Key = String;
    fn text_key(&self) -> Option<Self::Key> {
        self.as_str().map(str::to_owned)
    }
    fn keyed_object(fields: BTreeMap<Self::Key, Self>) -> Self {
        Self::Object(fields.into_iter().collect())
    }
    fn string(text: &'a str) -> Self {
        Self::String(text.into())
    }
    fn object(fields: Vec<(&'a str, Self)>) -> Self {
        Self::Object(fields.into_iter().map(|(key, value)| (key.into(), value)).collect())
    }
    fn array(items: Vec<Self>) -> Self {
        Self::Array(items)
    }
    fn text(&self) -> Option<&str> {
        self.as_str()
    }
    fn fields(&self) -> Option<impl Iterator<Item = (&str, &Self)>> {
        self.as_object().map(|fields| fields.iter().map(|(key, value)| (key.as_str(), value)))
    }
    fn items(&self) -> Option<&[Self]> {
        self.as_array().map(Vec::as_slice)
    }
    fn get(&self, key: &str) -> Option<&Self> {
        Value::get(self, key)
    }
    fn get_mut(&mut self, key: &str) -> Option<&mut Self> {
        Value::get_mut(self, key)
    }
    fn insert(&mut self, key: &'a str, value: Self) -> Option<Self> {
        self.as_object_mut().expect("object backend").insert(key.into(), value)
    }
    fn remove(&mut self, key: &str) -> Option<Self> {
        self.as_object_mut().expect("object backend").remove(key)
    }
    fn into_items(self) -> Option<Vec<Self>> {
        if let Self::Array(items) = self { Some(items) } else { None }
    }
    fn to_value(&self) -> Value {
        self.clone()
    }
}
#[derive(PartialEq)]
pub(crate) enum CompactTree<'a> {
    String(&'a str),
    Object(Vec<(&'a str, Self)>),
    Array(Vec<Self>),
}
impl<'a> Tree<'a> for CompactTree<'a> {
    type Key = &'a str;
    fn text_key(&self) -> Option<Self::Key> {
        if let Self::String(text) = self { Some(text) } else { None }
    }
    fn keyed_object(fields: BTreeMap<Self::Key, Self>) -> Self {
        Self::Object(fields.into_iter().collect())
    }
    fn string(text: &'a str) -> Self {
        Self::String(text)
    }
    fn object(mut fields: Vec<(&'a str, Self)>) -> Self {
        fields.sort_unstable_by_key(|(key, _)| *key);
        Self::Object(fields)
    }
    fn array(items: Vec<Self>) -> Self {
        Self::Array(items)
    }
    fn text(&self) -> Option<&str> {
        if let Self::String(text) = self { Some(text) } else { None }
    }
    fn fields(&self) -> Option<impl Iterator<Item = (&str, &Self)>> {
        if let Self::Object(fields) = self {
            Some(fields.iter().map(|(key, value)| (*key, value)))
        } else {
            None
        }
    }
    fn items(&self) -> Option<&[Self]> {
        if let Self::Array(items) = self { Some(items) } else { None }
    }
    fn get(&self, key: &str) -> Option<&Self> {
        if let Self::Object(fields) = self {
            fields.binary_search_by_key(&key, |(key, _)| *key).ok().map(|index| &fields[index].1)
        } else {
            None
        }
    }
    fn get_mut(&mut self, key: &str) -> Option<&mut Self> {
        if let Self::Object(fields) = self {
            fields
                .binary_search_by_key(&key, |(key, _)| *key)
                .ok()
                .map(|index| &mut fields[index].1)
        } else {
            None
        }
    }
    fn insert(&mut self, key: &'a str, value: Self) -> Option<Self> {
        let Self::Object(fields) = self else { unreachable!("object backend") };
        match fields.binary_search_by_key(&key, |(key, _)| *key) {
            Ok(index) => Some(std::mem::replace(&mut fields[index].1, value)),
            Err(index) => {
                fields.insert(index, (key, value));
                None
            }
        }
    }
    fn remove(&mut self, key: &str) -> Option<Self> {
        let Self::Object(fields) = self else { unreachable!("object backend") };
        fields.binary_search_by_key(&key, |(key, _)| *key).ok().map(|index| fields.remove(index).1)
    }
    fn into_items(self) -> Option<Vec<Self>> {
        if let Self::Array(items) = self { Some(items) } else { None }
    }
    fn to_value(&self) -> Value {
        match self {
            Self::String(text) => Value::String((*text).into()),
            Self::Object(fields) => Value::Object(
                fields.iter().map(|(key, value)| ((*key).into(), value.to_value())).collect(),
            ),
            Self::Array(items) => Value::Array(items.iter().map(Tree::to_value).collect()),
        }
    }
}
fn tree_node<'a, T: Tree<'a>>(key: &'a str, value: T) -> T {
    T::object(vec![(key, value)])
}

pub struct Parser<'a> {
    text: &'a str,
    offset: usize,
    lookahead: Option<&'a str>,
    retain_field_order: bool,
}
impl<'a> Parser<'a> {
    pub fn parse(text: &'a str) -> Result<Value> {
        Self::parse_with_field_order(text, true)
    }
    pub(crate) fn parse_numeric(text: &'a str) -> Result<Value> {
        Self::parse_with_field_order(text, false)
    }
    fn parse_with_field_order(text: &'a str, retain_field_order: bool) -> Result<Value> {
        Self::parse_backend::<Value>(text, retain_field_order)
    }
    pub(crate) fn parse_backend<T: Tree<'a>>(text: &'a str, retain_field_order: bool) -> Result<T> {
        let mut parser = Self { text, offset: 0, lookahead: None, retain_field_order };
        let value = parser.value::<T>()?;
        if parser.peek()?.is_some() {
            return fail("unconsumed Debug tokens");
        }
        Ok(value)
    }
    fn scan(&mut self) -> Result<Option<&'a str>> {
        let bytes = self.text.as_bytes();
        while self.offset < bytes.len() && bytes[self.offset].is_ascii_whitespace() {
            self.offset += 1;
        }
        if self.offset == bytes.len() {
            return Ok(None);
        }
        let start = self.offset;
        let first = bytes[start];
        self.offset += 1;
        match first {
            b'"' | b'\'' => loop {
                if self.offset >= bytes.len() {
                    return fail("unterminated Debug string");
                }
                let next = bytes[self.offset];
                self.offset += 1;
                if next == b'\\' {
                    if self.offset == bytes.len() {
                        return fail("unterminated Debug escape");
                    }
                    self.offset += 1;
                } else if next == first {
                    break;
                }
            },
            b'0'..=b'9' | b'-' => {
                if first == b'-' && !bytes.get(self.offset).is_some_and(u8::is_ascii_digit) {
                    return fail(format!("unknown Debug syntax at {start}"));
                }
                while bytes.get(self.offset).is_some_and(u8::is_ascii_digit) {
                    self.offset += 1;
                }
                if bytes.get(self.offset) == Some(&b'.')
                    && bytes.get(self.offset + 1).is_some_and(u8::is_ascii_digit)
                {
                    self.offset += 1;
                    while bytes.get(self.offset).is_some_and(u8::is_ascii_digit) {
                        self.offset += 1;
                    }
                }
                if matches!(bytes.get(self.offset), Some(b'e' | b'E')) {
                    let exponent = self.offset;
                    self.offset += 1;
                    if matches!(bytes.get(self.offset), Some(b'+' | b'-')) {
                        self.offset += 1;
                    }
                    let digits = self.offset;
                    while bytes.get(self.offset).is_some_and(u8::is_ascii_digit) {
                        self.offset += 1;
                    }
                    if self.offset == digits {
                        self.offset = exponent;
                    }
                }
            }
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                while self.offset < bytes.len() {
                    let ch = self.text[self.offset..].chars().next().unwrap();
                    if ch.is_alphanumeric() || ch == '_' {
                        self.offset += ch.len_utf8();
                    } else if bytes.get(self.offset..self.offset + 2) == Some(b"::") {
                        let next = self.offset + 2;
                        if self
                            .text
                            .get(next..)
                            .and_then(|s| s.chars().next())
                            .is_some_and(|c| c.is_alphanumeric() || c == '_')
                        {
                            self.offset = next;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
            b'{' | b'}' | b'[' | b']' | b'(' | b')' | b',' | b':' => {}
            _ => {
                return fail(format!(
                    "unknown Debug syntax at {start}: {:?}",
                    self.text[start..].chars().take(100).collect::<String>()
                ));
            }
        }
        Ok(Some(&self.text[start..self.offset]))
    }
    fn peek(&mut self) -> Result<Option<&'a str>> {
        if self.lookahead.is_none() {
            self.lookahead = self.scan()?;
        }
        Ok(self.lookahead)
    }
    fn pop(&mut self) -> Result<&'a str> {
        self.peek()?;
        self.lookahead.take().ok_or_else(|| "unexpected end of Debug payload".into())
    }
    fn expect(&mut self, expected: &str) -> Result<()> {
        let got = self.pop()?;
        if got != expected {
            return fail(format!("expected {expected:?}, got {got:?}"));
        }
        Ok(())
    }
    fn sequence<T: Tree<'a>>(&mut self, close: &str) -> Result<Vec<T>> {
        let mut items = Vec::new();
        while self.peek()? != Some(close) {
            items.push(self.value()?);
            if self.peek()? != Some(close) {
                self.expect(",")?;
            }
        }
        self.expect(close)?;
        Ok(items)
    }
    fn value<T: Tree<'a>>(&mut self) -> Result<T> {
        let token = self.pop()?;
        let enclosing_field_order = self.retain_field_order;
        if token.rsplit("::").next() == Some("Currency") {
            self.retain_field_order = true;
        }
        let value = match token {
            "[" | "(" => Ok(tree_node(
                "$items",
                T::array(self.sequence(if token == "[" { "]" } else { ")" })?),
            )),
            "{" => {
                let mut entries = Vec::new();
                while self.peek()? != Some("}") {
                    let key = self.value()?;
                    let entry = if self.peek()? == Some(":") {
                        self.expect(":")?;
                        vec![key, self.value()?]
                    } else {
                        vec![key]
                    };
                    entries.push(T::array(entry));
                    if self.peek()? != Some("}") {
                        self.expect(",")?;
                    }
                }
                self.expect("}")?;
                Ok(tree_node("$entries", T::array(entries)))
            }
            _ if token.starts_with(['"', '\'']) => Ok(tree_node("$text", T::string(token))),
            _ if token.starts_with('-') || token.as_bytes()[0].is_ascii_digit() => {
                Ok(tree_node("$number", T::string(token)))
            }
            _ if token.starts_with(|ch: char| ch.is_ascii_alphabetic() || ch == '_') => {
                let mut result = tree_node("$tag", T::string(token));
                if self.peek()? == Some("(") {
                    self.expect("(")?;
                    result.insert("$items", T::array(self.sequence(")")?));
                } else if self.peek()? == Some("{") {
                    self.expect("{")?;
                    let mut fields = T::object(Vec::new());
                    let mut field_order = self.retain_field_order.then(Vec::new);
                    while self.peek()? != Some("}") {
                        let name = self.pop()?;
                        self.expect(":")?;
                        if let Some(order) = &mut field_order {
                            order.push(name);
                        }
                        if fields.insert(name, self.value()?).is_some() {
                            return fail(format!("duplicate Debug field {name}"));
                        }
                        if self.peek()? != Some("}") {
                            self.expect(",")?;
                        }
                    }
                    self.expect("}")?;
                    result.insert("$fields", fields);
                    if let Some(order) = field_order {
                        result.insert(
                            "$field_order",
                            T::array(order.into_iter().map(T::string).collect()),
                        );
                    }
                }
                Ok(result)
            }
            _ => fail(format!("unexpected token {token:?}")),
        }?;
        self.retain_field_order = enclosing_field_order;
        Ok(value)
    }
}
pub fn normalize(tree: &mut Value) -> Result<()> {
    normalize_tree(tree)
}
pub(crate) fn normalize_tree<'a, T: Tree<'a>>(tree: &mut T) -> Result<()> {
    if tree.get("$tag").and_then(Tree::text) != Some("FederalResult") {
        return Ok(());
    }
    let lines = tree
        .get("$fields")
        .and_then(|fields| fields.get("lines"))
        .and_then(|lines| lines.get("$items"))
        .and_then(Tree::items)
        .ok_or("missing FederalResult lines")?;
    let line_id = |line: &T| {
        line.get("$fields")
            .and_then(|fields| fields.get("id"))
            .and_then(|id| id.get("$text"))
            .and_then(Tree::text_key)
            .ok_or("missing CalculationLine id")
    };
    // Validate before moving any value so failures leave the tree untouched.
    for line in lines {
        if line.get("$tag").and_then(Tree::text) != Some("CalculationLine") {
            return fail("unexpected FederalResult calculation line");
        }
        line_id(line)?;
    }
    let fields = tree.get_mut("$fields").unwrap();
    let lines = fields.get_mut("lines").unwrap().remove("$items").unwrap().into_items().unwrap();
    let mut by_id: BTreeMap<T::Key, Vec<T>> = BTreeMap::new();
    for line in lines {
        by_id.entry(line_id(&line).expect("validated calculation line id")).or_default().push(line);
    }
    fields.insert(
        "lines",
        tree_node(
            "$by_line_id",
            T::keyed_object(by_id.into_iter().map(|(id, lines)| (id, T::array(lines))).collect()),
        ),
    );
    Ok(())
}
#[derive(Default)]
pub struct Views {
    pub numeric: BTreeMap<String, Value>,
    pub currency: BTreeMap<String, Value>,
    pub structural: BTreeMap<String, Value>,
}
/// Ignore source-order bookkeeping when testing Currency projection equality.
pub fn currency_semantic(value: &Value) -> Value {
    match value {
        Value::Object(fields) => fields
            .iter()
            .filter(|(key, _)| key.as_str() != "$field_order")
            .map(|(key, value)| (key.clone(), currency_semantic(value)))
            .collect::<serde_json::Map<_, _>>()
            .into(),
        Value::Array(values) => values.iter().map(currency_semantic).collect::<Vec<_>>().into(),
        _ => value.clone(),
    }
}

/// Render the Python projection order while hiding transient source-order data.
pub fn currency_projection_json(value: &Value) -> crate::json::Json {
    use crate::json::{Json, object};
    fn node(value: &Value) -> Json {
        match value {
            Value::Array(values) => Json::Array(values.iter().map(node).collect()),
            Value::Object(fields) => {
                let mut result = Vec::new();
                if let Some(tag) = fields.get("$tag") {
                    result.push(("$tag".to_string(), node(tag)));
                }
                for (key, value) in fields {
                    if key == "$tag" || key == "$field_order" {
                        continue;
                    }
                    let rendered = if key == "$fields" {
                        if let Some(members) = value.as_object() {
                            let mut ordered = Vec::new();
                            if let Some(order) =
                                fields.get("$field_order").and_then(Value::as_array)
                            {
                                for name in order.iter().filter_map(Value::as_str) {
                                    if let Some(value) = members.get(name) {
                                        ordered.push((name.to_string(), node(value)));
                                    }
                                }
                            }
                            for (name, value) in members {
                                if !ordered.iter().any(|(key, _)| key == name) {
                                    ordered.push((name.clone(), node(value)));
                                }
                            }
                            object(ordered)
                        } else {
                            node(value)
                        }
                    } else {
                        node(value)
                    };
                    result.push((key.clone(), rendered));
                }
                object(result)
            }
            _ => Json::Value(value.clone()),
        }
    }
    if let Some(fields) = value.as_object()
        && ["currency_code", "scale", "minor_units"].iter().all(|key| fields.contains_key(*key))
    {
        return object(
            ["currency_code", "scale", "minor_units"]
                .into_iter()
                .map(|key| (key, node(&fields[key]))),
        );
    }
    node(value)
}

pub fn views(tree: &Value, structural: bool) -> Result<Views> {
    fn walk(node: &Value, path: &str, out: &mut Views, structure: bool) -> Result<()> {
        match node {
            Value::Object(fields) => {
                if let Some(number) = fields.get("$number") {
                    out.numeric.insert(path.into(), number.clone());
                }
                if fields
                    .get("$tag")
                    .and_then(Value::as_str)
                    .is_some_and(|tag| tag.rsplit("::").next() == Some("Currency"))
                {
                    let mut money = serde_json::Map::new();
                    for key in ["currency_code", "scale", "minor_units"] {
                        money.insert(
                            key.into(),
                            node["$fields"]
                                .get(key)
                                .ok_or("incomplete Currency Debug fields")?
                                .clone(),
                        );
                    }
                    out.currency.insert(path.into(), money.into());
                }
                if structure && fields.is_empty() {
                    out.structural.insert(path.into(), node.clone());
                }
                for (key, value) in fields {
                    if key != "$field_order" && (key != "$text" || structure) {
                        walk(value, &format!("{path}.{key}"), out, structure)?;
                    }
                }
            }
            Value::Array(values) => {
                if structure && values.is_empty() {
                    out.structural.insert(path.into(), node.clone());
                }
                for (index, value) in values.iter().enumerate() {
                    walk(value, &format!("{path}[{index}]"), out, structure)?;
                }
            }
            _ => {
                if structure {
                    out.structural.insert(path.into(), node.clone());
                }
            }
        }
        Ok(())
    }
    let mut out = Views::default();
    walk(tree, "$", &mut out, structural)?;
    Ok(out)
}

fn validate_currencies<'a, T: Tree<'a>>(tree: &T) -> Result<()> {
    if let Some(fields) = tree.fields() {
        if tree
            .get("$tag")
            .and_then(Tree::text)
            .is_some_and(|tag| tag.rsplit("::").next() == Some("Currency"))
        {
            for key in ["currency_code", "scale", "minor_units"] {
                tree.get("$fields")
                    .and_then(|fields| fields.get(key))
                    .ok_or("incomplete Currency Debug fields")?;
            }
        }
        for (key, value) in fields {
            if key != "$field_order" {
                validate_currencies(value)?;
            }
        }
    } else if let Some(values) = tree.items() {
        for value in values {
            validate_currencies(value)?;
        }
    }
    Ok(())
}
#[derive(Default)]
struct LargeDifferences {
    path: String,
    numeric: Vec<crate::json::Json>,
    currency: Vec<crate::json::Json>,
}
impl LargeDifferences {
    fn walk<'a, T: Tree<'a>>(&mut self, old: Option<&T>, new: Option<&T>) {
        use crate::json::{object, val};
        if old == new {
            return;
        }
        let old_number = old.and_then(|value| value.get("$number"));
        let new_number = new.and_then(|value| value.get("$number"));
        if old_number != new_number && (old_number.is_some() || new_number.is_some()) {
            self.numeric.push(object([
                ("path", val(self.path.clone())),
                ("old", val(old_number.map_or(Value::Null, Tree::to_value))),
                ("new", val(new_number.map_or(Value::Null, Tree::to_value))),
                ("old_present", val(old_number.is_some())),
                ("new_present", val(new_number.is_some())),
            ]));
        }
        let projection = |node: Option<&T>| {
            node.filter(|value| {
                value
                    .get("$tag")
                    .and_then(Tree::text)
                    .is_some_and(|tag| tag.rsplit("::").next() == Some("Currency"))
            })
            .map(|value| {
                Value::Object(
                    ["currency_code", "scale", "minor_units"]
                        .into_iter()
                        .map(|key| {
                            (key.into(), value.get("$fields").unwrap().get(key).unwrap().to_value())
                        })
                        .collect(),
                )
            })
        };
        let old_currency = projection(old);
        let new_currency = projection(new);
        if (old_currency.is_some() || new_currency.is_some())
            && old_currency.as_ref().map(currency_semantic)
                != new_currency.as_ref().map(currency_semantic)
        {
            self.currency.push(object([
                ("path", val(self.path.clone())),
                (
                    "old",
                    old_currency
                        .as_ref()
                        .map_or_else(|| val(Value::Null), currency_projection_json),
                ),
                (
                    "new",
                    new_currency
                        .as_ref()
                        .map_or_else(|| val(Value::Null), currency_projection_json),
                ),
                ("old_present", val(old_currency.is_some())),
                ("new_present", val(new_currency.is_some())),
            ]));
        }
        if let (Some(a), Some(b)) = (old.and_then(Tree::fields), new.and_then(Tree::fields)) {
            for (key, value) in a {
                if key != "$field_order" && key != "$text" {
                    self.field(key, Some(value), new.and_then(|node| node.get(key)));
                }
            }
            for (key, value) in b {
                if key != "$field_order"
                    && key != "$text"
                    && old.and_then(|node| node.get(key)).is_none()
                {
                    self.field(key, None, Some(value));
                }
            }
        } else if let (Some(a), Some(b)) = (old.and_then(Tree::items), new.and_then(Tree::items)) {
            for index in 0..a.len().max(b.len()) {
                self.index(index, a.get(index), b.get(index));
            }
        } else {
            if let Some(old) = old {
                self.only_children(old, true);
            }
            if let Some(new) = new {
                self.only_children(new, false);
            }
        }
    }
    fn field<'a, T: Tree<'a>>(&mut self, key: &str, old: Option<&T>, new: Option<&T>) {
        let len = self.path.len();
        self.path.push('.');
        self.path.push_str(key);
        self.walk(old, new);
        self.path.truncate(len);
    }
    fn index<'a, T: Tree<'a>>(&mut self, index: usize, old: Option<&T>, new: Option<&T>) {
        use std::fmt::Write;
        let len = self.path.len();
        write!(self.path, "[{index}]").expect("writing a path to String");
        self.walk(old, new);
        self.path.truncate(len);
    }
    fn only_children<'a, T: Tree<'a>>(&mut self, node: &T, old: bool) {
        if let Some(fields) = node.fields() {
            for (key, value) in fields {
                if key != "$field_order" && key != "$text" {
                    self.field(key, old.then_some(value), (!old).then_some(value));
                }
            }
        } else if let Some(values) = node.items() {
            for (index, value) in values.iter().enumerate() {
                self.index(index, old.then_some(value), (!old).then_some(value));
            }
        }
    }
}
fn large_tree_differences<'a, T: Tree<'a>>(
    old: &T,
    new: &T,
) -> Result<(Vec<crate::json::Json>, Vec<crate::json::Json>)> {
    // Equal subtrees can still contain malformed Currency nodes. Validate before
    // taking the equality shortcut, as complete numeric views do.
    validate_currencies(old)?;
    validate_currencies(new)?;
    let mut changes = LargeDifferences { path: "$".into(), ..LargeDifferences::default() };
    changes.walk(Some(old), Some(new));
    fn path(item: &crate::json::Json) -> &str {
        let crate::json::Json::Object(fields) = item else {
            unreachable!("difference object");
        };
        let crate::json::Json::Value(Value::String(path)) = &fields[0].1 else {
            unreachable!("difference path");
        };
        path
    }
    changes.numeric.sort_by(|a, b| path(a).cmp(path(b)));
    changes.currency.sort_by(|a, b| path(a).cmp(path(b)));
    Ok((changes.numeric, changes.currency))
}
/// Compare complete large Debug payloads without allocating paths for unchanged numbers.
pub fn large_numeric_differences(
    old_text: &str,
    new_text: &str,
) -> Result<(Vec<crate::json::Json>, Vec<crate::json::Json>)> {
    let mut old = Parser::parse_backend::<CompactTree<'_>>(old_text, false)?;
    let mut new = Parser::parse_backend::<CompactTree<'_>>(new_text, false)?;
    normalize_tree(&mut old)?;
    normalize_tree(&mut new)?;
    large_tree_differences(&old, &new)
}

const NUMERIC_CACHE_ENTRIES: usize = 32;
const NUMERIC_CACHE_BYTES: usize = 16 * 1024 * 1024;

struct CachedViews {
    text: Box<str>,
    hash: u64,
    views: Arc<Views>,
    bytes: usize,
}
struct NumericViewsCache {
    entries: VecDeque<CachedViews>,
    bytes: usize,
    limit: usize,
}
impl Default for NumericViewsCache {
    fn default() -> Self {
        Self::new(NUMERIC_CACHE_BYTES)
    }
}
impl NumericViewsCache {
    fn new(limit: usize) -> Self {
        Self { entries: VecDeque::new(), bytes: 0, limit }
    }
    fn views(&mut self, text: &str) -> Result<Arc<Views>> {
        let hash = if text.len() <= self.limit {
            let mut hasher = DefaultHasher::new();
            text.hash(&mut hasher);
            let hash = hasher.finish();
            if let Some(views) = self.get(text, hash) {
                return Ok(views);
            }
            Some(hash)
        } else {
            None
        };
        let mut tree = Parser::parse_numeric(text)?;
        normalize(&mut tree)?;
        let views = Arc::new(views(&tree, false)?);
        if let Some(hash) = hash {
            self.insert(text, hash, &views);
        }
        Ok(views)
    }
    fn get(&mut self, text: &str, hash: u64) -> Option<Arc<Views>> {
        let index = self
            .entries
            .iter()
            .position(|entry| entry.hash == hash && entry.text.as_ref() == text)?;
        let entry = self.entries.remove(index)?;
        let views = Arc::clone(&entry.views);
        self.entries.push_front(entry);
        Some(views)
    }
    fn insert(&mut self, text: &str, hash: u64, views: &Arc<Views>) {
        let bytes =
            text.len().saturating_add(size_of::<CachedViews>()).saturating_add(views_bytes(views));
        if bytes > self.limit {
            return;
        }
        while self.entries.len() >= NUMERIC_CACHE_ENTRIES
            || self.bytes.saturating_add(bytes) > self.limit
        {
            let Some(entry) = self.entries.pop_back() else {
                break;
            };
            self.bytes -= entry.bytes;
        }
        self.entries.push_front(CachedViews {
            text: text.into(),
            hash,
            views: Arc::clone(views),
            bytes,
        });
        self.bytes += bytes;
    }
}
fn views_bytes(views: &Views) -> usize {
    // Map nodes are implementation-private. Charge four extra pointer widths per
    // entry along with every key capacity, inline value, and nested allocation.
    fn map_bytes(map: &BTreeMap<String, Value>) -> usize {
        map.iter().fold(0usize, |bytes, (key, value)| {
            bytes
                .saturating_add(size_of::<(String, Value)>() + 4 * size_of::<usize>())
                .saturating_add(key.capacity())
                .saturating_add(value_bytes(value))
        })
    }
    fn value_bytes(value: &Value) -> usize {
        match value {
            Value::String(text) => text.capacity(),
            Value::Array(values) => values
                .iter()
                .fold(values.capacity().saturating_mul(size_of::<Value>()), |bytes, value| {
                    bytes.saturating_add(value_bytes(value))
                }),
            Value::Object(fields) => fields.iter().fold(0usize, |bytes, (key, value)| {
                bytes
                    .saturating_add(size_of::<(String, Value)>() + 4 * size_of::<usize>())
                    .saturating_add(key.capacity())
                    .saturating_add(value_bytes(value))
            }),
            _ => 0,
        }
    }
    size_of::<Views>()
        .saturating_add(2 * size_of::<usize>())
        .saturating_add(map_bytes(&views.numeric))
        .saturating_add(map_bytes(&views.currency))
        .saturating_add(map_bytes(&views.structural))
}
std::thread_local! {
    static NUMERIC_VIEWS_CACHE: RefCell<NumericViewsCache> = RefCell::new(NumericViewsCache::default());
}
/// Reuse exact complete Debug payloads without retaining unbounded observations.
/// Structural transformations must run separately for every observation.
pub fn cached_numeric_views(text: &str) -> Result<Arc<Views>> {
    NUMERIC_VIEWS_CACHE.with_borrow_mut(|cache| cache.views(text))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn retired_stable_id_does_not_shift_numbers_and_duplicates_stay_ordered() {
        let mut old=Parser::parse(r#"FederalResult { lines: [CalculationLine { id: "retired", amount: 7 }, CalculationLine { id: "tax", amount: 100 }, CalculationLine { id: "tax", amount: 200 }] }"#).unwrap();
        let mut new=Parser::parse(r#"FederalResult { lines: [CalculationLine { id: "tax", amount: 100 }, CalculationLine { id: "tax", amount: 201 }] }"#).unwrap();
        normalize(&mut old).unwrap();
        normalize(&mut new).unwrap();
        let a = views(&old, false).unwrap();
        let b = views(&new, false).unwrap();
        let changes = crate::compare::delta(&a.numeric, &b.numeric);
        assert_eq!(changes.len(), 2);
        let text = crate::json::Json::Array(changes).render(None);
        assert!(text.contains("retired"));
        assert!(text.contains("201"));
    }
    #[test]
    fn numbers_are_lossless_and_text_is_never_numeric() {
        let tree=Parser::parse(r#"Reply { value: -9999999999999999999999999999, ratio: 1.20e+3, text: "10002", char: '7', map: {3: Some(4), 5}, tuple: (6,) }"#).unwrap();
        let found = views(&tree, false).unwrap();
        assert_eq!(found.numeric["$.$fields.value"], "-9999999999999999999999999999");
        assert_eq!(found.numeric["$.$fields.ratio"], "1.20e+3");
        assert_eq!(found.numeric.len(), 6);
        assert!(Parser::parse("Reply { x: @ }").is_err());
        assert!(Parser::parse("Reply { x: 1, x: 2 }").is_err());
    }
    #[test]
    fn captured_home_use_context_keeps_array_numbers_and_empty_map() {
        let tree = Parser::parse(
            "HomeUseContext { agi: [60000, 60000], itemizes: None, dwelling_agi_cents: {} }",
        )
        .unwrap();
        let found = views(&tree, true).unwrap();
        assert_eq!(found.numeric.len(), 2);
        assert!(found.numeric.values().all(|value| value == "60000"));
        assert!(found.structural.keys().any(|path| path.contains("dwelling_agi_cents")));
        assert!(found.structural.values().any(|value| value == "HomeUseContext"));
    }
    #[test]
    fn every_currency_component_is_preserved() {
        let tree = Parser::parse(
            "Reply { cash: unit::Currency { currency_code: USD, scale: 2, minor_units: 10001 } }",
        )
        .unwrap();
        let found = views(&tree, false).unwrap();
        assert_eq!(found.currency["$.$fields.cash"]["minor_units"], json!({"$number":"10001"}));
        assert_eq!(found.currency["$.$fields.cash"]["currency_code"], json!({"$tag":"USD"}));
    }
}

#[cfg(test)]
mod cache_tests {
    use super::*;
    fn clear() {
        NUMERIC_VIEWS_CACHE.with_borrow_mut(|cache| *cache = NumericViewsCache::default());
    }
    #[test]
    fn repeated_complete_payload_reuses_the_same_numeric_and_currency_views() {
        clear();
        let text = "Reply { amount: 10001, cash: Currency { currency_code: USD, scale: 2, minor_units: 10001 } }";
        let first = cached_numeric_views(text).unwrap();
        let second = cached_numeric_views(text).unwrap();
        assert!(Arc::ptr_eq(&first, &second));
        let direct = views(&Parser::parse(text).unwrap(), false).unwrap();
        assert_eq!(first.numeric, direct.numeric);
        assert_eq!(first.currency, direct.currency);
        assert!(first.structural.is_empty());
        let changed = cached_numeric_views(&text.replace("10001", "10002")).unwrap();
        assert_ne!(first.numeric, changed.numeric);
        let differently_spelled = cached_numeric_views(&format!("{text} ")).unwrap();
        assert!(!Arc::ptr_eq(&first, &differently_spelled));
        assert_eq!(first.numeric, differently_spelled.numeric);
    }
    #[test]
    fn eviction_keeps_previously_returned_views_and_recent_hits_intact() {
        clear();
        let first = cached_numeric_views("Reply { amount: 10001 }").unwrap();
        for number in 0..NUMERIC_CACHE_ENTRIES {
            cached_numeric_views(&format!("Reply {{ amount: {number} }}")).unwrap();
        }
        NUMERIC_VIEWS_CACHE.with_borrow(|cache| {
            assert_eq!(cache.entries.len(), NUMERIC_CACHE_ENTRIES);
            assert!(cache.bytes <= NUMERIC_CACHE_BYTES);
        });
        assert_eq!(first.numeric["$.$fields.amount"], "10001");
        let fresh = cached_numeric_views("Reply { amount: 10001 }").unwrap();
        assert!(!Arc::ptr_eq(&first, &fresh));
        assert_eq!(first.numeric, fresh.numeric);
        let recent = cached_numeric_views("Reply { amount: 31 }").unwrap();
        for number in NUMERIC_CACHE_ENTRIES..2 * NUMERIC_CACHE_ENTRIES - 1 {
            cached_numeric_views(&format!("Reply {{ amount: {number} }}")).unwrap();
        }
        assert!(Arc::ptr_eq(&recent, &cached_numeric_views("Reply { amount: 31 }").unwrap()));
    }
    #[test]
    fn oversized_inputs_are_fully_evaluated_without_cache_retention() {
        let text = r#"Reply { text: "x", amount: 10001 }"#;
        let mut cache = NumericViewsCache::new(text.len() - 1);
        let first = cache.views(text).unwrap();
        let second = cache.views(text).unwrap();
        assert_eq!(first.numeric["$.$fields.amount"], "10001");
        assert_eq!(first.numeric, second.numeric);
        assert!(!Arc::ptr_eq(&first, &second));
        assert!(cache.entries.is_empty());
        assert_eq!(cache.bytes, 0);
    }
    #[test]
    fn view_allocations_are_charged_to_the_byte_budget() {
        let limit = 2048;
        let large = Arc::new(Views {
            numeric: BTreeMap::from([("amount".into(), Value::String("1".repeat(limit / 2)))]),
            ..Views::default()
        });
        let mut cache = NumericViewsCache::new(limit);
        cache.insert("first", 1, &large);
        assert_eq!(cache.entries.len(), 1);
        cache.insert("second", 2, &large);
        assert_eq!(cache.entries.len(), 1);
        assert!(cache.get("first", 1).is_none());
        let oversized = Arc::new(Views {
            numeric: BTreeMap::from([("amount".into(), Value::String("1".repeat(limit)))]),
            ..Views::default()
        });
        cache.insert("oversized", 3, &oversized);
        assert_eq!(cache.entries.len(), 1);
        assert!(cache.get("oversized", 3).is_none());
        assert!(cache.bytes <= limit);
    }
}

#[cfg(test)]
mod numeric_mode_tests {
    use super::*;
    fn has_field_order(value: &Value) -> bool {
        match value {
            Value::Object(fields) => {
                fields.contains_key("$field_order") || fields.values().any(has_field_order)
            }
            Value::Array(values) => values.iter().any(has_field_order),
            _ => false,
        }
    }
    #[test]
    fn numeric_mode_preserves_normalized_duplicates_and_nested_currency_members() {
        let text = r#"FederalResult { z: 9, lines: [CalculationLine { id: "tax", amount: 10001, cash: Currency { currency_code: Code { z: "USD", a: 7 }, scale: Some(Scale { value: 2 }), minor_units: Units([Part { amount: 10001 }]) } }, CalculationLine { id: "other", amount: 20 }, CalculationLine { id: "tax", amount: 20002 }], a: 3 }"#;
        let mut full = Parser::parse(text).unwrap();
        let mut numeric = Parser::parse_numeric(text).unwrap();
        assert_eq!(full["$field_order"], json!(["z", "lines", "a"]));
        assert!(has_field_order(&full));
        assert!(numeric.get("$field_order").is_none());
        assert!(numeric["$fields"]["lines"]["$items"][0].get("$field_order").is_none());
        assert!(has_field_order(&numeric["$fields"]["lines"]["$items"][0]["$fields"]["cash"]));
        assert!(numeric["$fields"]["lines"]["$items"][1].get("$field_order").is_none());
        normalize(&mut full).unwrap();
        normalize(&mut numeric).unwrap();
        let full = views(&full, false).unwrap();
        let numeric = views(&numeric, false).unwrap();
        assert_eq!(full.numeric, numeric.numeric);
        assert_eq!(
            currency_semantic(&json!(full.currency)),
            currency_semantic(&json!(numeric.currency))
        );
        assert_eq!(
            numeric.numeric["$.$fields.lines.$by_line_id.\"tax\"[0].$fields.amount"],
            "10001"
        );
        assert_eq!(
            numeric.numeric["$.$fields.lines.$by_line_id.\"tax\"[1].$fields.amount"],
            "20002"
        );
        assert!(!numeric.currency.is_empty());
        for currency in full.currency.values() {
            assert!(has_field_order(currency));
            assert!(!has_field_order(&currency_semantic(currency)));
            assert!(!currency_projection_json(currency).render(None).contains("$field_order"));
        }
        let cached = cached_numeric_views(text).unwrap();
        assert_eq!(cached.numeric, numeric.numeric);
        assert_eq!(
            currency_semantic(&json!(cached.currency)),
            currency_semantic(&json!(numeric.currency))
        );
    }
    #[test]
    fn numeric_mode_keeps_the_full_grammars_error_checks() {
        for text in [
            "Reply { x: @ }",
            "Reply { x: 1, x: 2 }",
            "Reply { x: 1 y: 2 }",
            "Reply { x: [1, }",
            "Reply { x: 1 } trailing",
        ] {
            assert!(Parser::parse(text).is_err(), "{text}");
            assert!(Parser::parse_numeric(text).is_err(), "{text}");
        }
        let full = Parser::parse("Reply { z: Nested { beta: 1, alpha: 2 }, a: 3 }").unwrap();
        assert_eq!(full["$field_order"], json!(["z", "a"]));
        assert_eq!(full["$fields"]["z"]["$field_order"], json!(["beta", "alpha"]));
    }
}

#[cfg(test)]
mod currency_order_tests {
    use super::*;
    #[test]
    fn nested_projection_matches_python_tag_and_original_field_order() {
        let text = r#"Reply { cash: Currency { currency_code: Code { z: "USD", a: 7 }, scale: Some(Scale { z: 2, a: 1 }), minor_units: Units([Part { z: 10001, a: 3 }]) }, tail: Tail { z: 1, a: 2 } }"#;
        let full = Parser::parse(text).unwrap();
        let numeric = Parser::parse_numeric(text).unwrap();
        assert!(numeric["$fields"]["tail"].get("$field_order").is_none());
        let full = views(&full, false).unwrap();
        let numeric = views(&numeric, false).unwrap();
        let expected = r#"{"currency_code":{"$tag":"Code","$fields":{"z":{"$text":"\"USD\""},"a":{"$number":"7"}}},"scale":{"$tag":"Some","$items":[{"$tag":"Scale","$fields":{"z":{"$number":"2"},"a":{"$number":"1"}}}]},"minor_units":{"$tag":"Units","$items":[{"$items":[{"$tag":"Part","$fields":{"z":{"$number":"10001"},"a":{"$number":"3"}}}]}]}}"#;
        assert_eq!(
            currency_projection_json(&full.currency["$.$fields.cash"]).render(None),
            expected
        );
        assert_eq!(
            currency_projection_json(&numeric.currency["$.$fields.cash"]).render(None),
            expected
        );
        let reordered =
            Parser::parse_numeric(&text.replace("z: \"USD\", a: 7", "a: 7, z: \"USD\"")).unwrap();
        let reordered = views(&reordered, false).unwrap();
        assert_eq!(
            currency_semantic(&full.currency["$.$fields.cash"]),
            currency_semantic(&reordered.currency["$.$fields.cash"])
        );
        assert_ne!(
            currency_projection_json(&numeric.currency["$.$fields.cash"]).render(None),
            currency_projection_json(&reordered.currency["$.$fields.cash"]).render(None)
        );
    }
}

#[cfg(test)]
mod large_difference_tests {
    use super::*;
    use crate::json::Json;
    fn json_rows(rows: Vec<Json>) -> Value {
        serde_json::from_str(&Json::Array(rows).render(None)).unwrap()
    }
    fn check(old: &str, new: &str) -> (Value, Value) {
        let mut a = Parser::parse(old).unwrap();
        let mut b = Parser::parse(new).unwrap();
        normalize(&mut a).unwrap();
        normalize(&mut b).unwrap();
        let a = views(&a, false).unwrap();
        let b = views(&b, false).unwrap();
        let expected_numbers = crate::compare::delta(&a.numeric, &b.numeric);
        let semantic = |values: &BTreeMap<String, Value>| {
            values.iter().map(|(key, value)| (key.clone(), currency_semantic(value))).collect()
        };
        let expected_currency =
            crate::compare::delta(&semantic(&a.currency), &semantic(&b.currency));
        let (numbers, currency) = large_numeric_differences(old, new).unwrap();
        assert_eq!(
            Json::Array(numbers.clone()).render(None),
            Json::Array(expected_numbers).render(None)
        );
        let currency_json = json_rows(currency);
        assert_eq!(currency_json, json_rows(expected_currency));
        // Equality ignores source ordering; emitted changes retain it exactly.
        for change in currency_json.as_array().unwrap() {
            let path = change["path"].as_str().unwrap();
            for (side, values) in [("old", &a.currency), ("new", &b.currency)] {
                if let Some(value) = values.get(path) {
                    assert_eq!(
                        change[side],
                        serde_json::from_str::<Value>(
                            &currency_projection_json(value).render(None)
                        )
                        .unwrap()
                    );
                }
            }
        }
        (json_rows(numbers), currency_json)
    }
    #[test]
    fn compact_backend_matches_json_grammar_and_all_small_value_pairs() {
        let values = [
            "0",
            "-9999999999999999999999999999",
            "1.20e+3",
            r#""12\\\"34""#,
            "'7'",
            "None",
            "mod::Some(2)",
            "[]",
            "()",
            "[1, Some(2),]",
            "{3: Some(4), 5}",
            "Tag {}",
            "Tag { z: 1, a: [2, 3] }",
            "Currency { currency_code: USD, scale: 2, minor_units: 10001 }",
            "Currency { minor_units: 10002, scale: 2, currency_code: USD }",
        ];
        for old in values {
            let compact = Parser::parse_backend::<CompactTree<'_>>(old, false).unwrap();
            assert_eq!(compact.to_value(), Parser::parse_numeric(old).unwrap(), "{old}");
            for new in values {
                check(old, new);
            }
        }
    }
    #[test]
    fn compact_backend_preserves_parser_and_normalization_errors() {
        for text in [
            "Reply { x: @ }",
            "Reply { x: 1, x: 2 }",
            "Reply { x: 1 y: 2 }",
            "Reply { x: [1, }",
            "Reply { x: 1 } trailing",
            "-",
            "\"unterminated",
            "FederalResult {}",
            "FederalResult { lines: [Other] }",
            "FederalResult { lines: [CalculationLine { id: 1 }] }",
        ] {
            let expected = Parser::parse_numeric(text).and_then(|mut tree| normalize(&mut tree));
            let actual = Parser::parse_backend::<CompactTree<'_>>(text, false)
                .and_then(|mut tree| normalize_tree(&mut tree));
            assert_eq!(
                actual.unwrap_err().to_string(),
                expected.unwrap_err().to_string(),
                "{text}"
            );
        }
    }
    #[test]
    fn additions_removals_and_node_shape_changes_match_complete_numeric_views() {
        check(
            "Reply { removed: 1, same: [1, 2], node: None, shape: Some(3) }",
            "Reply { added: 4, same: [1, 2, 5], node: Some(9), shape: 3 }",
        );
        check("Reply { node: 1 }", "Reply { node: Some(2) }");
        check("Reply { node: Some(2) }", "Reply { node: 1 }");
    }
    #[test]
    fn array_paths_are_sorted_lexically_after_direct_comparison() {
        let old: Vec<_> = (0..13).map(|n| n.to_string()).collect();
        let mut new: Vec<_> = (0..15).map(|n| n.to_string()).collect();
        new[2] = "99".into();
        new[10] = "98".into();
        let (numbers, _) = check(
            &format!("Reply {{ values: [{}] }}", old.join(",")),
            &format!("Reply {{ values: [{}] }}", new.join(",")),
        );
        let paths: Vec<_> =
            numbers.as_array().unwrap().iter().map(|row| row["path"].as_str().unwrap()).collect();
        assert!(paths[0].contains("[10]"));
        assert!(paths.last().unwrap().contains("[2]"));
    }
    #[test]
    fn stable_line_ids_keep_duplicate_occurrences_and_retired_values() {
        check(
            r#"FederalResult { lines: [CalculationLine { id: "tax", amount: 100 }, CalculationLine { id: "retired", amount: 7 }, CalculationLine { id: "tax", amount: 200 }] }"#,
            r#"FederalResult { lines: [CalculationLine { id: "tax", amount: 101 }, CalculationLine { id: "tax", amount: 200 }] }"#,
        );
    }
    #[test]
    fn nested_currency_reordering_is_semantic_but_changed_output_retains_order() {
        let old = r#"Reply { cash: Currency { currency_code: Code { z: Currency { currency_code: USD, scale: 2, minor_units: 9 }, a: 7 }, scale: Some(Scale { z: 2, a: 1 }), minor_units: 10001 } }"#;
        let reordered = old.replace("z: 2, a: 1", "a: 1, z: 2");
        let (numbers, currency) = check(old, &reordered);
        assert_eq!(numbers, json!([]));
        assert_eq!(currency, json!([]));
        let changed = reordered.replace("minor_units: 9", "minor_units: 10");
        let (_, currency) = check(old, &changed);
        assert_eq!(currency.as_array().unwrap().len(), 2);
        check(old, "Reply { cash: None }");
    }
    #[test]
    fn equal_subtrees_do_not_hide_malformed_currency() {
        let old = "Reply { cash: Currency { scale: 2, minor_units: 10001 } }";
        let new = format!("{old} ");
        assert!(
            large_numeric_differences(old, &new)
                .unwrap_err()
                .to_string()
                .contains("incomplete Currency Debug fields")
        );
    }
}
