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
#[derive(Debug, Serialize, Clone)]
pub struct Tv{
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
    pub status: String,
    #[pyo3(get)]
    pub genres: Vec<String>,
    #[pyo3(get)]
    pub number_of_episodes: u64,
    #[pyo3(get)]
    pub number_of_seasons: u64,
    #[pyo3(get)]
    pub episode_run_time: u64,
    #[pyo3(get)]
    pub adding: String,
    #[pyo3(get)]
    pub seasons: Vec<Season>,
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
    pub watched: u64,
    #[pyo3(get)]
    pub updated: String,
}

#[pymethods]
impl Tv{

    pub fn set_seasons(&mut self) -> PyResult<()>{
        self.seasons = DATABASE.get_seasons(&self.user, self.id)?;
        Ok(())
    }

    pub fn set_persons(&mut self) -> PyResult<()>{
        self.cast = DATABASE.get_tv_cast(&self.user, self.id)?;
        self.crew = DATABASE.get_tv_crew(&self.user, self.id)?;
        Ok(())
    }

    pub fn set_trailers(&mut self) -> PyResult<()>{
        self.trailer = DATABASE.get_tv_trailer(self.id)?;
        Ok(())
    }

    pub fn set_keywords(&mut self) -> PyResult<()>{
        self.keyword = DATABASE.get_tv_keywords(self.id)?;
        Ok(())
    }

    pub fn set_collection(&mut self) -> PyResult<()>{
        self.collection = CollectionSearch::new(&self.user).tv(self.id)?.results(None, None)?;
        Ok(())
    }

    pub fn season(&self, season_number: u64) -> PyResult<Option<Season>>{
        Ok(DATABASE.get_season(&self.user, self.id, season_number)?)
    }

    pub fn episode(&self, season_number: u64, episode_number: u64) -> PyResult<Option<Episode>>{
        Ok(EpisodeSearch::new(&self.user).tv(self.id)?.season(season_number)?.episode(episode_number)?.last()?)
    }

    pub fn set_watched(&mut self, b: bool) -> PyResult<()>{
        self.set_seasons()?;
        for season in &mut self.seasons{
            season.set_watched(b)?;
        }
        Ok(())
    }

