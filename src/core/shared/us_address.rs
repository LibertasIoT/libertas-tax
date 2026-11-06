//! A tolerant reading of a printed US address. The printed text stays as imported
//! (papers are true to form); this only reads it, so two printings of one address
//! compare equal: rows split by newlines or commas, with or without a country
//! ("US", "USA", "United States"), the state as its code or full name, a ZIP or
//! ZIP+4, in any letter case and spacing.
use alloc::{string::String, vec::Vec};

/// The parts of one printed US address
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UsAddress {
    /// Street rows, normalized (lowercase, single spaces, no punctuation)
    pub street: Vec<String>,
    /// City, normalized
    pub city: String,
    pub state: crate::TaxState,
    /// The five-digit ZIP code
    pub zip: String,
}

const STATES: [(&str, crate::TaxState); 56] = {
    use crate::TaxState::*;
    [
        ("alabama", AL),
        ("alaska", AK),
        ("arizona", AZ),
        ("arkansas", AR),
        ("california", CA),
        ("colorado", CO),
        ("connecticut", CT),
        ("delaware", DE),
        ("district of columbia", DC),
        ("florida", FL),
        ("georgia", GA),
        ("hawaii", HI),
        ("idaho", ID),
        ("illinois", IL),
        ("indiana", IN),
        ("iowa", IA),
        ("kansas", KS),
        ("kentucky", KY),
        ("louisiana", LA),
        ("maine", ME),
        ("maryland", MD),
        ("massachusetts", MA),
        ("michigan", MI),
        ("minnesota", MN),
        ("mississippi", MS),
        ("missouri", MO),
        ("montana", MT),
        ("nebraska", NE),
        ("nevada", NV),
        ("new hampshire", NH),
        ("new jersey", NJ),
        ("new mexico", NM),
        ("new york", NY),
        ("north carolina", NC),
        ("north dakota", ND),
        ("ohio", OH),
        ("oklahoma", OK),
        ("oregon", OR),
        ("pennsylvania", PA),
        ("rhode island", RI),
        ("south carolina", SC),
        ("south dakota", SD),
        ("tennessee", TN),
        ("texas", TX),
        ("utah", UT),
        ("vermont", VT),
        ("virginia", VA),
        ("washington", WA),
        ("west virginia", WV),
        ("wisconsin", WI),
        ("wyoming", WY),
        ("american samoa", AS),
        ("guam", GU),
        ("northern mariana islands", MP),
        ("puerto rico", PR),
        ("us virgin islands", VI),
    ]
};
const CODES: [&str; 56] = [
    "al", "ak", "az", "ar", "ca", "co", "ct", "de", "dc", "fl", "ga", "hi", "id", "il", "in", "ia",
    "ks", "ky", "la", "me", "md", "ma", "mi", "mn", "ms", "mo", "mt", "ne", "nv", "nh", "nj", "nm",
    "ny", "nc", "nd", "oh", "ok", "or", "pa", "ri", "sc", "sd", "tn", "tx", "ut", "vt", "va", "wa",
    "wv", "wi", "wy", "as", "gu", "mp", "pr", "vi",
];
const COUNTRIES: [&str; 5] = ["us", "usa", "united states", "united states of america", "u s a"];

// Lowercase words without punctuation; "U.S." reads as "us".
fn words(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|word| {
            word.chars().filter(|c| c.is_alphanumeric()).flat_map(char::to_lowercase).collect()
        })
        .filter(|word: &String| !word.is_empty())
        .collect()
}

fn state_at_end(words: &[String]) -> Option<(usize, crate::TaxState)> {
    if let Some(last) = words.last()
        && let Some(index) = CODES.iter().position(|code| code == last)
    {
        return Some((words.len() - 1, STATES[index].1));
    }
    // The longest name wins, so "West Virginia" is not read as "Virginia".
    STATES
        .iter()
        .filter_map(|(name, state)| {
            let name: Vec<&str> = name.split(' ').collect();
            (words.len() >= name.len() && words[words.len() - name.len()..].iter().eq(name.iter()))
                .then(|| (words.len() - name.len(), *state))
        })
        .min_by_key(|(at, _)| *at)
}

