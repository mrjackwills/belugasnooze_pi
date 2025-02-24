use serde::{Deserialize, Deserializer, de};
use std::ops::RangeInclusive;
pub struct IncomingSerializer;

impl IncomingSerializer {
    /// Check value is in given range
    fn in_range<'de, D>(deserializer: D, range: RangeInclusive<u8>) -> Result<u8, D::Error>
    where
        D: Deserializer<'de>,
    {
        let parsed = u8::deserialize(deserializer)?;
        if !range.contains(&parsed) {
            return Err(de::Error::custom(format!(
                "{parsed}, not in range {range:?}"
            )));
        }
        Ok(parsed)
    }

    /// Allow only vec (json array), max length 7, of items 0 to 6
    pub fn days<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let parsed = Vec::deserialize(deserializer)?;
        let range = 0..=6;

        if parsed.len() > 7 {
            return Err(de::Error::custom("too many days"));
        }
        for i in &parsed {
            if !range.contains(i) {
                return Err(de::Error::custom(format!("{i} not in range {range:?}")));
            }
        }
        Ok(parsed)
    }

    /// Allow only u8s from 0 to 23
    pub fn hour<'de, D>(deserializer: D) -> Result<u8, D::Error>
    where
        D: Deserializer<'de>,
    {
        let range = 0..=23u8;
        Self::in_range(deserializer, range)
    }

    /// Allow only positive i64, due to sql id issues
    pub fn id<'de, D>(deserializer: D) -> Result<i64, D::Error>
    where
        D: Deserializer<'de>,
    {
        let parsed = i64::deserialize(deserializer)?;
        if parsed < 1 {
            return Err(de::Error::custom(format!("{parsed} smaller than 1")));
        }
        Ok(parsed)
    }

    /// Allow only u8s from 0 to 59
    pub fn minute<'de, D>(deserializer: D) -> Result<u8, D::Error>
    where
        D: Deserializer<'de>,
    {
        let range = 0..=59u8;
        Self::in_range(deserializer, range)
    }

    /// Use timezones crate to make sure is valid timezone
    pub fn timezone<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        let parsed = String::deserialize(deserializer)?;
        match jiff::tz::TimeZone::get(&parsed) {
            Ok(_) => Ok(parsed),
            Err(_) => Err(de::Error::custom("unknown timezone")),
        }
    }
}

/// incoming_serializer
///
/// cargo watch -q -c -w src/ -x 'test incoming_serializer -- --test-threads=1 --nocapture'
#[cfg(test)]
#[expect(clippy::unwrap_used)]
mod tests {
    use serde::de::value::{Error as ValueError, StringDeserializer, U8Deserializer};
    use serde::de::{
        IntoDeserializer,
        value::{I64Deserializer, SeqDeserializer},
    };

    use crate::S;

    use super::*;

    #[test]
    fn incoming_serializer_days_err() {
        let deserializer: SeqDeserializer<std::vec::IntoIter<u8>, ValueError> =
            vec![0u8, 0u8, 1u8, 2u8, 3u8, 4u8, 5u8, 6u8].into_deserializer();
        let result = IncomingSerializer::days(deserializer);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "too many days");

        let deserializer: SeqDeserializer<std::vec::IntoIter<u8>, ValueError> =
            vec![1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8].into_deserializer();
        let result = IncomingSerializer::days(deserializer);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "7 not in range 0..=6");
    }

    #[test]
    fn incoming_serializer_days_ok() {
        let deserializer: SeqDeserializer<std::vec::IntoIter<u8>, ValueError> =
            vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8, 6u8].into_deserializer();
        let result = IncomingSerializer::days(deserializer);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), [0, 1, 2, 3, 4, 5, 6,]);
    }

    #[test]
    fn incoming_serializer_id_err() {
        let deserializer: I64Deserializer<ValueError> = 0i64.into_deserializer();
        let result = IncomingSerializer::id(deserializer);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "0 smaller than 1");
    }

    #[test]
    fn incoming_serializer_id_ok() {
        let deserializer: I64Deserializer<ValueError> = 10i64.into_deserializer();
        let result = IncomingSerializer::id(deserializer);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 10i64);
    }

    #[test]
    fn incoming_serializer_minute_err() {
        let deserializer: U8Deserializer<ValueError> = 60u8.into_deserializer();
        let result = IncomingSerializer::minute(deserializer);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "60, not in range 0..=59");
    }

    #[test]
    fn incoming_serializer_minute_ok() {
        let deserializer: U8Deserializer<ValueError> = 30u8.into_deserializer();
        let result = IncomingSerializer::minute(deserializer);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 30u8);
    }

    #[test]
    fn incoming_serializer_hour_err() {
        let deserializer: U8Deserializer<ValueError> = 24u8.into_deserializer();
        let result = IncomingSerializer::hour(deserializer);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "24, not in range 0..=23");
    }

    #[test]
    fn incoming_serializer_hour_ok() {
        let deserializer: U8Deserializer<ValueError> = 23u8.into_deserializer();
        let result = IncomingSerializer::hour(deserializer);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 23u8);
    }

    #[test]
    fn incoming_serializer_timezone_err() {
        let deserializer: StringDeserializer<ValueError> =
            S!("America/NEwYork").into_deserializer();
        let result = IncomingSerializer::timezone(deserializer);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "unknown timezone");
    }

    #[test]
    fn incoming_serializer_timezone_ok() {
        let deserializer: StringDeserializer<ValueError> =
            S!("America/New_York").into_deserializer();
        let result = IncomingSerializer::timezone(deserializer);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "America/New_York");
    }
}
