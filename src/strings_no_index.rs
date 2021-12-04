use core::iter::{IntoIterator, Iterator};
use core::mem;
use core::str;

use thin_vec::ThinVec;

/// Store any string efficiently in an immutable way.
///
/// Can store at most `u32::MAX` strings and only provides
/// `StringsNoIndexIter` and does not provide arbitary indexing.
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct StringsNoIndex {
    strs: ThinVec<u8>,
}

impl Default for StringsNoIndex {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl StringsNoIndex {
    pub fn new() -> Self {
        let mut strs = ThinVec::with_capacity(mem::size_of::<u32>());
        let len: u32 = 0;
        strs.extend_from_slice(&len.to_ne_bytes());

        Self { strs }
    }

    fn set_len(&mut self, new_len: u32) {
        self.strs[..4].copy_from_slice(&new_len.to_ne_bytes());
    }

    pub fn len(&self) -> u32 {
        u32::from_ne_bytes(self.strs[..4].try_into().unwrap())
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn push(&mut self, s: &str) {
        self.strs.extend_from_slice(s.as_bytes());
        self.strs.push(0);

        self.set_len(self.len() + 1);
    }

    /// Accumulate length of all strings.
    #[inline(always)]
    pub fn strs_len(&self) -> usize {
        self.strs.len()
    }

    #[inline(always)]
    pub fn reserve_strs(&mut self, cnt: usize) {
        self.strs.reserve(cnt);
    }

    pub fn shrink_to_fit(&mut self) {
        self.strs.shrink_to_fit();
    }

    #[inline(always)]
    pub fn iter(&self) -> StringsNoIndexIter<'_> {
        StringsNoIndexIter::new(&self.strs[4..], self.len())
    }
}
impl<'a> IntoIterator for &'a StringsNoIndex {
    type Item = &'a str;
    type IntoIter = StringsNoIndexIter<'a>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Clone, Debug)]
pub struct StringsNoIndexIter<'a>(&'a [u8], u32);

impl<'a> StringsNoIndexIter<'a> {
    fn new(strs: &'a [u8], len: u32) -> Self {
        Self(strs, len)
    }
}

impl<'a> Iterator for StringsNoIndexIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.is_empty() {
            return None;
        }

        let pos = self.0.iter().position(|byte| *byte == 0).unwrap();
        let slice = &self.0[..pos];
        self.0 = &self.0[(pos + 1)..];
        Some(unsafe { str::from_utf8_unchecked(slice) })
    }
}

#[cfg(test)]
mod tests {
    use super::StringsNoIndex;

    fn assert_strs_in(strs: &StringsNoIndex, input_strs: &Vec<String>) {
        for (string, input_str) in strs.iter().zip(input_strs) {
            assert_eq!(string, input_str);
        }
    }

    #[test]
    fn test() {
        let mut strs = StringsNoIndex::new();
        let input_strs: Vec<String> = (0..1024).map(|n| n.to_string()).collect();

        assert!(strs.is_empty());

        for (i, input_str) in input_strs.iter().enumerate() {
            strs.push(input_str);
            assert_eq!(strs.len() as usize, i + 1);

            assert_strs_in(&strs, &input_strs);
        }

        assert!(!strs.is_empty());

        assert!(input_strs.iter().eq(strs.iter()));
    }
}