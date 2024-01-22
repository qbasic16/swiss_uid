use std::fmt;

use itertools::Itertools;
use regex::Regex;

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
    /// - `CHE-109.322.551 MWST` (the suffix is ignored)0
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

        let pfx: UidPrefix = caps.name("pfx").map_or("", |v| v.as_str()).try_into()?;

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

        let inst = Self { pfx, n, p };
        match inst.checkdigit() {
            Some(c) => {
                if c == p {
                    Ok(inst)
                } else {
                    Err(UidError::MismatchedCheckDigit(format!(
                        "'{:?}' should have the check digit [{}]",
                        inst, c
                    )))
                }
            }
            None => Err(UidError::InvalidCheckDigit(format!(
                "'{:?}' is prohibited from use",
                inst
            ))),
        }
    }

    fn checkdigit(&self) -> Option<u32> {
        // Factors as defined in the specification
        // See: http://www.ech.ch/de/ech/ech-0097/5.2 (section 2.4.2)
        let multipliers: [u8; 8] = [5, 4, 3, 2, 7, 6, 5, 4];
        let checksum = multipliers
            .iter()
            .zip_eq(self.n.iter())
            .map(|v| (v.0 * v.1) as u32)
            .sum::<u32>();
        match 11 - (checksum % 11) {
            11 => Some(0),
            10 => None,
            n => Some(n),
        }
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
impl TryFrom<&str> for UidPrefix {
    type Error = UidError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "CHE" => Ok(UidPrefix::CHE),
            "ADM" => Ok(UidPrefix::ADM),
            _ => Err(UidError::InvalidFormat(format!(
                "'{}' prefix must be 'CHE' or 'ADM'",
                value
            ))),
        }
    }
}
impl fmt::Display for UidPrefix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum UidError {
    /// Malformed Swiss UID string format
    InvalidFormat(String),
    /// The calculated check digit is in the invalid range, no UID can have this check digit
    InvalidCheckDigit(String),
    /// The calculated check digit of the first 8 digits does not match the given 9th digit (right)
    MismatchedCheckDigit(String),
}
impl fmt::Debug for UidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UidError::InvalidFormat(s) => write!(f, "Invalid format: {}", s),
            UidError::InvalidCheckDigit(s) => write!(f, "Invalid check digit: {}", s),
            UidError::MismatchedCheckDigit(s) => write!(f, "Mismatched check digit: {}", s),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_valid_uid_che() {
        let uid = SwissUid::new("CHE-109.322.551");
        assert_eq!(uid.is_ok(), true);
        let uid = uid.unwrap();
        assert_eq!(uid.to_string(), "CHE-109.322.551");
        assert_eq!(uid.to_string_hr(), "CHE-109.322.551 HR");
        assert_eq!(uid.to_string_mwst(), "CHE-109.322.551 MWST");
        assert_eq!(format!("{}", uid), "CHE-109.322.551");
        assert_eq!(format!("{:?}", uid), "CHE-109.322.55[1]");
        assert_eq!(uid.checkdigit(), Some(1));
    }

    #[test]
    fn test_valid_uid_adm() {
        let uid = SwissUid::new("CHE-109.322.551");
        assert_eq!(uid.is_ok(), true);
        let uid = uid.unwrap();
        assert_eq!(uid.to_string(), "CHE-109.322.551");
        assert_eq!(uid.checkdigit(), Some(1));
    }

    #[test]
    fn test_invalid_format() {
        let uid = SwissUid::new("CH-109.322.552");
        assert_eq!(uid.is_err(), true);
        let uid = uid.unwrap_err();
        assert_eq!(format!("{:?}", uid), "Invalid format: 'CH-109.322.552'");
    }

    #[test]
    fn test_invalid_prefix() {
        let uid = SwissUid::new("ABC-109.322.551");
        assert_eq!(uid.is_err(), true);
        let uid = uid.unwrap_err();
        assert_eq!(
            format!("{:?}", uid),
            "Invalid format: 'ABC' prefix must be 'CHE' or 'ADM'"
        );
    }

    #[test]
    fn test_invalid_checkdigit() {
        let uid = SwissUid::new("CHE-000.002.000");
        assert_eq!(uid.is_err(), true);
        let uid = uid.unwrap_err();
        assert_eq!(
            format!("{:?}", uid),
            "Invalid check digit: 'CHE-000.002.00[0]' is prohibited from use"
        );
    }

    #[test]
    fn test_mismatched_checkdigit() {
        let uid = SwissUid::new("CHE-109.322.552");
        assert_eq!(uid.is_err(), true);
        let uid = uid.unwrap_err();
        assert_eq!(
            format!("{:?}", uid),
            "Mismatched check digit: 'CHE-109.322.55[2]' should have the check digit [1]"
        );
    }

    #[test]
    fn test_eq_uid() {
        let uid1 = SwissUid::new("CHE-109.322.551");
        assert_eq!(uid1.is_ok(), true);
        let uid1 = uid1.unwrap();
        let uid2 = SwissUid::new("CHE-109.322.551");
        assert_eq!(uid2.is_ok(), true);
        let uid2 = uid2.unwrap();
        assert_eq!(uid1, uid2);
    }

    #[test]
    fn test_clone_uid() {
        let uid1 = SwissUid::new("CHE-109.322.551");
        assert_eq!(uid1.is_ok(), true);
        let uid1 = uid1.unwrap();
        let uid2 = uid1.clone();
        assert_eq!(uid1, uid2);
    }
}
