use alloc::{boxed::Box, string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

// Empty UI text represents absence in the existing internal optional models.
pub(crate) fn optional_text(value: &str) -> Option<String> {
    (!value.is_empty()).then(|| value.into())
}

/// Answer
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum Answer {
    /// Yes
    Yes,
    /// No
    No,
    /// Not sure
    NotSure,
}

// A two-choice answer in the shape of the internal rules that still read Answer.
pub(crate) fn yes_no(value: bool) -> Answer {
    if value { Answer::Yes } else { Answer::No }
}

/// AmountBasis
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum AmountBasis {
    /// Full-year actual amounts
    /// Use only after you have the complete year, not year-to-date amounts.
    Actual,
    /// Full-year estimates
    /// Enter your own full-year estimates; this App does not annualize partial-year amounts.
    Estimate,
}

/// Owner
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum Owner {
    /// You
    Taxpayer,
    /// Your spouse
    Spouse,
}

/// ResultState
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum ResultState {
    /// More answers needed
    Incomplete,
    /// Outside this prototype’s coverage
    Unsupported,
    /// Estimated for the supported scope
    EstimatedForSupportedScope,
}

// What a calculation line's result is. A count (individuals, full-time
// equivalents, enrolled employees) is not money: the review shows its number in
// the row label, never as dollars.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LineKind {
    #[default]
    Money,
    Count,
    // A ratio in ten-thousandths (Form 1116 lines 3f and 19): the review row
    // shows the decimal, not dollars.
    Ratio,
    // Calendar year, never a currency amount or a grouped count.
    Year,
    // A percentage in hundredths of one percent (3500 means 35%).
    Percentage,
    // The explanation carries the localized text; amount is zero.
    Text,
}

// One calculation step's provenance: internal to the App (rules, tests and
// diagnostics); never part of a reply.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalculationLine {
    // Stable calculation identifier
    pub id: String,
    // Result: USD cents for a money line, hundredths of the number for a count
    pub amount: i64,
    // Whether the result is money or a count
    pub kind: LineKind,
    // Inputs
    // ----
    // Accepted input or earlier calculation identifier
    pub inputs: Vec<String>,
    // Rule identifier
    pub rule: String,
    // Authoritative source
    pub source: String,
    // Explanation
    pub explanation: Message,
    // Form line
    pub form_line: Option<crate::FormLine>,
}

// A message the App builds for display (calculation labels and
// explanations): a StringResources key and its arguments. It is encoded as
// FormattedText only in a reply; the English renderer is test-only.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Message {
    pub key: String,
    pub args: Vec<MessageArg>,
}
// An argument: literal data, a typed value, or a nested resource message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MessageArg {
    Text(String),
    Resource(String),
    Number(i64),
    DateOnly(i64),
    // Bounded decimal quantities such as percentages; currency stays exact.
    Decimal { units: i64, scale: u8 },
    Count(i64),
    Currency { currency_code: String, scale: i64, minor_units: i64 },
    Nested(Box<Message>),
    // A FormattedText value the App stored earlier (a record's label), passed
    // on unchanged as a nested argument.
    Encoded(Vec<u8>),
}
impl MessageArg {
    pub fn usd_dollars(dollars: i64) -> Self {
        Self::Currency { currency_code: "USD".into(), scale: 0, minor_units: dollars }
    }

