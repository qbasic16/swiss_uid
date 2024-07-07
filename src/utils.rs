/// Utility trait for converting between bytes and nibbles
pub(crate) trait ToSplitQuadNibble<T>: Copy {
    fn to_split_quad_nibble(self) -> T;
}

impl ToSplitQuadNibble<[u8; 4]> for u16 {
    /// Convert a u16 number into 4 bytes, nibbles from left (MSB) to right (LSB).
    ///
    /// # Example
    ///
    /// ```ignore
    /// let n: u16 = 0x1234;
    /// let n_split = n.to_split_quad_nibble();
    /// assert_eq!(n_split, [1, 2, 3, 4]);
    /// ```
    #[inline]
    fn to_split_quad_nibble(self) -> [u8; 4] {
        [
            (self >> 12) as u8,
            ((self & 0x0f00) >> 8) as u8,
            ((self & 0x00f0) >> 4) as u8,
            (self & 0x000f) as u8,
        ]
    }
}

impl ToSplitQuadNibble<[u8; 8]> for (u16, u16) {
    /// Convert a tuple of 2 u16 numbers into 8 bytes, nibbles from left (MSB)
    /// to right (LSB) per tuple element.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let n: (u16, u16) = (0x1234, 0x5678);
    /// let n_split = n.to_split_quad_nibble();
    /// assert_eq!(n_split, [1, 2, 3, 4, 5, 6, 7, 8]);
    /// ```
    #[inline]
    fn to_split_quad_nibble(self) -> [u8; 8] {
        let (a, b) = self;
        [
            (a >> 12) as u8,
            ((a & 0x0f00) >> 8) as u8,
            ((a & 0x00f0) >> 4) as u8,
            (a & 0x000f) as u8,
            (b >> 12) as u8,
            ((b & 0x0f00) >> 8) as u8,
            ((b & 0x00f0) >> 4) as u8,
            (b & 0x000f) as u8,
        ]
    }
}

pub(crate) trait ToQuadNibble {
    fn to_quad_nibble(self) -> u16;
}

impl ToQuadNibble for &[u8] {
    /// Convert 4 bytes into a u16 number, nibbles from left (MSB) to right (LSB).
    ///
    /// # Example
    ///
    /// ```ignore
    /// let n: &[u8] = &[11, 12, 13, 14];
    /// let n_quad = n.to_quad_nibble();
    /// assert_eq!(n_quad, 0xbcde);
    #[inline]
    fn to_quad_nibble(self) -> u16 {
        if self.len() < 4 {
            return 0;
        }
        (((self[0] & 0x0f) as u16) << 12)
            | (((self[1] & 0x0f) as u16) << 8)
            | (((self[2] & 0x0f) as u16) << 4)
            | ((self[3] & 0x0f) as u16)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_split_quad_nibble() {
        let n: u16 = 0x1234;
        let n_split = n.to_split_quad_nibble();
        assert_eq!(n_split, [1, 2, 3, 4]);
    }

    #[test]
    fn test_to_quad_nibble() {
        let n = [1, 2, 3, 4];
        let n_quad = n.to_quad_nibble();
        assert_eq!(n_quad, 0x1234);
    }

    #[test]
    fn test_to_quad_nibble_above_10() {
        let n = [11, 12, 13, 14];
        let n_quad = n.to_quad_nibble();
        assert_eq!(n_quad, 0xbcde);
        assert_eq!(format!("{n_quad:#x}"), format!("{:#x}", 0xbcde));
    }
}
