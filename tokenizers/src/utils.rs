use crate::tokenizer::{Encoding, PaddingDirection, Result};
use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct TruncationParams {
    pub max_length: usize,
    pub strategy: TruncationStrategy,
    pub stride: usize,
}

#[derive(Debug, Clone)]
pub struct PaddingParams {
    pub strategy: PaddingStrategy,
    pub direction: PaddingDirection,
    pub pad_id: u32,
    pub pad_type_id: u32,
    pub pad_token: String,
}

#[derive(Debug, Clone)]
pub enum PaddingStrategy {
    BatchLongest,
    Fixed(usize),
}

#[derive(Debug)]
pub enum Error {
    SecondSequenceNotProvided,
    SequenceTooShort,
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::SecondSequenceNotProvided => {
                write!(fmt, "Truncation error: Second sequence not provided")
            }
            Error::SequenceTooShort => write!(
                fmt,
                "Truncation error: Sequence to truncate too short to respect the provided max_length"
            ),
        }
    }
}
impl std::error::Error for Error {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TruncationStrategy {
    LongestFirst,
    OnlyFirst,
    OnlySecond,
}

impl std::convert::AsRef<str> for TruncationStrategy {
    fn as_ref(&self) -> &str {
        match self {
            TruncationStrategy::LongestFirst => "longest_first",
            TruncationStrategy::OnlyFirst => "only_first",
            TruncationStrategy::OnlySecond => "only_second",
        }
    }
}

pub fn truncate_encodings(
    mut encoding: Encoding,
    mut pair_encoding: Option<Encoding>,
    params: &TruncationParams,
) -> Result<(Encoding, Option<Encoding>)> {
    if params.max_length == 0 {
        return Ok((encoding, pair_encoding));
    }

    let total_length = encoding.get_ids().len()
        + pair_encoding
            .as_ref()
            .map(|e| e.get_ids().len())
            .unwrap_or(0);
    let to_remove = if total_length > params.max_length {
        total_length - params.max_length
    } else {
        return Ok((encoding, pair_encoding));
    };

    match params.strategy {
        TruncationStrategy::LongestFirst => {
            let mut n_first = encoding.get_ids().len();
            let mut n_second = pair_encoding.as_ref().map_or(0, |e| e.get_ids().len());
            for _ in 0..to_remove {
                if n_first > n_second {
                    n_first -= 1;
                } else {
                    n_second -= 1;
                }
            }

            encoding.truncate(n_first, params.stride);
            if let Some(encoding) = pair_encoding.as_mut() {
                encoding.truncate(n_second, params.stride);
            }
        }
        TruncationStrategy::OnlyFirst | TruncationStrategy::OnlySecond => {
            let target = if params.strategy == TruncationStrategy::OnlyFirst {
                Ok(&mut encoding)
            } else if let Some(encoding) = pair_encoding.as_mut() {
                Ok(encoding)
            } else {
                Err(Box::new(Error::SecondSequenceNotProvided))
            }?;

            let target_len = target.get_ids().len();
            if target_len > to_remove {
                target.truncate(target_len - to_remove, params.stride);
            } else {
                return Err(Box::new(Error::SequenceTooShort));
            }
        }
    }

    Ok((encoding, pair_encoding))
}

pub fn pad_encodings(
    mut encodings: Vec<Encoding>,
    params: &PaddingParams,
) -> Result<Vec<Encoding>> {
    if encodings.is_empty() {
        return Ok(encodings);
    }

    let pad_length = match params.strategy {
        PaddingStrategy::Fixed(size) => size,
        PaddingStrategy::BatchLongest => encodings
            .par_iter()
            .map(|e| e.get_ids().len())
            .max()
            .unwrap(),
    };

    encodings.par_iter_mut().for_each(|encoding| {
        encoding.pad(
            pad_length,
            params.pad_id,
            params.pad_type_id,
            &params.pad_token,
            params.direction,
        )
    });

    Ok(encodings)
}
