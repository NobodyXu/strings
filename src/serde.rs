use super::{Strings, StringsIter, StringsNoIndex, StringsNoIndexIter};

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

#[cfg(test)]
mod tests {
    use super::Strings;

    use once_cell::sync::OnceCell;
    use serde_test::{assert_ser_tokens, assert_tokens, Token};

    // Test using serde_test

    #[test]
    fn test_ser_de_empty_serde() {
        assert_tokens(
            &Strings::new(),
            &[Token::Tuple { len: 1 }, Token::U32(0), Token::TupleEnd],
        );
    }

    fn assert_ser_de_serde(strings: &'static Strings) {
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
    fn test_ser_de_serde() {
        assert_ser_de_serde(get_strings());
    }

    // Test using serde_json

    fn assert_ser_de_json(strings: &Strings) {
        assert_eq!(
            serde_json::from_str::<'_, Strings>(&serde_json::to_string(strings).unwrap()).unwrap(),
            *strings
        );
    }

    #[test]
    fn test_ser_de_serde_json() {
        assert_ser_de_json(get_strings());
    }
}
