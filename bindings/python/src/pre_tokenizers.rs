extern crate tokenizers as tk;

use super::error::{PyError, ToPyResult};
use super::utils::Container;
use pyo3::exceptions;
use pyo3::prelude::*;
use pyo3::types::*;
use tk::tokenizer::{Offsets, Result};

#[pyclass(dict)]
pub struct PreTokenizer {
    pub pretok: Container<dyn tk::tokenizer::PreTokenizer>,
}
#[pymethods]
impl PreTokenizer {
    #[staticmethod]
    fn custom(pretok: PyObject) -> PyResult<Self> {
        let py_pretok = PyPreTokenizer::new(pretok)?;
        Ok(PreTokenizer {
            pretok: Container::Owned(Box::new(py_pretok)),
        })
    }

    fn pre_tokenize(&self, s: &str) -> PyResult<Vec<(String, Offsets)>> {
        // TODO: Expose the NormalizedString
        let mut normalized = tk::tokenizer::NormalizedString::from(s);
        ToPyResult(
            self.pretok
                .execute(|pretok| pretok.pre_tokenize(&mut normalized)),
        )
        .into()
    }
}

#[pyclass(extends=PreTokenizer)]
pub struct ByteLevel {}
#[pymethods]
impl ByteLevel {
    #[new]
    #[args(kwargs = "**")]
    fn new(kwargs: Option<&PyDict>) -> PyResult<(Self, PreTokenizer)> {
        let mut byte_level = tk::pre_tokenizers::byte_level::ByteLevel::default();
        if let Some(kwargs) = kwargs {
            for (key, value) in kwargs {
                let key: &str = key.extract()?;
                match key {
                    "add_prefix_space" => {
                        byte_level = byte_level.add_prefix_space(value.extract()?)
                    }
                    _ => println!("Ignored unknown kwargs option {}", key),
                }
            }
        }

        Ok((
            ByteLevel {},
            PreTokenizer {
                pretok: Container::Owned(Box::new(byte_level)),
            },
        ))
    }

    #[staticmethod]
    fn alphabet() -> Vec<String> {
        tk::pre_tokenizers::byte_level::ByteLevel::alphabet()
            .into_iter()
            .map(|c| c.to_string())
            .collect()
    }
}

#[pyclass(extends=PreTokenizer)]
pub struct Whitespace {}
#[pymethods]
impl Whitespace {
    #[new]
    fn new() -> PyResult<(Self, PreTokenizer)> {
        Ok((
            Whitespace {},
            PreTokenizer {
                pretok: Container::Owned(Box::new(tk::pre_tokenizers::whitespace::Whitespace)),
            },
        ))
    }
}

#[pyclass(extends=PreTokenizer)]
pub struct WhitespaceSplit {}
#[pymethods]
impl WhitespaceSplit {
    #[new]
    fn new() -> PyResult<(Self, PreTokenizer)> {
        Ok((
            WhitespaceSplit {},
            PreTokenizer {
                pretok: Container::Owned(Box::new(tk::pre_tokenizers::whitespace::WhitespaceSplit)),
            },
        ))
    }
}

#[pyclass(extends=PreTokenizer)]
pub struct CharDelimiterSplit {}
#[pymethods]
impl CharDelimiterSplit {
    #[new]
    pub fn new(delimiter: &str) -> PyResult<(Self, PreTokenizer)> {
        let chr_delimiter = delimiter
            .chars()
            .nth(0)
            .ok_or(exceptions::Exception::py_err(
                "delimiter must be a single character",
            ))?;
        Ok((
            CharDelimiterSplit {},
            PreTokenizer {
                pretok: Container::Owned(Box::new(
                    tk::pre_tokenizers::delimiter::CharDelimiterSplit::new(chr_delimiter),
                )),
            },
        ))
    }
}

#[pyclass(extends=PreTokenizer)]
pub struct BertPreTokenizer {}
#[pymethods]
impl BertPreTokenizer {
    #[new]
    fn new() -> PyResult<(Self, PreTokenizer)> {
        Ok((
            BertPreTokenizer {},
            PreTokenizer {
                pretok: Container::Owned(Box::new(tk::pre_tokenizers::bert::BertPreTokenizer)),
            },
        ))
    }
}

#[pyclass(extends=PreTokenizer)]
pub struct Metaspace {}
#[pymethods]
impl Metaspace {
    #[new]
    #[args(kwargs = "**")]
    fn new(kwargs: Option<&PyDict>) -> PyResult<(Self, PreTokenizer)> {
        let mut replacement = '▁';
        let mut add_prefix_space = true;

        if let Some(kwargs) = kwargs {
            for (key, value) in kwargs {
                let key: &str = key.extract()?;
                match key {
                    "replacement" => {
                        let s: &str = value.extract()?;
                        replacement = s.chars().nth(0).ok_or(exceptions::Exception::py_err(
                            "replacement must be a character",
                        ))?;
                    }
                    "add_prefix_space" => add_prefix_space = value.extract()?,
                    _ => println!("Ignored unknown kwarg option {}", key),
                }
            }
        }

        Ok((
            Metaspace {},
            PreTokenizer {
                pretok: Container::Owned(Box::new(tk::pre_tokenizers::metaspace::Metaspace::new(
                    replacement,
                    add_prefix_space,
                ))),
            },
        ))
    }
}

/// Attempt at providing Python the ability to give its own PreTokenizer
struct PyPreTokenizer {
    class: PyObject,
}

impl PyPreTokenizer {
    pub fn new(class: PyObject) -> PyResult<Self> {
        Ok(PyPreTokenizer { class })
    }
}

impl tk::tokenizer::PreTokenizer for PyPreTokenizer {
    fn pre_tokenize(
        &self,
        sentence: &mut tk::tokenizer::NormalizedString,
    ) -> Result<Vec<(String, Offsets)>> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let args = PyTuple::new(py, &[sentence.get()]);
        match self.class.call_method(py, "pre_tokenize", args, None) {
            Ok(res) => Ok(res
                .cast_as::<PyList>(py)
                .map_err(|_| {
                    PyError::from("`pre_tokenize is expected to return a List[(str, (uint, uint))]")
                })?
                .extract::<Vec<(String, Offsets)>>()
                .map_err(|_| {
                    PyError::from(
                        "`pre_tokenize` is expected to return a List[(str, (uint, uint))]",
                    )
                })?),
            Err(e) => {
                e.print(py);
                Err(Box::new(PyError::from(
                    "Error while calling `pre_tokenize`",
                )))
            }
        }
    }
}
