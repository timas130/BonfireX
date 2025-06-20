use async_graphql::{InputValueResult, Scalar, ScalarType, Value, from_value, to_value};
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::any::{Any, TypeId};

#[repr(transparent)]
pub struct Blob<T>(pub T);

#[Scalar]
impl<T> ScalarType for Blob<T>
where
    T: AsRef<[u8]> + FromIterator<u8> + Any + Send + Sync,
{
    fn parse(value: Value) -> InputValueResult<Self> {
        Ok(from_value(value)?)
    }

    fn to_value(&self) -> Value {
        to_value(self).unwrap_or_else(|_| Value::Null)
    }
}

impl<T> Serialize for Blob<T>
where
    T: AsRef<[u8]>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&BASE64_STANDARD.encode(&self.0))
    }
}

impl<'de, T> Deserialize<'de> for Blob<T>
where
    T: FromIterator<u8> + Any,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        BASE64_STANDARD
            .decode(String::deserialize(deserializer)?)
            .map(|arr| {
                if TypeId::of::<T>() == TypeId::of::<Vec<u8>>() {
                    *(Box::new(arr) as Box<dyn Any>).downcast().unwrap()
                } else {
                    arr.into_iter().collect()
                }
            })
            .map(Blob)
            .map_err(serde::de::Error::custom)
    }
}
