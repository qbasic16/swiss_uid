use ::std::{error::Error, fmt, str::FromStr};

use ::itertools::Itertools;

use crate::utils::IntoNibblesNum;

// Factors as defined in the specification
// See: http://www.ech.ch/de/ech/ech-0097/5.2 (section 2.4.2)
const DIGIT_FACTORS: [u8; SwissUid::NUM_CHARS_DIGITS] = [5, 4, 3, 2, 7, 6, 5, 4];

/// Calculates the check digit for the given 8 normal digits of the UID.
#[inline]
pub fn calculate_checkdigit(main_digits: &[u8]) -> Result<u8, UidError> {
    if main_digits.len() != DIGIT_FACTORS.len() {
        Err(UidError::InvalidFormat("UID must have 8 digits".to_owned()))
    } else {
        let checksum: u32 = DIGIT_FACTORS
            .iter()
            .zip_eq(main_digits.iter())
            .map(|v| (v.0 * v.1) as u32)
            .sum();
        match 11 - (checksum % 11) {
            11 => Ok(0u8),
            10 => Err(UidError::InvalidCheckDigit(10.to_string())),
            n => Ok(n as u8),
        }
    }
}

/// A Swiss UID (Unternehmens-Identifikationsnummer) is a unique identifier for
/// companies in Switzerland. The rightmost of the 9 digits is the checksum digit.
///
/// # Example
///
/// ```rust
/// use swiss_uid::uid::SwissUid;
///
/// let uid = SwissUid::new("CHE-109.322.551").unwrap();
/// assert_eq!(format!("{:?}", uid), "CHE-109.322.55[1]".to_owned());
/// assert_eq!(format!("{}", uid), "CHE-109.322.551".to_owned());
/// assert_eq!(uid.to_string_mwst(), "CHE-109.322.551 MWST".to_owned());
/// assert_eq!(uid.to_string_hr(), "CHE-109.322.551 HR".to_owned());
///
/// let uid2: SwissUid = "CHE-109.322.551".parse().unwrap();
/// assert_eq!(uid2.to_string().len(), 15);
/// ```
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SwissUid {
    pub(self) a: u16,
    pub(self) b: u16,
    pub(self) p: u16,
    pub(self) pfx: UidPrefix,
}

impl SwissUid {
    const NUM_CHARS_PFX: usize = 3;
    const NUM_CHARS_DIGITS: usize = 8;

    /// Creates a SwissUID from a string.
    ///
    /// The only requirements for successful parsing are:
    /// - The prefix must be either "CHE" or "ADM"
    /// - The UID must have 8 digits before the check digit
    /// - One check digit
    ///
    /// After the prefix and inbetween and after the digits there can be any characters
    /// which will be ignored. The string "CHE-109.322.551" will be handled as "CHE109322551".
    ///
    /// # Example
    ///
    /// ```rust
    /// use swiss_uid::uid::SwissUid;
    ///
    /// let uid = SwissUid::new("CHE-109.322.551").unwrap();
    /// assert_eq!(format!("{}", uid), "CHE-109.322.551".to_owned());
    /// ```
    pub fn new(uid: &str) -> Result<Self, UidError> {
        uid.parse()
    }

    /// Generates a random valid Swiss UID.
    ///
    /// # Example
    /// ```rust
    /// use swiss_uid::uid::SwissUid;
    ///
    /// let uid = SwissUid::rand().unwrap();
    /// assert_eq!(uid.to_string().len(), 15);
    /// ```
    #[cfg(feature = "rand")]
    pub fn rand() -> Result<Self, UidError> {
        use rand::Rng;

        let mut rng = rand::thread_rng();
        let mut n = [0u8; Self::NUM_CHARS_DIGITS];
        let mut n_iter = n.iter_mut();

        // The first digit must be between 1 and 9
        *n_iter.next().unwrap() = rng.gen_range(1..10);
        // The rest can be between 0 and 9
        for d in n_iter {
            *d = rng.gen_range(0..10);
        }

        let p = calculate_checkdigit(&n).or_else(|_| {
            // The check digit was 10 thus we change one digit and recalculate
            if n[0] <= 1 {
                n[0] += 1;
            } else {
                n[0] -= 1;
            }
            calculate_checkdigit(&n)
        })?;

        Ok(Self {
            pfx: UidPrefix::CHE,
            a: (&n[0..4]).into_nibbles_num(),
            b: (&n[4..8]).into_nibbles_num(),
            p: p as u16,
        })
    }

