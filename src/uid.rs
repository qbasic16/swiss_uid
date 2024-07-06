use std::{error::Error, fmt, str::FromStr};

use itertools::Itertools;
use rand::Rng;

use crate::utils::{ToQuadNibble, ToSplitQuadNibble};

/// Calculates the check digit for the given 8 normal digits of the UID.
pub fn calculate_checkdigit(
    main_digits: &[u8; SwissUid::NUM_CHARS_DIGITS],
) -> Result<u8, UidError> {
    // Factors as defined in the specification
    // See: http://www.ech.ch/de/ech/ech-0097/5.2 (section 2.4.2)
    let multipliers: [u8; SwissUid::NUM_CHARS_DIGITS] = [5, 4, 3, 2, 7, 6, 5, 4];
    let checksum = multipliers
        .iter()
        .zip_eq(main_digits.iter())
        .map(|v| (v.0 * v.1) as u32)
        .sum::<u32>();
    match 11 - (checksum % 11) {
        11 => Ok(0u8),
        10 => Err(UidError::InvalidCheckDigit(10.to_string())),
        n => Ok(n as u8),
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
/// let uid = SwissUid::new("CHE-109.322.551");
/// assert!(uid.is_ok());
/// let uid = uid.unwrap();
/// assert_eq!(format!("{:?}", uid), "CHE-109.322.55[1]".to_owned());
/// assert_eq!(format!("{}", uid), "CHE-109.322.551".to_owned());
/// assert_eq!(uid.to_string_mwst(), "CHE-109.322.551 MWST".to_owned());
/// assert_eq!(uid.to_string_hr(), "CHE-109.322.551 HR".to_owned());
///
/// let uid: SwissUid = "CHE-109.322.551".parse().unwrap();
/// assert!(uid.is_ok());
/// ```
#[derive(Clone, Copy)]
pub struct SwissUid {
    a: u16,
    b: u16,
    p: u16,
    pfx: UidPrefix,
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
    /// let uid = SwissUid::new("CHE-109.322.551");
    /// assert!(uid.is_ok());
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
    /// assert!(uid.is_ok());
    /// ```
    pub fn rand() -> Result<Self, Box<dyn Error>> {
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
            a: (&n[0..4]).to_quad_nibble(),
            b: (&n[4..8]).to_quad_nibble(),
            p: p as u16,
        })
    }

    #[deprecated(
        since = "1.1.0",
        note = "Use `swiss_uid::uid::calculate_checkdigit()` instead"
    )]
    pub fn checkdigit(&self) -> Result<u8, UidError> {
        calculate_checkdigit(&(self.a, self.b).to_split_quad_nibble())
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

        // Get the 8 digits
        let mut n = [0u8; Self::NUM_CHARS_DIGITS];
        n.copy_from_slice(&digits[..Self::NUM_CHARS_DIGITS]);

        // Get the check digit and calculate its counterpart from the first 8 digits
        let p = digits[Self::NUM_CHARS_DIGITS];
        let p_calculated = calculate_checkdigit(&n)?;
        if p_calculated == p {
            Ok(Self {
                pfx,
                a: (&n[0..4]).to_quad_nibble(),
                b: (&n[4..8]).to_quad_nibble(),
                p: p as u16,
            })
        } else {
            Err(UidError::MismatchedCheckDigit(format!(
                "Calculated check digit is [{}]",
                p_calculated
            )))
        }
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
            "{}-{:x}.{:x}{:x}.{:x}[{}]",
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
            "{}-{:x}.{:x}{:x}.{:x}{}",
            self.pfx, a012, a3, b01, b23, self.p
        )
    }
}

impl PartialEq for SwissUid {
    fn eq(&self, other: &Self) -> bool {
        self.pfx == other.pfx && self.a == other.a && self.b == other.b && self.p == other.p
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
        assert_eq!(uid.checkdigit(), Ok(1));
    }

    #[test]
    fn test_valid_uid_adm() {
        let uid = SwissUid::new("CHE-109.322.551");
        assert_eq!(uid.is_ok(), true);
        let uid = uid.unwrap();
        assert_eq!(uid.to_string(), "CHE-109.322.551");
        assert_eq!(uid.checkdigit(), Ok(1));
    }

    #[test]
    fn test_invalid_format() {
        let uid = SwissUid::new("CH-109.322.552");
        assert_eq!(uid.is_err(), true);
        let uid = uid.unwrap_err();
        assert_eq!(
            format!("{}", uid),
            "Invalid format: Prefix must be 'CHE' or 'ADM'"
        );
    }

    #[test]
    fn test_invalid_prefix() {
        let uid = SwissUid::new("ABC-109.322.551");
        assert_eq!(uid.is_err(), true);
        let uid = uid.unwrap_err();
        assert_eq!(
            format!("{}", uid),
            "Invalid format: Prefix must be 'CHE' or 'ADM'"
        );
    }

    #[test]
    fn test_invalid_checkdigit() {
        let uid = SwissUid::new("CHE-000.002.000");
        assert_eq!(uid.is_err(), true);
        let uid = uid.unwrap_err();
        assert_eq!(format!("{:?}", uid), "InvalidCheckDigit(\"10\")");
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