    pub fn usd_cents(minor_units: i64) -> Self {
        Self::Currency { currency_code: "USD".into(), scale: 2, minor_units }
    }
}
// A fixture label: opaque FormattedText for a record whose label the App names.
#[cfg(test)]
pub(crate) fn test_label(text: &str) -> Vec<u8> {
    Message::new("TEXT", alloc::vec![MessageArg::Text(text.into())]).encode()
}
impl Message {
    pub fn new(key: impl Into<String>, args: Vec<MessageArg>) -> Self {
        let key = key.into();
        #[cfg(test)]
        assert_short_currency_args(&key, &args);
        Self { key, args }
    }
    pub fn encode(&self) -> Vec<u8> {
        use libertas::LibertasMessageArgument as A;
        let nested: Vec<Option<Vec<u8>>> = self
            .args
            .iter()
            .map(|arg| match arg {
                MessageArg::Nested(message) => Some(message.encode()),
                _ => None,
            })
            .collect();
        let args: Vec<A> = self
            .args
            .iter()
            .zip(&nested)
            .map(|(a, bytes)| match a {
                MessageArg::Text(text) => A::LiteralText(text),
                MessageArg::Resource(key) => A::ResourceText(key),
                MessageArg::Number(value) => A::Signed(*value),
                MessageArg::DateOnly(value) => A::DateOnly(*value),
                MessageArg::Count(value) => A::Plural(*value),
                MessageArg::Decimal { units, scale } => {
                    A::Double(*units as f64 / 10_u64.pow(u32::from(*scale)) as f64)
                }
                MessageArg::Currency { currency_code, scale, minor_units } => {
                    A::Currency { currency_code, scale: *scale, minor_units: *minor_units }
                }
                MessageArg::Nested(_) => A::formatted_text(bytes.as_ref().expect("nested tuple")),
                MessageArg::Encoded(value) => A::formatted_text(value),
            })
            .collect();
        libertas::libertas_formatted_text(&self.key, &args)
    }
    // A small English fixture renderer for tests; production sends the tuple.
    #[cfg(test)]
    pub fn english(&self) -> String {
        let template = resource_text(&self.key);
        let args: Vec<String> = self
            .args
            .iter()
            .enumerate()
            .map(|(index, a)| match a {
                MessageArg::Text(text) => text.clone(),
                MessageArg::Resource(key) => render(resource_text(key), &[]),
                MessageArg::Number(value)
                | MessageArg::Count(value)
                | MessageArg::DateOnly(value) => alloc::format!("{value}"),
                MessageArg::Decimal { units, scale } => {
                    let divisor = 10_u64.pow(u32::from(*scale));
                    let magnitude = units.unsigned_abs();
                    let sign = if *units < 0 { "-" } else { "" };
                    alloc::format!(
                        "{sign}{}.{:0width$}",
                        magnitude / divisor,
                        magnitude % divisor,
                        width = usize::from(*scale)
                    )
                }
                MessageArg::Currency { currency_code, scale, minor_units } => {
                    // The fixture locale is English/US; real locale formatting belongs to clients.
                    let prefix = if currency_code == "USD" && short_currency_slot(template, index) {
                        "$".into()
                    } else {
                        alloc::format!("{currency_code}\u{00a0}")
                    };
                    let divisor = 10_u64.pow(*scale as u32);
                    let magnitude = minor_units.unsigned_abs();
                    let sign = if *minor_units < 0 { "-" } else { "" };
                    if *scale == 0 {
                        alloc::format!("{sign}{prefix}{magnitude}")
                    } else {
                        alloc::format!(
                            "{sign}{prefix}{}.{:0width$}",
                            magnitude / divisor,
                            magnitude % divisor,
                            width = *scale as usize
                        )
                    }
                }
                MessageArg::Nested(message) => message.english(),
                MessageArg::Encoded(value) => Message::decode(value).english(),
            })
            .collect();
        render(template, &args)
    }
    // The message a stored FormattedText value encodes (tests only).
    #[cfg(test)]
    pub fn decode(value: &[u8]) -> Self {
        use libertas::{AvroDecode, LibertasMessageArgumentDecode as A};
        let mut offset = 0;
        let name = crate::common::FormattedMessage::avro_decode(value, &mut offset)
            .expect("a FormattedText value");
        assert_eq!(offset, value.len(), "one FormattedText tuple");
        assert!(
            crate::APP_STRINGS.iter().any(|(key, _)| *key == name.0),
            "undeclared resource {}",
            name.0
        );
        let args = name
            .1
            .iter()
            .map(|argument| match argument {
                A::LiteralText(text) => MessageArg::Text(text.clone()),
                A::ResourceText(key) => MessageArg::Resource(key.clone()),
                A::Signed(value) => MessageArg::Number(*value),
                A::DateOnly(value) => MessageArg::DateOnly(*value),
                A::Plural(value) => MessageArg::Count(*value),
                A::Currency { currency_code, scale, minor_units } => MessageArg::Currency {
                    currency_code: currency_code.clone(),
                    scale: *scale,
                    minor_units: *minor_units,
                },
                A::FormattedText(bytes) => MessageArg::Nested(Box::new(Message::decode(bytes))),
                other => panic!("unexpected argument {other:?}"),
            })
            .collect();
        Message::new(name.0, args)
    }
}
// Every synthetic test building a message checks its typed monetary slots.
// This catches new currency-bearing notes beyond the dedicated estimate fixtures.
#[cfg(test)]
fn assert_short_currency_args(key: &str, args: &[MessageArg]) {
    if !args.iter().any(|arg| matches!(arg, MessageArg::Currency { .. })) {
        return;
    }
    let template = resource_text(key);
    for (index, arg) in args.iter().enumerate() {
        if matches!(arg, MessageArg::Currency { .. }) {
            assert!(
                short_currency_slot(template, index),
                "{key} argument {index} must use {{index,number,currency}}: {template}"
            );
        }
    }
}

