use std::fmt::Debug;
use std::str::FromStr;
use yaml_rust::Yaml;

pub fn parse_str_into<T>(key: &Yaml, val: &Yaml) -> Result<T, String>
where
    T: FromStr<Err = String>,
{
    match val {
        Yaml::String(s) => T::from_str(s),
        _ => Err(format!("Value for '{:?}' is not str: {:?}", key, val)),
    }
}

pub fn parse_int<T>(key: &Yaml, val: &Yaml) -> Result<T, String>
where
    T: TryFrom<i64>,
    <T as TryFrom<i64>>::Error: Debug,
{
    match val {
        Yaml::Integer(i) => T::try_from(*i).map_err(|e| format!("{:?}", e)),
        _ => Err(format!(
            "Value for '{:?}' is not integer or doesn't fit: {:?}",
            key, val
        )),
    }
}

pub fn parse_bool(key: &Yaml, val: &Yaml) -> Result<bool, String> {
    match val {
        Yaml::Boolean(b) => Ok(*b),
        _ => Err(format!("Value for '{:?}' is not boolean: {:?}", key, val)),
    }
}
