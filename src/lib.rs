//! Parsing and normalization for single-line US postal addresses.
//!
//! The expected shape is `STREET, CITY, STATE ZIP`, e.g.
//! `12 Elm St, Springfield, IL 62704`. That's the format most CSV exports
//! and address-book dumps already use, so the parser doesn't try to guess
//! at anything fancier (no multi-line recipient blocks, no apartment-number
//! heuristics). Garbage in gets a specific `ParseError` back, not a guess.

use std::fmt;

/// A parsed, normalized postal address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Address {
    pub street: String,
    pub city: String,
    /// Always a two-letter USPS code, upper case.
    pub state: String,
    /// Either `NNNNN` or `NNNNN-NNNN`.
    pub zip: String,
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}, {} {}", self.street, self.city, self.state, self.zip)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// Didn't find exactly three comma-separated fields.
    Format,
    /// Trailing field wasn't `STATE ZIP`.
    MissingZip,
    /// State token isn't a known two-letter USPS code.
    UnknownState(String),
    /// ZIP isn't `NNNNN` or `NNNNN-NNNN`.
    InvalidZip(String),
    /// Street or city field was empty after trimming.
    EmptyField(&'static str),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Format => {
                write!(f, "expected \"street, city, state zip\" (three comma-separated fields)")
            }
            ParseError::MissingZip => write!(f, "last field must be \"STATE ZIP\""),
            ParseError::UnknownState(s) => write!(f, "unknown state code: {s:?}"),
            ParseError::InvalidZip(z) => write!(f, "invalid zip code: {z:?}"),
            ParseError::EmptyField(name) => write!(f, "{name} is empty"),
        }
    }
}

impl std::error::Error for ParseError {}

/// Parse a single address line of the form `STREET, CITY, STATE ZIP`.
pub fn parse_line(line: &str) -> Result<Address, ParseError> {
    let fields: Vec<&str> = line.trim().split(',').map(str::trim).collect();
    let [street, city, tail] = fields.as_slice() else {
        return Err(ParseError::Format);
    };

    if street.is_empty() {
        return Err(ParseError::EmptyField("street"));
    }
    if city.is_empty() {
        return Err(ParseError::EmptyField("city"));
    }

    let tokens: Vec<&str> = tail.split_whitespace().collect();
    let (state_tokens, zip) = match tokens.split_last() {
        Some((zip, rest)) if !rest.is_empty() => (rest, *zip),
        _ => return Err(ParseError::MissingZip),
    };
    let state = state_tokens.join(" ").to_uppercase();

    if !is_known_state(&state) {
        return Err(ParseError::UnknownState(state));
    }
    if !is_valid_zip(zip) {
        return Err(ParseError::InvalidZip(zip.to_string()));
    }

    Ok(Address {
        street: street.to_string(),
        city: city.to_string(),
        state,
        zip: zip.to_string(),
    })
}

fn is_valid_zip(zip: &str) -> bool {
    match zip.as_bytes() {
        [d1, d2, d3, d4, d5] => [d1, d2, d3, d4, d5].iter().all(|b| b.is_ascii_digit()),
        [d1, d2, d3, d4, d5, b'-', e1, e2, e3, e4] => {
            [d1, d2, d3, d4, d5].iter().all(|b| b.is_ascii_digit())
                && [e1, e2, e3, e4].iter().all(|b| b.is_ascii_digit())
        }
        _ => false,
    }
}

fn is_known_state(code: &str) -> bool {
    STATE_CODES.contains(&code)
}

// USPS two-letter codes: 50 states, DC, inhabited territories, and the
// military "state" codes used for APO/FPO/DPO addresses.
const STATE_CODES: &[&str] = &[
    "AL", "AK", "AZ", "AR", "CA", "CO", "CT", "DE", "FL", "GA", "HI", "ID", "IL", "IN", "IA",
    "KS", "KY", "LA", "ME", "MD", "MA", "MI", "MN", "MS", "MO", "MT", "NE", "NV", "NH", "NJ",
    "NM", "NY", "NC", "ND", "OH", "OK", "OR", "PA", "RI", "SC", "SD", "TN", "TX", "UT", "VT",
    "VA", "WA", "WV", "WI", "WY", "DC", "PR", "VI", "GU", "AS", "MP", "AA", "AE", "AP",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_plain_address() {
        let addr = parse_line("12 Elm St, Springfield, IL 62704").unwrap();
        assert_eq!(addr.street, "12 Elm St");
        assert_eq!(addr.city, "Springfield");
        assert_eq!(addr.state, "IL");
        assert_eq!(addr.zip, "62704");
    }

    #[test]
    fn accepts_zip_plus_four() {
        let addr = parse_line("1 Infinite Loop, Cupertino, CA 95014-2083").unwrap();
        assert_eq!(addr.zip, "95014-2083");
    }

    #[test]
    fn lower_case_state_is_normalized() {
        let addr = parse_line("221B Baker St, London, il 60601").unwrap();
        assert_eq!(addr.state, "IL");
    }

    #[test]
    fn rejects_missing_field() {
        assert_eq!(parse_line("12 Elm St, Springfield"), Err(ParseError::Format));
    }

    #[test]
    fn rejects_unknown_state() {
        match parse_line("12 Elm St, Springfield, ZZ 62704") {
            Err(ParseError::UnknownState(s)) => assert_eq!(s, "ZZ"),
            other => panic!("expected UnknownState, got {other:?}"),
        }
    }

    #[test]
    fn rejects_bad_zip() {
        match parse_line("12 Elm St, Springfield, IL 6270") {
            Err(ParseError::InvalidZip(z)) => assert_eq!(z, "6270"),
            other => panic!("expected InvalidZip, got {other:?}"),
        }
    }

    #[test]
    fn display_round_trips_normalized_form() {
        let addr = parse_line("12 elm st, springfield, il 62704").unwrap();
        assert_eq!(addr.to_string(), "12 elm st, springfield, IL 62704");
    }
}