    pub fn delete(&mut self) -> PyResult<()>{
        if EpisodeSearch::new(&self.user).tv(self.id)?.exist()?{
            return Ok(())
        }
        self.set_persons()?;
        DATABASE.delete_tv(self.id)?;
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
#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct TvResult{
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
impl TvResult{
    pub fn full(&self) -> PyResult<Tv>{
        Ok(DATABASE.get_tv(&self.user, self.id)?.unwrap())
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
pub struct TvSearch{
   parameters: HashMap<String, Option<(String, String)>>,
   user: String,
   order_by: Option<String>,
}

impl TvSearch{
    pub fn new(user: &String) -> TvSearch{
        TvSearch{
            parameters: HashMap::new(),
            user: user.clone(),
            order_by: None,
        }
    }
}

#[pymethods]
impl TvSearch{
    pub fn id(&mut self, id: u64) -> PyResult<TvSearch>{
        self.find("Tvs.id", "=", Some(id.to_string()))
    }

    pub fn cast(&mut self, person_id: u64) -> PyResult<TvSearch>{
        self.find("TvCasts.person_id", "=", Some(person_id.to_string()))
    }

    pub fn crew(&mut self, person_id: u64) -> PyResult<TvSearch>{
        self.find("TvCrews.person_id", "=", Some(person_id.to_string()))
    }

    pub fn collection(&mut self, collection_id: u64) -> PyResult<TvSearch>{
        self.find("TvCollectionLinks.collection_id", "=", Some(collection_id.to_string()))
    }

    pub fn find(&mut self, column: &str, operator: &str, value: Option<String>) -> PyResult<TvSearch>{
        if let Some(value) = value {
            self.parameters.insert(column.to_string(), Some((operator.to_string(), value)));
        }
        else{
            self.parameters.insert(column.to_string(), None);
        }
        Ok(self.clone())
    }
    pub fn order_by(&mut self, order_by: String) -> PyResult<TvSearch>{
        self.order_by = Some(order_by);
        Ok(self.clone())
    }

    pub fn exist(&self) -> PyResult<bool>{
        Ok(self.results(None, None)?.len() > 0)
    } 

    pub fn results(&self, limit: Option<u64>, offset: Option<u64>) -> PyResult<Vec<TvResult>>{
        Ok(DATABASE.get_tvs(&self.user, &self.parameters, &self.order_by, limit, offset)?)
    }

    pub fn json_results(&self, limit: Option<u64>, offset: Option<u64>) -> PyResult<String>{
        let list = self.results(limit, offset)?;
        Ok(serde_json::to_string(&list).unwrap())
    }

    pub fn last(&self) -> PyResult<Option<TvResult>>{
        Ok(self.results(None, None)?.pop())
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
pub struct Season{
    pub user: String,
    #[pyo3(get)]
    pub tv_id: u64,
    #[pyo3(get)]
    pub season_number: u64,
    #[pyo3(get)]
    pub episode_count: u64,
    #[pyo3(get)]
    pub title: String,
    #[pyo3(get)]
    pub overview: String,
    #[pyo3(get)]
    pub poster_path: String,
    #[pyo3(get)]
    pub release_date: String,
    #[pyo3(get)]
    pub episodes: Vec<Episode>,
    #[pyo3(get)]
    pub tv: Option<Tv>,
    #[pyo3(get)]
    pub watched: u64,
    #[pyo3(get)]
    pub updated: String,
}

#[pymethods]
impl Season{

    pub fn set_tv(&mut self) -> PyResult<()>{
        self.tv = DATABASE.get_tv(&self.user, self.tv_id)?;
        Ok(())
    }

    pub fn episode(&mut self, episode_number: u64) -> PyResult<Option<Episode>>{
        Ok(EpisodeSearch::new(&self.user).tv(self.tv_id)?.season(self.season_number)?.episode(episode_number)?.last()?)
    }

    pub fn set_episodes(&mut self) -> PyResult<()>{
        self.episodes = EpisodeSearch::new(&self.user).tv(self.tv_id)?.season(self.season_number)?.results(None, None)?;
        Ok(())
    }

    pub fn set_episode_videos(&mut self) -> PyResult<()>{
        for episode in &mut self.episodes{
            episode.set_videos()?;
        }
        Ok(())
    }

    pub fn set_watched(&mut self, b: bool) -> PyResult<()>{
        self.set_episodes()?;
        for episode in &self.episodes{
            episode.set_watched(b)?;
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
#[derive(Debug, Clone)]
pub struct EpisodeSearch{
   parameters: HashMap<String, Option<(String, String)>>,
   user: String,
   order_by: Option<String>,
}

impl EpisodeSearch{
    pub fn new(user: &String) -> EpisodeSearch{
        EpisodeSearch{
            parameters: HashMap::new(),
            user: user.clone(),
            order_by: None,
        }
    }
}

#[pymethods]
impl EpisodeSearch{
    pub fn find(&mut self, column: &str, operator: &str, value: Option<String>) -> PyResult<EpisodeSearch>{
        if let Some(value) = value {
            self.parameters.insert(column.to_string(), Some((operator.to_string(), value)));
        }
        else{
            self.parameters.insert(column.to_string(), None);
        }
        Ok(self.clone())
    }

    pub fn id(&mut self, episode_id: u64) -> PyResult<EpisodeSearch>{
        self.find("Episodes.id", "=", Some(episode_id.to_string()))
    }

    pub fn season(&mut self, season_number: u64) -> PyResult<EpisodeSearch>{
        self.find("Episodes.season_number", "=", Some(season_number.to_string()))
    }

    pub fn episode(&mut self, episode_number: u64) -> PyResult<EpisodeSearch>{
        self.find("Episodes.episode_number", "=", Some(episode_number.to_string()))
    }

    pub fn tv(&mut self, tv_id: u64) -> PyResult<EpisodeSearch>{
        self.find("Episodes.tv_id", "=", Some(tv_id.to_string()))
    }

    pub fn cast(&mut self, person_id: u64) -> PyResult<EpisodeSearch>{
        self.find("EpisodeCasts.person_id", "=", Some(person_id.to_string()))
    }

    pub fn crew(&mut self, person_id: u64) -> PyResult<EpisodeSearch>{
        self.find("EpisodeCrews.person_id", "=", Some(person_id.to_string()))
    }

    pub fn order_by(&mut self, order_by: String) -> PyResult<EpisodeSearch>{
        self.order_by = Some(order_by);
        Ok(self.clone())
    }

    pub fn exist(&self) -> PyResult<bool>{
        Ok(self.results(None, None)?.len() > 0)
    } 

    pub fn results(&self, limit: Option<u64>, offset: Option<u64>) -> PyResult<Vec<Episode>>{
        Ok(DATABASE.get_episodes(&self.user, &self.parameters, &self.order_by, limit, offset)?)
    }

    pub fn json_results(&self, limit: Option<u64>, offset: Option<u64>) -> PyResult<String>{
        let list = self.results(limit, offset)?;
        Ok(serde_json::to_string(&list).unwrap())
    }

    pub fn last(&self) -> PyResult<Option<Episode>>{
        Ok(self.results(None, None)?.pop())
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
pub struct Episode{
    pub user: String,
    #[pyo3(get)]
    pub tv_id: u64,
    #[pyo3(get)]
    pub id: u64,
    #[pyo3(get)]
    pub season_number: u64,
    #[pyo3(get)]
    pub episode_number: u64,
    #[pyo3(get)]
    pub release_date: String,
    #[pyo3(get)]
    pub title: String,
    #[pyo3(get)]
    pub overview: String,
    #[pyo3(get)]
    pub vote_average: f64,
    #[pyo3(get)]
    pub vote_count: u64,
    #[pyo3(get)]
    pub video: Vec<VideoResult>,
    #[pyo3(get)]
    pub tv: Option<Tv>,
    #[pyo3(get)]
    pub season: Option<Season>,
    #[pyo3(get)]
    pub watched: u64,
    #[pyo3(get)]
    pub updated: String,
    #[pyo3(get)]
    pub poster_path: String,
    #[pyo3(get)]
    pub tv_title: String,
    #[pyo3(get)]
    pub cast: Vec<Cast>,
    #[pyo3(get)]
    pub crew: Vec<Crew>,

}

#[pymethods]
impl Episode{

    pub fn set_tv(&mut self) -> PyResult<()>{
        self.tv = DATABASE.get_tv(&self.user, self.tv_id)?;
        Ok(())
    }

    pub fn set_season(&mut self) -> PyResult<()>{
        self.season = DATABASE.get_season(&self.user, self.tv_id, self.season_number)?;
        Ok(())
    }

    pub fn set_videos(&mut self) -> PyResult<()>{
        self.video = VideoSearch::new(&self.user).tv()?.media_id(self.id)?.results(None, None)?;
        Ok(())
    }

    pub fn set_persons(&mut self) -> PyResult<()>{
        self.cast = DATABASE.get_episode_cast(&self.user, self.id)?;
        self.crew = DATABASE.get_episode_crew(&self.user, self.id)?;
        Ok(())
    }

    pub fn set_watched(&self, b: bool) -> PyResult<()>{
        if b{
            Ok(DATABASE.set_episode_watched(self.user.clone(), self.id, self.watched+1)?)
        }
        else{
            Ok(DATABASE.set_episode_watched(self.user.clone(), self.id, 0)?)
        }
        
    }

    pub fn delete(&mut self) -> PyResult<()>{
        if VideoSearch::new(&self.user).tv()?.media_id(self.id)?.exist()?{
            return Ok(())
        }

        self.set_tv()?;

        DATABASE.delete_episode(self.id)?;

        if let Some(tv) = &mut self.tv{
            tv.delete()?;
        }

        //todo person remove for episode

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
