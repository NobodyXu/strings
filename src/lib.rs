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

/// The format of `Strings` is as follows:
///  - len: u32,
///  - data: &[str],
#[cfg(feature = "serde")]
impl serde::Serialize for Strings {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;

        let mut struct_serializer = serializer.serialize_struct("Strings", 2)?;

        let len: u32 = self.len().try_into().unwrap();

        struct_serializer.serialize_field("len", &len)?;
        struct_serializer.serialize_field("data", &self.iter())?;

        struct_serializer.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Strings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use core::fmt;
        use serde::de::{Error, SeqAccess, Visitor};

        struct StringsVisitor;

        impl<'de> Visitor<'de> for StringsVisitor {
            type Value = Strings;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "A u32 length and &[str]")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let len: u32 = seq
                    .next_element()?
                    .ok_or_else(|| Error::invalid_length(0, &self))?;

                let mut strings = Strings::new();
                strings.reserve(len as usize);

                for i in 0..len {
                    strings.push(
                        seq.next_element()?
                            .ok_or_else(|| Error::invalid_length((i + 1) as usize, &self))?,
                    );
                }

                Ok(strings)
            }
        }

        deserializer.deserialize_struct("Strings", &["len", "data"], StringsVisitor)
    }
}

#[derive(Clone)]
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

/// The format of `StringsIter` is as follows:
///  - &[str],
#[cfg(feature = "serde")]
impl serde::Serialize for StringsIter<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;

        let mut seq_serializer = serializer.serialize_seq(None)?;
        for each in self.clone() {
            seq_serializer.serialize_element(each)?;
        }

        seq_serializer.end()
    }
}

#[cfg(test)]
mod tests {
    use super::Strings;

    fn assert_strs_in(strs: &Strings, input_strs: &Vec<String>) {
        for (string, input_str) in strs.iter().zip(input_strs) {
            assert_eq!(string, input_str);
        }
    }

    #[test]
    fn test() {
        let mut strs = Strings::new();
        let input_strs: Vec<String> = (0..1024).map(|n| n.to_string()).collect();

        assert!(strs.is_empty());

        for (i, input_str) in input_strs.iter().enumerate() {
            strs.push(input_str);
            assert_eq!(strs.len(), i + 1);

            assert_strs_in(&strs, &input_strs);
        }

        assert!(input_strs.iter().eq(strs.iter()));

        for (i, input_str) in input_strs.iter().enumerate() {
            assert_eq!(strs.get(i.try_into().unwrap()).unwrap(), input_str);
        }

        let input_str = input_strs.concat();

        assert_eq!(strs.as_str(), input_str);
        assert_eq!(strs.into_str(), input_str);
    }
}
