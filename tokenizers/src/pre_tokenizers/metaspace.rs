use crate::tokenizer::{Decoder, NormalizedString, Offsets, PreTokenizer, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
/// Replaces all the whitespaces by the provided meta character and then
/// splits on this character
pub struct Metaspace {
    replacement: char,
    add_prefix_space: bool,
}

impl Metaspace {
    pub fn new(replacement: char, add_prefix_space: bool) -> Self {
        Self {
            replacement,
            add_prefix_space,
        }
    }
}

impl Default for Metaspace {
    fn default() -> Self {
        Self::new('▁', true)
    }
}

#[typetag::serde]
impl PreTokenizer for Metaspace {
    fn pre_tokenize(&self, normalized: &mut NormalizedString) -> Result<Vec<(String, Offsets)>> {
        if self.add_prefix_space && !normalized.get().starts_with(' ') {
            normalized.prepend(" ");
        }

        let mut words = vec![];
        let mut word = Vec::with_capacity(1000);
        let mut offset = 0;
        normalized.get().chars().for_each(|c| {
            if c.is_whitespace() {
                if !word.is_empty() {
                    let offsets = (offset - word.len(), offset);
                    words.push((word.drain(0..).collect::<String>(), offsets));
                }
                word.push(self.replacement)
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

#[typetag::serde]
impl Decoder for Metaspace {
    fn decode(&self, tokens: Vec<String>) -> Result<String> {
        Ok(tokens
            .iter()
            .flat_map(|t| t.chars())
            .enumerate()
            .filter_map(|(i, c)| {
                if c == self.replacement {
                    if i == 0 && self.add_prefix_space {
                        None
                    } else {
                        Some(' ')
                    }
                } else {
                    Some(c)
                }
            })
            .collect::<String>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let pretok = Metaspace::new('▁', true);
        let mut input = NormalizedString::from("Hey friend!");
        let res = pretok.pre_tokenize(&mut input).unwrap();
        assert_eq!(
            &res,
            &[("▁Hey".into(), (0, 4)), ("▁friend!".into(), (4, 12)),]
        );
    }

    #[test]
    fn multiple_spaces() {
        let pretok = Metaspace::new('▁', true);
        let mut input = NormalizedString::from("Hey   friend!");
        let res = pretok.pre_tokenize(&mut input).unwrap();
        assert_eq!(
            &res,
            &[
                ("▁Hey".into(), (0, 4)),
                ("▁".into(), (4, 5)),
                ("▁".into(), (5, 6)),
                ("▁friend!".into(), (6, 14)),
            ]
        );
    }

    #[test]
    fn decode() {
        let decoder = Metaspace::new('▁', true);
        let res = decoder
            .decode(vec!["▁Hey".into(), "▁friend!".into()])
            .unwrap();
        assert_eq!(&res, "Hey friend!")
    }
}
