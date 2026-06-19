use std::fmt::Display;
use std::str::FromStr;
use serde::{Deserialize, Deserializer};

pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + Deserialize<'de>,
    <T as FromStr>::Err: Display
{
    match NumberString::<T>::deserialize(deserializer)? {
        NumberString::String(str) => {
            str.parse::<T>().map_err(serde::de::Error::custom)
        },
        NumberString::Number(num) => Ok(num),
    }
}

pub fn deserialize_opt<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + Deserialize<'de>,
    <T as FromStr>::Err: Display
{
    match Option::<NumberString<T>>::deserialize(deserializer)? {
        Some(NumberString::String(str)) => {
            Ok(Some(str.parse::<T>().map_err(serde::de::Error::custom)?))
        },
        Some(NumberString::Number(num)) => Ok(Some(num)),
        None => Ok(None),
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum NumberString<T> {
    String(String),
    Number(T),
}