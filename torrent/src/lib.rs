use serde::{
    de::{self, Deserializer, Unexpected},
    Deserialize,
};

pub mod meta_info;
pub mod tracker;

pub fn bool_from_int<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    match u8::deserialize(deserializer)? {
        0 => Ok(false),
        1 => Ok(true),
        other => Err(de::Error::invalid_value(
            Unexpected::Unsigned(other as u64),
            &"zero or one",
        )),
    }
}

pub fn bool_from_optional_int<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<u8>::deserialize(deserializer);
    // TODO: is there a better way to handle this?

    match value {
        Ok(value) => match value {
            Some(0) => Ok(Some(false)),
            Some(1) => Ok(Some(true)),
            Some(other) => Err(de::Error::invalid_value(
                Unexpected::Unsigned(other as u64),
                &"zero or one",
            )),
            None => Ok(None), // If the field is missing, return `None`
        },
        Err(err) => {
            todo!()
        }
    }
}
