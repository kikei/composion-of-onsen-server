use serde_json::Value;
use std::convert::TryFrom;

pub fn from_value<'a, T>(value: Option<&'a Value>) -> Result<T, String>
where
    T: TryFrom<&'a Value, Error = String>
{
    let value = value.ok_or("Not found")?;
    T::try_from(value)
}
