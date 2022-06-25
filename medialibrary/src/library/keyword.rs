use pyo3::prelude::*;

#[pyclass]
#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct Keyword{
    #[pyo3(get)]
    pub id: u64,
    #[pyo3(get)]
    pub name: String,
}

#[pymethods]
impl Keyword {
    fn __str__(&self) -> PyResult<String>{
        Ok(format!("{:?}", self))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}