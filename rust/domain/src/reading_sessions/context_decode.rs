use serde::{
    Deserializer,
    de::{Error, Visitor},
};
use std::fmt;

pub(super) fn name<'de, D: Deserializer<'de>>(input: D) -> Result<Option<String>, D::Error> {
    text::<D, 128>(input)
}
pub(super) fn panel<'de, D: Deserializer<'de>>(input: D) -> Result<Option<String>, D::Error> {
    text::<D, 512>(input)
}
fn text<'de, D: Deserializer<'de>, const LIMIT: usize>(
    input: D,
) -> Result<Option<String>, D::Error> {
    struct Optional<const N: usize>;
    struct Bounded<const N: usize>;
    impl<'de, const N: usize> Visitor<'de> for Optional<N> {
        type Value = Option<String>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("a bounded context string or null")
        }
        fn visit_none<E: Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_unit<E: Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_some<D: Deserializer<'de>>(self, input: D) -> Result<Self::Value, D::Error> {
            input.deserialize_str(Bounded::<N>).map(Some)
        }
    }
    impl<const N: usize> Visitor<'_> for Bounded<N> {
        type Value = String;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "a context string of at most {N} bytes")
        }
        fn visit_str<E: Error>(self, value: &str) -> Result<Self::Value, E> {
            if value.len() > N {
                Err(E::custom(
                    "Stored reading context string exceeds its supported bound",
                ))
            } else {
                Ok(value.to_owned())
            }
        }
    }
    input.deserialize_option(Optional::<LIMIT>)
}