    pub fn checkdigit(&self) -> u8 {
        self.p as u8
    }

    /// Returns the UID as a string with the suffix " MWST" (Mehrwertsteuer).
    ///
    /// # Example
    ///
    /// ```rust
    /// use swiss_uid::uid::SwissUid;
    ///
    /// let uid = SwissUid::new("CHE-109.322.551").unwrap();
    /// assert_eq!(uid.to_string_mwst(), "CHE-109.322.551 MWST".to_owned());
    /// ```
    pub fn to_string_mwst(&self) -> String {
        format!("{} MWST", self)
    }

    /// Returns the UID as a string with the suffix " HR" (Handelsregister).
    ///
    /// # Example
    ///
    /// ```rust
    /// use swiss_uid::uid::SwissUid;
    ///
    /// let uid = SwissUid::new("CHE-109.322.551").unwrap();
    /// assert_eq!(uid.to_string_hr(), "CHE-109.322.551 HR".to_owned());
    /// ```
    pub fn to_string_hr(&self) -> String {
        format!("{} HR", self)
    }
}

impl FromStr for SwissUid {
    type Err = UidError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pfx: UidPrefix = s[..Self::NUM_CHARS_PFX].parse()?;

        // Parse the 9 ascii digits
        let digits: Vec<u8> = s
            .chars()
            .skip(Self::NUM_CHARS_PFX)
            .filter(|c| c.is_ascii_digit())
            .take(Self::NUM_CHARS_DIGITS + 1)
            .filter_map(|c| c.to_digit(10).map(|d| d as u8))
            .collect();
        if digits.len() != Self::NUM_CHARS_DIGITS + 1 {
            return Err(UidError::InvalidFormat("UID must have 9 digits".to_owned()));
        }
        if digits[0] == 0 {
            return Err(UidError::LeadingZeroNotAllowed);
        }

        // Get the check digit and calculate its counterpart from the first 8 digits
        let p = digits[Self::NUM_CHARS_DIGITS];
        calculate_checkdigit(&digits[..Self::NUM_CHARS_DIGITS]).and_then(|p_calculated| {
            if p_calculated == p {
                Ok(Self {
                    pfx,
                    a: (&digits[0..4]).into_nibbles_num(),
                    b: (&digits[4..8]).into_nibbles_num(),
                    p: p as u16,
                })
            } else {
                Err(UidError::MismatchedCheckDigit(format!(
                    "Calculated check digit is [{}]",
                    p_calculated
                )))
            }
        })
    }
}

impl fmt::Debug for SwissUid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let a012 = self.a >> 4;
        let a3 = self.a & 0x000f;

        let b01 = (self.b & 0xff00) >> 8;
        let b23 = self.b & 0x00ff;

        write!(
            f,
            "{}-{:03x}.{:x}{:02x}.{:02x}[{}]",
            self.pfx, a012, a3, b01, b23, self.p
        )
    }
}

impl fmt::Display for SwissUid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let a012 = self.a >> 4;
        let a3 = self.a & 0x000f;

        let b01 = (self.b & 0xff00) >> 8;
        let b23 = self.b & 0x00ff;

        write!(
            f,
            "{}-{:03x}.{:x}{:02x}.{:02x}{}",
            self.pfx, a012, a3, b01, b23, self.p
        )
    }
}

