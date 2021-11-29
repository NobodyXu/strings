use core::convert::TryInto;
use core::iter::{IntoIterator, Iterator};
use core::slice;
use core::str;

/// Store any string efficiently in an immutable way.
///
/// Can store at most `u32::MAX` strings, the accumulated length
/// of these strings can be at most `u32::MAX`.
#[derive(Debug, Default)]
pub struct Strings {
    strs: Vec<u8>,
    ends: Vec<u32>,
}

impl Strings {
    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, s: &str) {
        self.strs.extend_from_slice(s.as_bytes());
        self.ends.push(
            self.strs
                .len()
                .try_into()
                .expect("Strings cannot contain more than u32::MAX strings"),
        );
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.ends.len()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline(always)]
    pub fn reserve(&mut self, strs_cnt: usize) {
        self.ends.reserve(strs_cnt);
    }

    #[inline(always)]
    pub fn reserve_strs(&mut self, cnt: usize) {
        self.strs.reserve(cnt);
    }

    pub fn shrink_to_fit(&mut self) {
        self.strs.shrink_to_fit();
        self.ends.shrink_to_fit();
    }

    #[inline(always)]
    pub fn iter(&self) -> StringsIter<'_> {
        StringsIter {
            strings: self,
            ends_iter: self.ends.iter(),
            start: 0,
        }
    }

    pub fn get(&self, index: u32) -> Option<&str> {
        let end = *self.ends.get(index as usize)?;
        let start = if index == 0 {
            0
        } else {
            self.ends[(index - 1) as usize]
        };

        Some(self.get_str_impl(start, end))
    }

    #[inline(always)]
    fn get_str_impl(&self, start: u32, end: u32) -> &str {
        unsafe { str::from_utf8_unchecked(&self.strs[(start as usize)..(end as usize)]) }
    }

    pub fn as_str(&self) -> &str {
        self.get_str_impl(
            0,
            self.strs
                .len()
                .try_into()
                .expect("Strings cannot contain more than u32::MAX strings"),
        )
    }

    pub fn into_str(self) -> String {
        unsafe { String::from_utf8_unchecked(self.strs) }
    }
}
impl<'a> IntoIterator for &'a Strings {
    type Item = &'a str;
    type IntoIter = StringsIter<'a>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct StringsIter<'a> {
    strings: &'a Strings,
    ends_iter: slice::Iter<'a, u32>,
    start: u32,
}

impl<'a> Iterator for StringsIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.start;
        let end = *self.ends_iter.next()?;

        self.start = end;

        Some(self.strings.get_str_impl(start, end))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
