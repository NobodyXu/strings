use core::fmt;
use core::str;

/// Box of two strings.
/// Store two strings efficiently in an immutable way.
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct TwoStrs(Box<[u8]>);

impl From<(&str, &str)> for TwoStrs {
    fn from((s1, s2): (&str, &str)) -> Self {
        Self::new(s1, s2)
    }
}

impl TwoStrs {
    /// * `s1` - must not contain null byte.
    /// * `s2` - must not contain null byte.
    pub fn new(s1: &str, s2: &str) -> Self {
        let mut bytes = Vec::with_capacity(s1.len() + 1 + s2.len());
        bytes.extend_from_slice(s1.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(s2.as_bytes());

        Self(bytes.into_boxed_slice())
    }

    pub fn get(&self) -> (&str, &str) {
        let pos = self.0.iter().position(|byte| *byte == 0).unwrap();

        (
            unsafe { str::from_utf8_unchecked(&self.0[..pos]) },
            unsafe { str::from_utf8_unchecked(&self.0[pos + 1..]) },
        )
    }
}

impl fmt::Display for TwoStrs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let (s1, s2) = self.get();
        write!(f, "({}, {})", s1, s2)
    }
}

#[cfg(test)]
mod tests {
    use super::TwoStrs;

    fn assert(s1: &str, s2: &str) {
        let two_strs = TwoStrs::new(s1, s2);
        assert_eq!(two_strs.get(), (s1, s2));
    }

    #[test]
    fn test() {
        assert("", "");
        assert("12", "");
        assert("", "12");
        assert("12", "12");
        assert("12", "2333");
        assert("acdbd3", "2333");
    }
}
