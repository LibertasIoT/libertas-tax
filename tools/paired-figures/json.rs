//! Python-compatible JSON rendering, including ASCII escaping and insertion order.
use serde_json::Value;

#[derive(Clone, Debug)]
pub enum Json {
    Value(Value),
    Object(Vec<(String, Json)>),
    Array(Vec<Json>),
}
impl From<Value> for Json {
    fn from(value: Value) -> Self {
        Self::Value(value)
    }
}
pub fn object(items: impl IntoIterator<Item = (impl Into<String>, Json)>) -> Json {
    Json::Object(items.into_iter().map(|(key, value)| (key.into(), value)).collect())
}
pub fn val(value: impl Into<Value>) -> Json {
    Json::Value(value.into())
}
pub fn ascii_string(text: &str) -> String {
    let mut out = String::from("\"");
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            c if (' '..='~').contains(&c) => out.push(c),
            c => {
                for unit in c.encode_utf16(&mut [0; 2]) {
                    out.push_str(&format!("\\u{unit:04x}"));
                }
            }
        }
    }
    out.push('"');
    out
}
/// Python's JSON float spelling uses a two-digit exponent and switches to
/// scientific notation below 1e-4 or at 1e16. Integer tokens remain exact.
pub fn number_text(number: &serde_json::Number) -> String {
    let text = number.to_string();
    if !number.is_f64() {
        return text;
    }
    let (sign, unsigned) = text.strip_prefix('-').map_or(("", text.as_str()), |s| ("-", s));
    let (mantissa, exponent) = unsigned
        .split_once('e')
        .map_or((unsigned, 0), |(m, e)| (m, e.parse::<i32>().expect("JSON number exponent")));
    let digits = mantissa.replace('.', "");
    let Some(first) = digits.bytes().position(|byte| byte != b'0') else {
        return text;
    };
    let decimal = mantissa.find('.').unwrap_or(mantissa.len()) as i32;
    let exponent = exponent + decimal - first as i32 - 1;
    let digits = digits[first..].trim_end_matches('0');
    if !(-4..16).contains(&exponent) {
        let fraction = if digits.len() > 1 { format!(".{}", &digits[1..]) } else { String::new() };
        return format!("{sign}{}{fraction}e{exponent:+03}", &digits[..1]);
    }
    let decimal = exponent + 1;
    if decimal <= 0 {
        format!("{sign}0.{}{digits}", "0".repeat((-decimal) as usize))
    } else if decimal as usize >= digits.len() {
        format!("{sign}{digits}{}.0", "0".repeat(decimal as usize - digits.len()))
    } else {
        format!("{sign}{}.{}", &digits[..decimal as usize], &digits[decimal as usize..])
    }
}

impl Json {
    pub fn render(&self, indent: Option<usize>) -> String {
        let mut out = Vec::new();
        self.write_to(&mut out, indent).expect("writing to a Vec cannot fail");
        String::from_utf8(out).expect("JSON rendering is UTF-8")
    }
    pub fn write_to(
        &self,
        out: &mut impl std::io::Write,
        indent: Option<usize>,
    ) -> std::io::Result<()> {
        self.write(out, indent, 0)
    }
    fn write(
        &self,
        out: &mut impl std::io::Write,
        indent: Option<usize>,
        depth: usize,
    ) -> std::io::Result<()> {
        match self {
            Self::Value(value) => Self::write_value(value, out, indent, depth)?,
            Self::Object(items) => {
                out.write_all(b"{")?;
                for (index, (key, value)) in items.iter().enumerate() {
                    Self::separator(out, indent, depth, index)?;
                    Self::key(out, key, indent)?;
                    value.write(out, indent, depth + 1)?;
                }
                Self::close(out, indent, depth, items.len())?;
                out.write_all(b"}")?;
            }
            Self::Array(items) => {
                out.write_all(b"[")?;
                for (index, value) in items.iter().enumerate() {
                    Self::separator(out, indent, depth, index)?;
                    value.write(out, indent, depth + 1)?;
                }
                Self::close(out, indent, depth, items.len())?;
                out.write_all(b"]")?;
            }
        }
        Ok(())
    }
    fn write_value(
        value: &Value,
        out: &mut impl std::io::Write,
        indent: Option<usize>,
        depth: usize,
    ) -> std::io::Result<()> {
        match value {
            Value::Object(items) => {
                out.write_all(b"{")?;
                for (index, (key, value)) in items.iter().enumerate() {
                    Self::separator(out, indent, depth, index)?;
                    Self::key(out, key, indent)?;
                    Self::write_value(value, out, indent, depth + 1)?;
                }
                Self::close(out, indent, depth, items.len())?;
                out.write_all(b"}")?;
            }
            Value::Array(items) => {
                out.write_all(b"[")?;
                for (index, value) in items.iter().enumerate() {
                    Self::separator(out, indent, depth, index)?;
                    Self::write_value(value, out, indent, depth + 1)?;
                }
                Self::close(out, indent, depth, items.len())?;
                out.write_all(b"]")?;
            }
            Value::String(text) => out.write_all(ascii_string(text).as_bytes())?,
            Value::Number(number) => out.write_all(number_text(number).as_bytes())?,
            scalar => out.write_all(scalar.to_string().as_bytes())?,
        }
        Ok(())
    }
    fn key(out: &mut impl std::io::Write, key: &str, indent: Option<usize>) -> std::io::Result<()> {
        out.write_all(ascii_string(key).as_bytes())?;
        out.write_all(if indent.is_some() { b": " } else { b":" })
    }
    fn separator(
        out: &mut impl std::io::Write,
        indent: Option<usize>,
        depth: usize,
        index: usize,
    ) -> std::io::Result<()> {
        if index > 0 {
            out.write_all(b",")?;
        }
        if let Some(width) = indent {
            out.write_all(b"\n")?;
            out.write_all(" ".repeat((depth + 1) * width).as_bytes())?;
        }
        Ok(())
    }
    fn close(
        out: &mut impl std::io::Write,
        indent: Option<usize>,
        depth: usize,
        len: usize,
    ) -> std::io::Result<()> {
        if len > 0
            && let Some(width) = indent
        {
            out.write_all(b"\n")?;
            out.write_all(" ".repeat(depth * width).as_bytes())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ascii_escaping_and_order_match_python() {
        let json = object([("z", val("café 🦀\n")), ("a", Json::Array(vec![]))]);
        assert_eq!(json.render(None), r#"{"z":"caf\u00e9 \ud83e\udd80\n","a":[]}"#);
        assert_eq!(
            json.render(Some(1)),
            "{\n \"z\": \"caf\\u00e9 \\ud83e\\udd80\\n\",\n \"a\": []\n}"
        );
    }
    #[test]
    fn streaming_preserves_borrowed_containers_and_reports_write_failure() {
        let value = val(serde_json::json!({"nested": ["café", {"number": 42}], "empty": {}}));
        let mut stream = Vec::new();
        value.write_to(&mut stream, None).unwrap();
        assert_eq!(stream, br#"{"empty":{},"nested":["caf\u00e9",{"number":42}]}"#);
        for indent in [None, Some(2)] {
            let mut stream = Vec::new();
            value.write_to(&mut stream, indent).unwrap();
            assert_eq!(stream, value.render(indent).as_bytes());
        }
        let mut exhausted = std::io::Cursor::new([0_u8; 2]);
        assert_eq!(
            value.write_to(&mut exhausted, None).unwrap_err().kind(),
            std::io::ErrorKind::WriteZero
        );
    }
}
