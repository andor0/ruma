use std::{num::NonZeroU8, str::FromStr};

use crate::{crypto_algorithms::DeviceKeyAlgorithm, Error};

pub fn validate(s: &str) -> Result<NonZeroU8, Error> {
    let colon_idx = NonZeroU8::new(s.find(':').ok_or(Error::MissingKeyDelimiter)? as u8)
        .ok_or(Error::UnknownKeyAlgorithm)?;

    DeviceKeyAlgorithm::from_str(&s[0..colon_idx.get() as usize])
        .map_err(|_| Error::UnknownKeyAlgorithm)?;

    Ok(colon_idx)
}
