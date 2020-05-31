use crate::tokenizer::{NormalizedString, Offsets, PreTokenizer, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CharDelimiterSplit {
    delimiter: char,
}

impl CharDelimiterSplit {
    pub fn new(delimiter: char) -> Self {
        CharDelimiterSplit { delimiter }
    }
}

#[typetag::serde]
impl PreTokenizer for CharDelimiterSplit {
    fn pre_tokenize(&self, normalized: &mut NormalizedString) -> Result<Vec<(String, Offsets)>> {
        let mut words = vec![];
        let mut word = Vec::with_capacity(1000);
        let mut offset = 0;

        normalized.get().chars().for_each(|c| {
            if c == self.delimiter {
                if !word.is_empty() {
                    let offsets = (offset - word.len(), offset);
                    words.push((word.drain(0..).collect::<String>(), offsets));
                }
            } else {
                word.push(c);
            }
            offset += 1;
        });
        if !word.is_empty() {
            let offsets = (offset - word.len(), offset);
            words.push((word.drain(0..).collect::<String>(), offsets));
        }

        Ok(words)
    }
}
