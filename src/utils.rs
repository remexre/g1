//! Utilities.

pub mod string {
    //! Serde support via `Display`/`FromStr`.
    //!
    //! ## Example
    //!
    //! ```
    //! use std::net::Ipv4Address;
    //! let s = Ipv4Address::from([127, 0, 0, 1]);
    //! ```

    use serde::{de::Error, Deserialize, Deserializer, Serializer};
    use std::{fmt::Display, str::FromStr};

    /// Serializes a value as a string with its `Display` impl.
    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer,
    {
        serializer.collect_str(value)
    }

    /// Deserializes a value as a string with its `FromStr` impl
    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: FromStr,
        T::Err: Display,
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(Error::custom)
    }
}
