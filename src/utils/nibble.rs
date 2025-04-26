use ::std::ops::{BitAnd, BitOr, Shl, Shr};

pub trait FromNibbles:
    Shl<usize, Output = Self> + Default + BitOr<Output = Self> + From<u8>
{
    fn from_nibbles(digits: &[u8]) -> Self {
        digits
            .iter()
            .take(size_of::<Self>() * 2)
            .fold(Self::default(), |acc, &d| (acc << 4) | d.into())
    }
}

impl FromNibbles for u16 {}
impl FromNibbles for u32 {}

pub trait IntoNibblesNum<T>
where
    T: FromNibbles,
{
    fn into_nibbles_num(&self) -> T;
}

impl<T> IntoNibblesNum<T> for [u8]
where
    T: FromNibbles,
{
    fn into_nibbles_num(&self) -> T {
        T::from_nibbles(self)
    }
}

pub trait IntoNibbles:
    FromNibbles + Shr<usize, Output = Self> + BitAnd<Output = Self> + Clone
{
    /// Returns an iterator over the nibbles (4-bit digits) of the number.
    /// The iterator starts with the most significant nibble.
    #[inline(always)]
    fn into_iter_nibbles(self) -> impl Iterator<Item = u8> {
        let n = self;
        (0..(size_of::<Self>() * 2))
            .into_iter()
            .rev()
            .map(move |i| Self::into_u8(n.clone() >> (i * 4)) & 0x0f)
    }

    fn into_u8(v: Self) -> u8;
}

impl IntoNibbles for u16 {
    #[inline(always)]
    fn into_u8(v: Self) -> u8 {
        v as u8
    }
}
impl IntoNibbles for u32 {
    #[inline(always)]
    fn into_u8(v: Self) -> u8 {
        v as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_nibbles() {
        let n: u16 = 0x1234;
        let n_split: Vec<u8> = n.into_iter_nibbles().collect();
        assert_eq!(n_split, [1, 2, 3, 4]);
    }

    #[test]
    fn test_from_nibbles() {
        let n = [1u8, 2u8, 3u8, 4u8];
        let n_quad: u16 = n.into_nibbles_num();
        assert_eq!(n_quad, 0x1234);
    }

    #[test]
    fn test_to_quad_nibble_above_10() {
        let n = [11u8, 12u8, 13u8, 14u8];
        let n_quad: u16 = n.into_nibbles_num();
        assert_eq!(n_quad, 0xbcde);
        assert_eq!(format!("{n_quad:#x}"), format!("{:#x}", 0xbcde));
    }
}
