use core::iter::{IntoIterator, Iterator};
use core::str;

use thin_vec::ThinVec;

/// Store any string efficiently in an immutable way.
///
/// Doesn't have any limit on length, however `StringsNoIndex` only provides
/// `StringsNoIndexIter` and does not provide arbitary indexing.
#[derive(Debug, Default, Eq, PartialEq, Clone, Hash)]
pub struct StringsNoIndex {
    strs: ThinVec<u8>,
}

impl StringsNoIndex {
    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, s: &str) {
        self.strs.extend_from_slice(s.as_bytes());
        self.strs.push(0);
    }

    /// Accumulate length of all strings.
    #[inline(always)]
    pub fn strs_len(&self) -> usize {
        self.strs.len()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.strs_len() == 0
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
        StringsNoIndexIter::new(&self.strs)
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
pub struct StringsNoIndexIter<'a>(&'a [u8]);

impl<'a> StringsNoIndexIter<'a> {
    fn new(slices: &'a [u8]) -> Self {
        Self(slices)
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

        for input_str in input_strs.iter() {
            eprintln!("pushing {}", input_str);

            strs.push(input_str);

            eprintln!("{:#?}", strs);

            assert_strs_in(&strs, &input_strs);
        }

        assert!(!strs.is_empty());

        assert!(input_strs.iter().eq(strs.iter()));
    }
}
