use super::{Error, ErrorKind, TMDBKEY, LANGUAGE};

use super::model::{SearchMovie, SearchResult, ErrorModel};

pub struct MovieSearch <'a>{
    api_key: String,
    language: String,
    query: &'a str,
    page: u64,
    include_adult: Option<bool>,
    region: Option<&'a str>,
    year: Option<u64>,
    primary_release_year: Option<u64>
}

impl <'a>MovieSearch<'a> {
    pub fn new(query: &'a str,) -> MovieSearch<'a>{
        MovieSearch{
            api_key: TMDBKEY.lock().unwrap().to_string(),
            language: LANGUAGE.lock().unwrap().to_string(),
            query,
            page: 1,
            include_adult: None,
            region: None,
            year: None,
            primary_release_year: None,
        }
    }

    #[allow(dead_code)]
    pub fn page(&mut self, page: u64) -> &mut MovieSearch<'a>{
        self.page = page;
        self
    }

    #[allow(dead_code)]
    pub fn language(&mut self, language: &str) -> &mut MovieSearch<'a>{
        self.language = language.to_string();
        self
    }

    #[allow(dead_code)]
    pub fn include_adult(&mut self, include_adult: bool) -> &mut MovieSearch<'a>{
        self.include_adult = Some(include_adult);
        self
    }

    #[allow(dead_code)]
    pub fn region(&mut self, region: &'a str)-> &mut MovieSearch<'a>{
        self.region = Some(region);
        self
    }

    pub fn year(&mut self, year: u64)-> &mut MovieSearch<'a>{
        self.year = Some(year);
        self
    }

    #[allow(dead_code)]
    pub fn primary_release_year(&mut self, primary_release_year: u64)-> &mut MovieSearch<'a>{
        self.primary_release_year = Some(primary_release_year);
        self
    }



    pub fn request(&self) -> Result<SearchResult<SearchMovie>, Error>{

        let mut parameters = format!("api_key={}&query={}&page={}&language={}", self.api_key, self.query, self.page, self.language);

        if let Some(region) = self.region{
            parameters += "&region=";
            parameters += region;
        }

        if let Some(include_adult) = self.include_adult{
            parameters += "&include_adult=";
            parameters += &include_adult.to_string();
        }

        if let Some(year) = self.year{
            parameters += "&year=";
            parameters += &year.to_string();
        }

        if let Some(primary_release_year) = self.primary_release_year{
            parameters += "&primary_release_year=";
            parameters += &primary_release_year.to_string();
        }

        let body = match reqwest::blocking::get(format!("https://api.themoviedb.org/3/search/movie?{}",parameters)){
            Ok(body) => body,
            Err(e) => return Err(Error::from_reqwest(e, &format!("tmdb.SearchMovie({})", self.query)))
        };
        if body.status().is_success(){
            match body.json(){
                Ok(movie) => return Ok(movie),
                Err(e) => return Err(Error::from_reqwest(e, &format!("tmdb.SearchMovie({}) parse body",  self.query))),
            };
        }
        let e: ErrorModel = match body.json(){
            Ok(e) => e,
            Err(e) => return Err(Error::from_reqwest(e, &format!("tmdb.SearchMovie({}) parse error",  self.query))),
        };
        Err(Error::new(ErrorKind::Tmdb, e.status_message, &format!("tmdb.SearchMovie({})",  self.query)))
    }
}
