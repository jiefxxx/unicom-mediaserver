use pyo3::prelude::*;

#[pyclass]
#[derive(Debug, Serialize, Clone)]
pub struct Genre{
    #[pyo3(get)]
    pub id: u64,
    #[pyo3(get)]
    pub name: String,
}

#[pymethods]
impl Genre{
    fn __str__(&self) -> PyResult<String>{
        Ok(format!("{:?}", self))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}