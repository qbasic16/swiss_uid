use std::{
    error::Error,
    fmt::{self, write},
    str::FromStr,
};

use itertools::Itertools;
use rand::Fill;
use regex::Regex;

/// Calculates the check digit for the given 8 normal digits of the UID.
pub fn calculate_checkdigit(main_digits: &[u8; 8]) -> Result<u32, UidError> {
    // Factors as defined in the specification
    // See: http://www.ech.ch/de/ech/ech-0097/5.2 (section 2.4.2)
    let multipliers: [u8; 8] = [5, 4, 3, 2, 7, 6, 5, 4];
    let checksum = multipliers
        .iter()
        .zip_eq(main_digits.iter())
        .map(|v| (v.0 * v.1) as u32)
        .sum::<u32>();
    match 11 - (checksum % 11) {
        11 => Ok(0),
        10 => Err(UidError::InvalidCheckDigit(10.to_string())),
        n => Ok(n),
    }
}

/// A Swiss UID (Unternehmens-Identifikationsnummer) is a unique identifier for
/// companies in Switzerland. The rightmost of the 9 digits is the checksum digit.
#[derive(Clone, Copy)]
pub struct SwissUid {
    pfx: UidPrefix,
    n: [u8; 8],
    p: u32,
}

impl SwissUid {
    /// Creates a new SwissUID instance from a string in a valid format.
    ///
    /// The string can be in any of the following formats:
    ///
    /// - `CHE-109.322.551`
    /// - `CHE109322551`
    /// - `CHE109322551`
    /// - `CHE-109.322.551 MWST` (the suffix is ignored)
    pub fn new(uid: &str) -> Result<Self, UidError> {
        let re = Regex::new(
            r"^(?:(?<pfx>[A-Z]{3})[ -]?)?(?<n2>\d{3})[\. ]?(?<n1>\d{3})[\. ]?(?<n0>\d{2})(?<p>\d)",
        );
        if let Err(err) = re {
            panic!("Invalid SwissUID regex: {:?}", err);
        }
        let re = re.unwrap();
        let Some(caps) = re.captures(uid) else {
            return Err(UidError::InvalidFormat(format!("'{}'", uid.to_string())));
        };

        let pfx: UidPrefix = caps.name("pfx").map_or("", |v| v.as_str()).parse()?;

        let mut n: [u8; 8] = [0; 8];
        {
            let digits: Vec<u8> = caps
                .name("n2")
                .map_or("", |v| v.as_str())
                .chars()
                .chain(caps.name("n1").map_or("", |v| v.as_str()).chars())
                .chain(caps.name("n0").map_or("", |v| v.as_str()).chars())
                .map(|c| c.to_digit(10).unwrap() as u8)
                .collect();
            for i in 0..8 {
                n[i] = digits[i];
            }
        }
        let p = caps
            .name("p")
            .map_or(0, |v| v.as_str().parse::<u32>().map_or(0, |n| n));

        let p_calculated = calculate_checkdigit(&n)?;
        let inst = Self { pfx, n, p };
        if p_calculated == p {
            Ok(inst)
        } else {
            Err(UidError::MismatchedCheckDigit(format!(
                "'{:?}' should have the check digit [{}]",
                inst, p_calculated
            )))
        }
    }

    pub fn rand() -> Result<Self, Box<dyn Error>> {
        let mut rng = rand::thread_rng();
        let mut n: [u8; 8] = [0; 8];
        n.try_fill(&mut rng)?;
        let p = calculate_checkdigit(&n).or_else(|_| {
            if n[7] == 0 {
                n[7] += 1;
            } else {
                n[7] -= 1;
            }
            calculate_checkdigit(&n)
        })?;
        Ok(Self {
            pfx: UidPrefix::CHE,
            n,
            p,
        })
    }

    #[deprecated(since = "1.1.0", note = "Use `calculate_checkdigit()` instead")]
    pub fn checkdigit(&self) -> Result<u32, UidError> {
        calculate_checkdigit(&self.n)
    }

    pub fn to_string_mwst(&self) -> String {
        format!("{} MWST", self)
    }

    pub fn to_string_hr(&self) -> String {
        format!("{} HR", self)
    }
}

impl fmt::Debug for SwissUid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}-{}{}{}.{}{}{}.{}{}[{}]",
            self.pfx,
            self.n[0],
            self.n[1],
            self.n[2],
            self.n[3],
            self.n[4],
            self.n[5],
            self.n[6],
            self.n[7],
            self.p
        )
    }
}

impl fmt::Display for SwissUid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}-{}{}{}.{}{}{}.{}{}{}",
            self.pfx,
            self.n[0],
            self.n[1],
            self.n[2],
            self.n[3],
            self.n[4],
            self.n[5],
            self.n[6],
            self.n[7],
            self.p
        )
    }
}

impl PartialEq for SwissUid {
    fn eq(&self, other: &Self) -> bool {
        self.pfx == other.pfx && self.n == other.n && self.p == other.p
    }
}

impl Eq for SwissUid {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UidPrefix {
    CHE,
    ADM,
}

impl FromStr for UidPrefix {
    type Err = UidError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "CHE" => Ok(UidPrefix::CHE),
            "ADM" => Ok(UidPrefix::ADM),
            _ => Err(UidError::InvalidFormat(format!(
                "'{}' prefix must be 'CHE' or 'ADM'",
                s
            ))),
        }
    }
}

impl fmt::Display for UidPrefix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UidError {
    /// Malformed Swiss UID string format
    InvalidFormat(String),
    /// The calculated check digit is in the invalid range, no UID can have this check digit
    InvalidCheckDigit(String),
    /// The calculated check digit of the first 8 digits does not match the given 9th digit (right)
    MismatchedCheckDigit(String),
}

impl Error for UidError {}

impl fmt::Display for UidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UidError::InvalidFormat(s) => write!(f, "Invalid format: {}", s),
            UidError::InvalidCheckDigit(s) => write!(f, "Invalid check digit: {}", s),
            UidError::MismatchedCheckDigit(s) => write!(f, "Mismatched check digit: {}", s),
        }
    }
}
