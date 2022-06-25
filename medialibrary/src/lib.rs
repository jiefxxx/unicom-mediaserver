#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;

mod database;
mod rustmdb;
mod library;

use pyo3::prelude::*;
use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::PyErr;


use library::Library;
use library::video::Video;

create_exception!(medialibrary, DatabaseError, PyException);

impl std::convert::From<database::Error> for PyErr {
    fn from(err: database::Error) -> PyErr {
        DatabaseError::new_err(err.to_string())
    }
}

use rustmdb::{set_api_key, set_language, Tmdb};

create_exception!(medialibrary, TmdbError, PyException);

impl std::convert::From<rustmdb::Error> for PyErr {
    fn from(err: rustmdb::Error) -> PyErr {
        TmdbError::new_err(err.to_string())
    }
}

create_exception!(medialibrary, LibraryError, PyException);

impl std::convert::From<library::Error> for PyErr {
    fn from(err: library::Error) -> PyErr {
        LibraryError::new_err(err.to_string())
    }
}

#[pyfunction]
fn tmdb_init(key: &str, lang: &str)  -> PyResult<()> {
    set_api_key(key);
    set_language(lang);
    Ok(())
}

#[pymodule]
fn medialibrary(py: Python, module: &PyModule) -> PyResult<()> {
    module.add("TmdbError", py.get_type::<TmdbError>())?;
    module.add("LibraryError", py.get_type::<LibraryError>())?;
    module.add_function(wrap_pyfunction!(tmdb_init, module)?)?;
    module.add_class::<Tmdb>()?;
    module.add_class::<Library>()?;
    module.add_class::<Video>()?;
    Ok(())
}