/// The state a printed label names, as a full name or a two-letter code word, and the label's
/// other words joined without punctuation or case ("CA SDI" is CA and "sdi").
pub(crate) fn split_state(label: &str) -> (Option<crate::TaxState>, String) {
    let mut tokens: Vec<String> = label
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(|word| word.chars().flat_map(char::to_lowercase).collect())
        .collect();
    let named = STATES.iter().find_map(|(name, state)| {
        let name: Vec<&str> = name.split(' ').collect();
        tokens
            .windows(name.len())
            .position(|window| window.iter().eq(name.iter()))
            .map(|at| (at, name.len(), *state))
    });
    let coded = || {
        tokens
            .iter()
            .position(|word| CODES.contains(&word.as_str()))
            .map(|at| (at, 1, STATES[CODES.iter().position(|code| *code == tokens[at]).unwrap()].1))
    };
    let found = named.or_else(coded);
    if let Some((at, len, _)) = found {
        tokens.drain(at..at + len);
    }
    (found.map(|(_, _, state)| state), tokens.concat())
}

/// A two-letter state code at the start of joined label text, with the rest ("casdi" is CA, "sdi").
pub(crate) fn code_prefix(joined: &str) -> Option<(crate::TaxState, &str)> {
    let (code, rest) = joined.split_at_checked(2)?;
    let at = CODES.iter().position(|known| *known == code)?;
    Some((STATES[at].1, rest))
}

impl UsAddress {
    /// Reads a printed US address, or None when it has no state and ZIP code.
    pub(crate) fn parse(text: &str) -> Option<Self> {
        let rows: Vec<Vec<String>> =
            text.split(['\n', ',']).map(words).filter(|row| !row.is_empty()).collect();
        // Everything after the street rows, read as one run: city, state, country, ZIP.
        let mut tail: Vec<String> = rows.iter().skip(1).flatten().cloned().collect();
        let zip = tail.pop().filter(|zip| zip.len() == 5 || zip.len() == 9)?;
        if !zip.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        for country in COUNTRIES {
            let n = country.split(' ').count();
            if tail.len() > n && tail[tail.len() - n..].join(" ") == country {
                tail.truncate(tail.len() - n);
                break;
            }
        }
        let (at, state) = state_at_end(&tail)?;
        tail.truncate(at);
        Some(Self {
            street: rows.first().map(|row| alloc::vec![row.join(" ")]).unwrap_or_default(),
            city: tail.join(" "),
            state,
            zip: zip[..5].into(),
        })
    }
}

/// Whether two printed texts are the same US address.
#[cfg(test)]
fn same_us_address(a: &str, b: &str) -> bool {
    match (UsAddress::parse(a), UsAddress::parse(b)) {
        (Some(a), Some(b)) => a == b,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_address_in_every_printing_reads_the_same() {
        let printed = "8 Example Trust Plaza\nExample Grove\nVA US 00000";
        for other in [
            "8 Example Trust Plaza, Example Grove, VA 00000",
            "8 example trust plaza\nEXAMPLE GROVE, va 00000",
            "8 Example Trust Plaza\nExample Grove, Virginia 00000",
            "8 Example Trust Plaza\nExample Grove VIRGINIA USA 00000-1234",
            "8 Example Trust Plaza\nExample Grove, VA, United States, 00000",
            "8 Example Trust Plaza.\nExample Grove VA U.S. 00000",
        ] {
            assert!(same_us_address(printed, other), "{other}");
        }
        let parsed = UsAddress::parse("1 Main St\nCharleston, West Virginia 25301").unwrap();
        assert_eq!((parsed.city.as_str(), parsed.state), ("charleston", crate::TaxState::WV));
        let parsed = UsAddress::parse("1 Main St, Washington, District of Columbia 20001").unwrap();
        assert_eq!(parsed.state, crate::TaxState::DC);
    }

    #[test]
    fn a_different_part_or_no_state_and_zip_is_not_the_same_address() {
        let printed = "8 Example Trust Plaza\nExample Grove\nVA US 00000";
        assert!(!same_us_address(printed, "9 Example Trust Plaza, Example Grove, VA 00000"));
        assert!(!same_us_address(printed, "8 Example Trust Plaza, Example Grove, MD 00000"));
        assert!(!same_us_address(printed, "8 Example Trust Plaza, Example Grove, VA 00001"));
        assert!(UsAddress::parse("8 Example Trust Plaza, Example Grove").is_none());
        assert!(UsAddress::parse("8 Example Trust Plaza, Example Grove, VA").is_none());
    }
}
