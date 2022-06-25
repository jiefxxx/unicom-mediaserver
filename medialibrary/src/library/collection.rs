use std::collections::HashMap;

use pyo3::prelude::*;

use crate::database::DATABASE;

use super::movie::MovieResult;
use super::movie::MovieSearch;
use super::tv::TvResult;
use super::tv::TvSearch;

#[pyclass]
#[derive(Debug, Serialize, Clone)]
pub struct Collection{
    #[pyo3(get)]
    pub user: String,
    #[pyo3(get)]
    pub id: u64,
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub description: String,
    #[pyo3(get)]
    pub creator: String,
    #[pyo3(get)]
    pub creation_date: String,
    #[pyo3(get)]
    pub poster_path: String,
    #[pyo3(get)]
    pub movie: Vec<MovieResult>,
    #[pyo3(get)]
    pub tv: Vec<TvResult>, 
}

#[pymethods]
impl Collection{

    pub fn set_movie(&mut self) -> PyResult<()>{
        println!("self.id {}", self.id);
        self.movie = MovieSearch::new(&self.user).collection(self.id)?.results(None, None)?;
        println!("self.id {:?}", self.movie);
        Ok(())
    }

    pub fn set_tv(&mut self) -> PyResult<()>{
        self.tv = TvSearch::new(&self.user).collection(self.id)?.results(None, None)?;
        Ok(())
    }

    pub fn add_movie(&mut self, movie_id: u64) -> PyResult<()>{
        let movie = MovieSearch::new(&self.user).id(movie_id)?.last()?.unwrap();
        if self.poster_path.len() == 0{
            self.poster_path = movie.poster_path
        }
        Ok(DATABASE.add_movie_collection(self.id, movie_id)?)
    }

    pub fn add_tv(&mut self, tv_id: u64) -> PyResult<()>{
        let movie = TvSearch::new(&self.user).id(tv_id)?.last()?.unwrap();
        if self.poster_path.len() == 0{
            self.poster_path = movie.poster_path
        }
        Ok(DATABASE.add_tv_collection(self.id, tv_id)?)
    }

    pub fn edit_description(&mut self, description: String){
        self.description = description;
    }

    pub fn edit_poster_path(&mut self, poster_path: String){
        self.poster_path = poster_path;
    }

    pub fn save(&self)  -> PyResult<Collection>{
        Ok(DATABASE.update_collection(&self.user, &self)?)
    }

    pub fn delete(&self) -> PyResult<()>{
        DATABASE.delete_collection(self.id)?;
        Ok(())
    }


    pub fn json(&self) -> PyResult<String>{
        return Ok(serde_json::to_string(self).unwrap())
    }

    fn __str__(&self) -> PyResult<String>{
        Ok(format!("{:?}", self))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}

#[pyclass]
#[derive(Debug, Serialize, Clone)]
pub struct CollectionResult{
    #[pyo3(get)]
    pub user: String,
    #[pyo3(get)]
    pub id: u64,
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub creator: String,
    #[pyo3(get)]
    pub creation_date: String,
    #[pyo3(get)]
    pub poster_path: String,
}

#[pymethods]
impl CollectionResult{
    pub fn full(&self) -> PyResult<Collection>{
        Ok(DATABASE.get_collection(&self.user, self.id)?.unwrap())
    }

    fn __str__(&self) -> PyResult<String>{
        Ok(format!("{:?}", self))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct CollectionSearch{
   parameters: HashMap<String, Option<(String, String)>>,
   user: String,
   order_by: Option<String>,
}

impl CollectionSearch{
    pub fn new(user: &String) -> CollectionSearch{
        CollectionSearch{
            parameters: HashMap::new(),
            user: user.clone(),
            order_by: None,
        }
    }
}

#[pymethods]
impl CollectionSearch{
    pub fn movie(&mut self, movie_id: u64) -> PyResult<CollectionSearch>{
        self.find("MovieCollectionLinks.movie_id", "=", Some(movie_id.to_string()))
    }

    pub fn tv(&mut self, tv_id: u64) -> PyResult<CollectionSearch>{
        self.find("TvCollectionLinks.tv_id", "=", Some(tv_id.to_string()))
    }

    pub fn restrict(&mut self) -> PyResult<CollectionSearch>{
        self.find("Collections.creator", "=", Some(self.user.clone()))
    }

    pub fn find(&mut self, column: &str, operator: &str, value: Option<String>) -> PyResult<CollectionSearch>{
        if let Some(value) = value {
            self.parameters.insert(column.to_string(), Some((operator.to_string(), value)));
        }
        else{
            self.parameters.insert(column.to_string(), None);
        }
        Ok(self.clone())
    }

    pub fn order_by(&mut self, order_by: String) -> PyResult<CollectionSearch>{
        self.order_by = Some(order_by);
        Ok(self.clone())

    }

    pub fn exist(&self) -> PyResult<bool>{
        Ok(self.results(None, None)?.len() > 0)
    } 

    pub fn results(&self, limit: Option<u64>, offset: Option<u64>) -> PyResult<Vec<CollectionResult>>{
        Ok(DATABASE.get_collections(&self.user, &self.parameters, &self.order_by, limit, offset)?)
    }

    pub fn json_results(&self, limit: Option<u64>, offset: Option<u64>) -> PyResult<String>{
        let list = self.results(limit, offset)?;
        Ok(serde_json::to_string(&list).unwrap())
    }

    pub fn last(&self) -> PyResult<Option<CollectionResult>>{
        Ok(self.results(None, None)?.pop())
    }

    fn __str__(&self) -> PyResult<String>{
        Ok(format!("{:?}", self))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}