unsafe impl Send for SwissUid {}
unsafe impl Sync for SwissUid {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UidPrefix {
    CHE,
    ADM,
}

impl FromStr for UidPrefix {
    type Err = UidError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "CHE" => Ok(UidPrefix::CHE),
            "ADM" => Ok(UidPrefix::ADM),
            _ => Err(UidError::InvalidFormat(
                "Prefix must be 'CHE' or 'ADM'".to_owned(),
            )),
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
    /// Leading zero is not allowed in the UID
    LeadingZeroNotAllowed,
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
            UidError::LeadingZeroNotAllowed => write!(f, "Leading zero is not allowed"),
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
        assert_eq!(uid.pfx, UidPrefix::CHE);
        assert_eq!(uid.a, 0x1093);
        assert_eq!(uid.b, 0x2255);
        assert_eq!(uid.p, 1);
        assert_eq!(uid.to_string(), "CHE-109.322.551");
        assert_eq!(uid.to_string_hr(), "CHE-109.322.551 HR");
        assert_eq!(uid.to_string_mwst(), "CHE-109.322.551 MWST");
        assert_eq!(format!("{}", uid), "CHE-109.322.551");
        assert_eq!(format!("{:?}", uid), "CHE-109.322.55[1]");
    }

    #[test]
    fn test_valid_uid_parse() {
        let uid: SwissUid = "CHE-109.322.551".parse().unwrap();
        assert_eq!(uid.pfx, UidPrefix::CHE);
        assert_eq!(uid.a, 0x1093);
        assert_eq!(uid.b, 0x2255);
        assert_eq!(uid.p, 1);
        assert_eq!(uid.to_string(), "CHE-109.322.551");
    }

    #[test]
    fn test_valid_uid_with_zeroes() {
        let uid = SwissUid::new("CHE-100.002.005");
        assert_eq!(uid.is_ok(), true);
        let uid = uid.unwrap();
        assert_eq!(uid.pfx, UidPrefix::CHE);
        assert_eq!(uid.a, 0x1000);
        assert_eq!(uid.b, 0x0200);
        assert_eq!(uid.p, 5);
        assert_eq!(uid.to_string(), "CHE-100.002.005");
    }

    #[cfg(feature = "rand")]
    #[test]
    fn test_valid_uid_rand() {
        let uid = SwissUid::rand();
        assert_eq!(uid.is_ok(), true);
        let uid = uid.unwrap();
        assert_eq!(uid.pfx, UidPrefix::CHE);
        assert_eq!(uid.to_string().len(), 15, "{}", uid);
    }

    #[test]
    fn test_valid_uid_adm() {
        let uid = SwissUid::new("ADM-109.322.551");
        assert_eq!(uid.is_ok(), true);
        let uid = uid.unwrap();
        assert_eq!(uid.to_string(), "ADM-109.322.551");
    }

    #[test]
    fn test_incomplete_prefix() {
        let uid = SwissUid::new("CH-109.322.552");
        assert_eq!(uid.is_err(), true);
        let uid = uid.unwrap_err();
        assert_eq!(
            format!("{}", uid),
            "Invalid format: Prefix must be 'CHE' or 'ADM'"
        );
    }

    #[test]
    fn test_unknown_prefix() {
        let uid = SwissUid::new("ABC-109.322.551");
        assert_eq!(uid.is_err(), true);
        let uid = uid.unwrap_err();
        assert_eq!(
            format!("{}", uid),
            "Invalid format: Prefix must be 'CHE' or 'ADM'"
        );
    }

    #[test]
    fn test_leading_zero_not_allowed() {
        let uid = SwissUid::new("CHE-010.322.557");
        assert_eq!(uid.is_err(), true, "{:?}", uid);
        let uid = uid.unwrap_err();
        assert_eq!(format!("{:?}", uid), "LeadingZeroNotAllowed");
    }

    #[test]
    fn test_invalid_checkdigit() {
        let uid = SwissUid::new("CHE-100.002.000");
        assert_eq!(uid.is_err(), true);
        let uid = uid.unwrap_err();
        assert_eq!(
            format!("{:?}", uid),
            "MismatchedCheckDigit(\"Calculated check digit is [5]\")"
        );
    }

    #[test]
    fn test_mismatched_checkdigit() {
        let uid = SwissUid::new("CHE-109.322.552");
        assert_eq!(uid.is_err(), true);
        let uid = uid.unwrap_err();
        assert_eq!(
            format!("{}", uid),
            "Mismatched check digit: Calculated check digit is [1]"
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
