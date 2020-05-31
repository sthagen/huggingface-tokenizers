extern crate tokenizers as tk;

use super::utils::Container;
use pyo3::exceptions;
use pyo3::prelude::*;
use pyo3::types::*;

#[pyclass(dict, module = "tokenizers.processors")]
pub struct PostProcessor {
    pub processor: Container<dyn tk::tokenizer::PostProcessor>,
}

#[pymethods]
impl PostProcessor {
    fn __getstate__(&self, py: Python) -> PyResult<PyObject> {
        let data = self
            .processor
            .execute(|processor| serde_json::to_string(&processor))
            .map_err(|e| {
                exceptions::Exception::py_err(format!(
                    "Error while attempting to pickle PostProcessor: {}",
                    e.to_string()
                ))
            })?;
        Ok(PyBytes::new(py, data.as_bytes()).to_object(py))
    }

    fn __setstate__(&mut self, py: Python, state: PyObject) -> PyResult<()> {
        match state.extract::<&PyBytes>(py) {
            Ok(s) => {
                self.processor =
                    Container::Owned(serde_json::from_slice(s.as_bytes()).map_err(|e| {
                        exceptions::Exception::py_err(format!(
                            "Error while attempting to unpickle PostProcessor: {}",
                            e.to_string()
                        ))
                    })?);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    fn num_special_tokens_to_add(&self, is_pair: bool) -> usize {
        self.processor.execute(|p| p.added_tokens(is_pair))
    }
}

#[pyclass(extends=PostProcessor, module = "tokenizers.processors")]
pub struct BertProcessing {}
#[pymethods]
impl BertProcessing {
    #[new]
    fn new(sep: (String, u32), cls: (String, u32)) -> PyResult<(Self, PostProcessor)> {
        Ok((
            BertProcessing {},
            PostProcessor {
                processor: Container::Owned(Box::new(tk::processors::bert::BertProcessing::new(
                    sep, cls,
                ))),
            },
        ))
    }

    fn __getnewargs__<'p>(&self, py: Python<'p>) -> PyResult<&'p PyTuple> {
        Ok(PyTuple::new(py, &[("", 0), ("", 0)]))
    }
}

#[pyclass(extends=PostProcessor, module = "tokenizers.processors")]
pub struct RobertaProcessing {}
#[pymethods]
impl RobertaProcessing {
    #[new]
    #[args(trim_offsets = true, add_prefix_space = true)]
    fn new(
        sep: (String, u32),
        cls: (String, u32),
        trim_offsets: bool,
        add_prefix_space: bool,
    ) -> PyResult<(Self, PostProcessor)> {
        Ok((
            RobertaProcessing {},
            PostProcessor {
                processor: Container::Owned(Box::new(
                    tk::processors::roberta::RobertaProcessing::new(sep, cls)
                        .trim_offsets(trim_offsets)
                        .add_prefix_space(add_prefix_space),
                )),
            },
        ))
    }

    fn __getnewargs__<'p>(&self, py: Python<'p>) -> PyResult<&'p PyTuple> {
        Ok(PyTuple::new(py, &[("", 0), ("", 0)]))
    }
}

#[pyclass(extends=PostProcessor, module = "tokenizers.processors")]
pub struct ByteLevel {}
#[pymethods]
impl ByteLevel {
    #[new]
    #[args(kwargs = "**")]
    fn new(kwargs: Option<&PyDict>) -> PyResult<(Self, PostProcessor)> {
        let mut byte_level = tk::processors::byte_level::ByteLevel::default();

        if let Some(kwargs) = kwargs {
            for (key, value) in kwargs {
                let key: &str = key.extract()?;
                match key {
                    "trim_offsets" => byte_level = byte_level.trim_offsets(value.extract()?),
                    _ => println!("Ignored unknown kwargs option {}", key),
                }
            }
        }
        Ok((
            ByteLevel {},
            PostProcessor {
                processor: Container::Owned(Box::new(byte_level)),
            },
        ))
    }
}
