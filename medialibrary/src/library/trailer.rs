
use pyo3::prelude::*;

#[pyclass]
#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct Trailer{
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub youtube_id: String,
}

#[pymethods]
impl Trailer {
    fn __str__(&self) -> PyResult<String>{
        Ok(format!("{:?}", self))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}