#[cfg(test)]
pub(crate) fn short_currency_slot(template: &str, index: usize) -> bool {
    let chars: Vec<char> = template.chars().collect();
    let mut quoted = false;
    let mut found = false;
    let mut at = 0;
    while at < chars.len() {
        match chars[at] {
            '\'' if chars.get(at + 1) == Some(&'\'') => at += 1,
            '\'' if quoted || matches!(chars.get(at + 1), Some('{' | '}' | '#')) => {
                quoted = !quoted;
            }
            '{' if !quoted => {
                let end = chars[at + 1..]
                    .iter()
                    .position(|c| matches!(c, ',' | '}'))
                    .map(|end| at + 1 + end)
                    .unwrap_or(chars.len());
                let slot: String = chars[at + 1..end].iter().collect();
                if slot.trim().parse::<usize>().ok() == Some(index) {
                    let close = chars[end..]
                        .iter()
                        .position(|c| *c == '}')
                        .map(|close| end + close)
                        .unwrap_or(chars.len());
                    let presentation: String = chars[end..close].iter().collect();
                    let parts: Vec<_> = presentation.split(',').map(str::trim).collect();
                    if parts != ["", "number", "currency"] {
                        return false;
                    }
                    found = true;
                }
            }
            _ => {}
        }
        at += 1;
    }
    found
}

// A resource template rendered with text arguments, the LMF1 subset the
// App's resources use: {n}, {n,plural,one {...} other {...}} with #, and
// MF1 apostrophe quoting ('' is an apostrophe; '{' quotes a brace).
#[cfg(test)]
pub fn render(template: &str, args: &[String]) -> String {
    let chars: Vec<char> = template.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '\'' if chars.get(i + 1) == Some(&'\'') => {
                out.push('\'');
                i += 2;
            }
            '\'' if matches!(chars.get(i + 1), Some('{' | '}' | '#' | '|')) => {
                i += 1;
                while i < chars.len() && chars[i] != '\'' {
                    out.push(chars[i]);
                    i += 1;
                }
                i += 1;
            }
            '{' => {
                let mut depth = 0;
                let mut j = i;
                while j < chars.len() {
                    match chars[j] {
                        '{' => depth += 1,
                        '}' => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        _ => {}
                    }
                    j += 1;
                }
                let inner: String = chars[i + 1..j.min(chars.len())].iter().collect();
                let mut parts = inner.splitn(3, ',');
                let index: usize = parts.next().unwrap_or("").trim().parse().unwrap_or(usize::MAX);
                let arg = args
                    .get(index)
                    .cloned()
                    .unwrap_or_else(|| panic!("missing message argument {index}: {template}"));
                match (parts.next().map(str::trim), parts.next()) {
                    (Some("plural"), Some(variants)) => {
                        let selector = if arg == "1" { "one" } else { "other" };
                        let body = variants
                            .find(&alloc::format!("{selector} {{"))
                            .map(|at| &variants[at + selector.len() + 2..])
                            .map(|body| {
                                let mut depth = 1;
                                let end = body
                                    .char_indices()
                                    .find_map(|(at, character)| {
                                        match character {
                                            '{' => depth += 1,
                                            '}' => depth -= 1,
                                            _ => {}
                                        }
                                        (depth == 0).then_some(at)
                                    })
                                    .unwrap_or(body.len());
                                &body[..end]
                            })
                            .unwrap_or("");
                        out.push_str(&render(&body.replace('#', &arg), args));
                    }
                    _ => out.push_str(&arg),
                }
                i = j + 1;
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    out
}
#[cfg(test)]
fn resource_text(key: &str) -> &'static str {
    crate::APP_STRINGS
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, text)| *text)
        .unwrap_or_else(|| panic!("undeclared message resource: {key}"))
}

#[cfg(test)]
pub(crate) type FormattedMessage = (String, Vec<libertas::LibertasMessageArgumentDecode>);
