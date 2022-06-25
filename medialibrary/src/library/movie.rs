
use std::collections::HashMap;

use pyo3::prelude::*;

use crate::database::DATABASE;

use super::cast::Cast;
use super::cast::Crew;
use super::collection::CollectionResult;
use super::collection::CollectionSearch;
use super::keyword::Keyword;
use super::trailer::Trailer;
use super::video::VideoResult;
use super::video::VideoSearch;

#[pyclass]
#[derive(Debug, Serialize)]
pub struct Movie{
    pub user: String,
    #[pyo3(get)]
    pub id: u64,
    #[pyo3(get)]
    pub original_title: String,
    #[pyo3(get)]
    pub original_language: String,
    #[pyo3(get)]
    pub title: String,
    #[pyo3(get)]
    pub release_date: String,
    #[pyo3(get)]
    pub overview: String,
    #[pyo3(get)]
    pub popularity: f64,
    #[pyo3(get)]
    pub poster_path: String,
    #[pyo3(get)]
    pub backdrop_path: String,
    #[pyo3(get)]
    pub vote_average: f64,
    #[pyo3(get)]
    pub vote_count: i64,
    #[pyo3(get)]
    pub tagline: String,
    #[pyo3(get)]
    pub status: String,
    #[pyo3(get)]
    pub genres: Vec<String>,
    #[pyo3(get)]
    pub adding: String,
    #[pyo3(get)]
    pub watched: u64,
    #[pyo3(get)]
    pub video: Vec<VideoResult>,
    #[pyo3(get)]
    pub cast: Vec<Cast>,
    #[pyo3(get)]
    pub crew: Vec<Crew>,
    #[pyo3(get)]
    pub trailer: Vec<Trailer>,
    #[pyo3(get)]
    pub keyword: Vec<Keyword>,
    #[pyo3(get)]
    pub collection: Vec<CollectionResult>,
    #[pyo3(get)]
    pub updated: String,
}

#[pymethods]
impl Movie{

    pub fn set_videos(&mut self) -> PyResult<()>{
        self.video = VideoSearch::new(&self.user).movie()?.media_id(self.id)?.results(None, None)?;
        Ok(())
    }

    pub fn set_collection(&mut self) -> PyResult<()>{
        self.collection = CollectionSearch::new(&self.user).movie(self.id)?.results(None, None)?;
        Ok(())
    }

    pub fn set_persons(&mut self) -> PyResult<()>{
        self.cast = DATABASE.get_movie_cast(&self.user, self.id)?;
        self.crew = DATABASE.get_movie_crew(&self.user, self.id)?;
        Ok(())
    }

    pub fn set_trailers(&mut self) -> PyResult<()>{
        self.trailer = DATABASE.get_movie_trailer(self.id)?;
        Ok(())
    }

    pub fn set_keywords(&mut self) -> PyResult<()>{
        self.keyword = DATABASE.get_movie_keywords(self.id)?;
        Ok(())
    }

    pub fn set_watched(&self, b: bool) -> PyResult<()>{
        if b{
            Ok(DATABASE.set_movie_watched(self.user.clone(), self.id, self.watched+1)?)
        }
        else{
            Ok(DATABASE.set_movie_watched(self.user.clone(), self.id, 0)?)
        }
        
    }

    pub fn delete(&mut self) -> PyResult<()>{
        if VideoSearch::new(&self.user).media_id(self.id)?.exist()?{
            return Ok(())
        }
        self.set_persons()?;
        DATABASE.delete_movie(self.id)?;
        for crew in &self.crew{
            crew.full()?.delete()?;
        }
        for cast in &self.cast{
            cast.full()?.delete()?;
        }
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
pub struct MovieResult{
    pub user: String,
    #[pyo3(get)]
    pub id: u64,
    #[pyo3(get)]
    pub title: String,
    #[pyo3(get)]
    pub release_date: String,
    #[pyo3(get)]
    pub poster_path: String,
    #[pyo3(get)]
    pub backdrop_path: String,
    #[pyo3(get)]
    pub vote_average: f64,
    #[pyo3(get)]
    pub genres: Vec<String>,
    #[pyo3(get)]
    pub adding: String,
    #[pyo3(get)]
    pub watched: u64,
}

#[pymethods]
impl MovieResult{
    pub fn full(&self) -> PyResult<Movie>{
        Ok(DATABASE.get_movie(&self.user, self.id)?.unwrap())
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
pub struct MovieSearch{
   parameters: HashMap<String, Option<(String, String)>>,
   user: String,
   order_by: Option<String>,
}

impl MovieSearch{
    pub fn new(user: &String) -> MovieSearch{
        MovieSearch{
            parameters: HashMap::new(),
            user: user.clone(),
            order_by: None,
        }
    }
}

#[pymethods]
impl MovieSearch{
    pub fn id(&mut self, id: u64) -> PyResult<MovieSearch>{
        self.find("Movies.id", "=", Some(id.to_string()))
    }

    pub fn cast(&mut self, person_id: u64) -> PyResult<MovieSearch>{
        self.find("MovieCasts.person_id", "=", Some(person_id.to_string()))
    }

    pub fn collection(&mut self, collection_id: u64) -> PyResult<MovieSearch>{
        self.find("MovieCollectionLinks.collection_id", "=", Some(collection_id.to_string()))
    }

    pub fn crew(&mut self, person_id: u64) -> PyResult<MovieSearch>{
        self.find("MovieCrews.person_id", "=", Some(person_id.to_string()))
    }

    pub fn find(&mut self, column: &str, operator: &str, value: Option<String>) -> PyResult<MovieSearch>{
        if let Some(value) = value {
            self.parameters.insert(column.to_string(), Some((operator.to_string(), value)));
        }
        else{
            self.parameters.insert(column.to_string(), None);
        }
        Ok(self.clone())
    }

    pub fn order_by(&mut self, order_by: String) -> PyResult<MovieSearch>{
        self.order_by = Some(order_by);
        Ok(self.clone())

    }

    pub fn exist(&self) -> PyResult<bool>{
        Ok(self.results(None, None)?.len() > 0)
    } 

    pub fn results(&self, limit: Option<u64>, offset: Option<u64>) -> PyResult<Vec<MovieResult>>{
        Ok(DATABASE.get_movies(&self.user, &self.parameters, &self.order_by, limit, offset)?)
    }

    pub fn json_results(&self, limit: Option<u64>, offset: Option<u64>) -> PyResult<String>{
        let list = self.results(limit, offset)?;
        Ok(serde_json::to_string(&list).unwrap())
    }

    pub fn last(&self) -> PyResult<Option<MovieResult>>{
        Ok(self.results(None, None)?.pop())
    }

    fn __str__(&self) -> PyResult<String>{
        Ok(format!("{:?}", self))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}