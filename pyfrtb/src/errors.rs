use std::fmt::{Debug, Formatter};
use std::io::Error;

use polars::prelude::PolarsError;
use pyo3::{create_exception, PyErr};
use pyo3::exceptions::{PyException, PyIOError};
//use pyo3::prelude::*;
use thiserror::Error;

#[derive(Error)]
pub enum PyUltimaErr {
    #[error(transparent)]
    Polars(#[from] PolarsError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error("{0}")]
    Other(String),
}

impl std::convert::From<std::io::Error> for PyUltimaErr {
    fn from(value: Error) -> Self {
        PyUltimaErr::Other(format!("{:?}", value))
    }
}

impl std::convert::From<PyUltimaErr> for PyErr {
    fn from(err: PyUltimaErr) -> PyErr {
        //let default = || PyRuntimeError::new_err(format!("{:?}", &err));

        use PyUltimaErr::*;
        match &err {
            Polars(err) => match err {
                PolarsError::NotFound(name) => NotFoundError::new_err(name.to_string()),
                PolarsError::ComputeError(err) => ComputeError::new_err(err.to_string()),
                PolarsError::NoData(err) => NoDataError::new_err(err.to_string()),
                PolarsError::ShapeMisMatch(err) => ShapeError::new_err(err.to_string()),
                PolarsError::SchemaMisMatch(err) => SchemaError::new_err(err.to_string()),
                PolarsError::Io(err) => PyIOError::new_err(err.to_string()),
                PolarsError::ArrowError(err) => ArrowErrorException::new_err(format!("{:?}", err)),
                PolarsError::Duplicate(err) => DuplicateError::new_err(err.to_string()),
                PolarsError::InvalidOperation(err) => {
                    InvalidOperationError::new_err(err.to_string())
                }
            },
            SerdeJson(err) => SerdeJsonError::new_err(format!("Couldn't (de)serialise input. Check format. {}", err)),
            Other(_str) => OtherError::new_err(format!("{_str}")),

        }
    }
}

impl Debug for PyUltimaErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use PyUltimaErr::*;
        match self {
            Polars(err) => write!(f, "{:?}", err),
            SerdeJson(err) => write!(f, "Couldn't serialize string. Check format. {:?}", err),
            Other(err) => write!(f, "BindingsError: {:?}", err),
        }
    }
}

create_exception!(exceptions, NotFoundError, PyException);
create_exception!(exceptions, ComputeError, PyException);
create_exception!(exceptions, NoDataError, PyException);
create_exception!(exceptions, ArrowErrorException, PyException);
create_exception!(exceptions, ShapeError, PyException);
create_exception!(exceptions, SchemaError, PyException);
create_exception!(exceptions, DuplicateError, PyException);
create_exception!(exceptions, InvalidOperationError, PyException);
create_exception!(exceptions, SerdeJsonError, PyException);
create_exception!(exceptions, OtherError, PyException);


