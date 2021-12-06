use super::{Strings, StringsIter, StringsNoIndex, StringsNoIndexIter, TwoStrs};

use core::fmt;

use serde::de::{Deserialize, Deserializer, Error, SeqAccess, Visitor};
use serde::ser::{Serialize, SerializeTuple, Serializer};

macro_rules! impl_ser_de_for_strings {
    ($Strings:ident) => {
        /// The format is as follows:
        ///  - u32,
        ///  - &str,
        ///  - ...
        impl Serialize for $Strings {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                let mut tuple_serializer = serializer.serialize_tuple(1 + self.len() as usize)?;

                let len: u32 = self.len().try_into().unwrap();

                tuple_serializer.serialize_element(&len)?;
                for string in self {
                    tuple_serializer.serialize_element(string)?;
                }

                tuple_serializer.end()
            }
        }

        impl<'de> Deserialize<'de> for $Strings {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                struct StringsVisitor;

                impl<'de> Visitor<'de> for StringsVisitor {
                    type Value = $Strings;

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

                        let mut strings = $Strings::with_capacity(len);

                        for i in 0..len {
                            strings.push(
                                seq.next_element()?.ok_or_else(|| {
                                    Error::invalid_length((i + 1) as usize, &self)
                                })?,
                            );
                        }

                        strings.shrink_to_fit();

                        Ok(strings)
                    }
                }

                deserializer.deserialize_tuple(2, StringsVisitor)
            }
        }
    };
}

impl_ser_de_for_strings!(Strings);
impl_ser_de_for_strings!(StringsNoIndex);

macro_rules! impl_Serialize_for_iter {
    ($Iter:ident) => {
        /// The format is as follows:
        ///  - &str,
        ///  - ...
        impl Serialize for $Iter<'_> {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                let mut tuple_serializer = serializer.serialize_tuple(self.size_hint().0)?;

                for string in self.clone() {
                    tuple_serializer.serialize_element(string)?;
                }

                tuple_serializer.end()
            }
        }
    };
}

impl_Serialize_for_iter!(StringsIter);
impl_Serialize_for_iter!(StringsNoIndexIter);

/// Format: (&str, &str)
impl Serialize for TwoStrs {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.get().serialize(serializer)
    }
}

/// Format: (&str, &str)
impl<'de> Deserialize<'de> for TwoStrs {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let (s1, s2) = <(&'de str, &'de str)>::deserialize(deserializer)?;
        Ok(Self::new(s1, s2))
    }
}

#[cfg(test)]
mod tests {
    use super::{Strings, StringsNoIndex, TwoStrs};

    use once_cell::sync::OnceCell;
    use serde_test::{assert_ser_tokens, assert_tokens, Token};

    // Test using serde_test

    #[test]
    fn test_ser_de_empty_serde_strings() {
        assert_tokens(
            &Strings::new(),
            &[Token::Tuple { len: 1 }, Token::U32(0), Token::TupleEnd],
        );
    }

    #[test]
    fn test_ser_de_empty_serde_strings_no_index() {
        assert_tokens(
            &StringsNoIndex::new(),
            &[Token::Tuple { len: 1 }, Token::U32(0), Token::TupleEnd],
        );
    }

    macro_rules! assert_ser_de_serde {
        ($strings:expr) => {
            let strings = $strings;

            // Test Strings
            let mut tokens = vec![
                Token::Tuple {
                    len: 1 + strings.len() as usize,
                },
                Token::U32(strings.len().try_into().unwrap()),
            ];

            for string in strings {
                tokens.push(Token::BorrowedStr(string));
            }

            tokens.push(Token::TupleEnd);

            assert_tokens(strings, &tokens);

            // Test StringsIter
            tokens[0] = Token::Tuple {
                len: strings.len() as usize,
            };
            tokens.remove(1);

            assert_ser_tokens(&strings.iter(), &tokens);
        };
    }

    fn get_strings() -> &'static Strings {
        static STRINGS: OnceCell<Strings> = OnceCell::new();

        STRINGS.get_or_init(|| {
            let mut strings = Strings::new();
            for i in 0..1024 {
                strings.push(&i.to_string());
            }
            strings
        })
    }

    #[test]
    fn test_ser_de_serde_strings() {
        assert_ser_de_serde!(get_strings());
    }

    fn get_strings_no_index() -> &'static StringsNoIndex {
        static STRINGS: OnceCell<StringsNoIndex> = OnceCell::new();

        STRINGS.get_or_init(|| {
            let mut strings = StringsNoIndex::new();
            for i in 0..1024 {
                strings.push(&i.to_string());
            }
            strings
        })
    }

    #[test]
    fn test_ser_de_serde_strings_no_index() {
        assert_ser_de_serde!(get_strings_no_index());
    }

    // Test using serde_json

    macro_rules! assert_ser_de_json {
        ($strings:expr, $strings_type:ident) => {
            let strings = $strings;
            let json = serde_json::to_string(strings).unwrap();
            assert_eq!(
                serde_json::from_str::<'_, $strings_type>(&json).unwrap(),
                *strings
            );
        };
    }

    #[test]
    fn test_ser_de_serde_json_strings() {
        assert_ser_de_json!(get_strings(), Strings);
    }

    #[test]
    fn test_ser_de_serde_json_strings_no_index() {
        assert_ser_de_json!(get_strings_no_index(), StringsNoIndex);
    }

    #[test]
    fn test_ser_de_two_strs() {
        let s1 = "1234<<";
        let s2 = "234a";

        let two_strs = TwoStrs::new(s1, s2);

        assert_tokens(
            &two_strs,
            &[
                Token::Tuple { len: 2 },
                Token::BorrowedStr(s1),
                Token::BorrowedStr(s2),
                Token::TupleEnd,
            ],
        );
    }
}
