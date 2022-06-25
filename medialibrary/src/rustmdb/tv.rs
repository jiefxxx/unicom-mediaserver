use super::{Error, ErrorKind, TMDBKEY, LANGUAGE, model::{SearchResult, SearchTv, ErrorModel}};

pub struct TvSearch <'a>{
    api_key: String,
    language: String,
    query: &'a str,
    page: u64,
    include_adult: Option<bool>,
    first_air_date_year: Option<u64>
}

impl <'a>TvSearch<'a> {
    pub fn new(query: &'a str) -> TvSearch<'a>{
        TvSearch{
            api_key: TMDBKEY.lock().unwrap().to_string(),
            language: LANGUAGE.lock().unwrap().to_string(),
            query,
            page: 1,
            include_adult: None,
            first_air_date_year: None,
        }
    }
    #[allow(dead_code)]
    pub fn page(&mut self, page: u64) -> &mut TvSearch<'a>{
        self.page = page;
        self
    }

    #[allow(dead_code)]
    pub fn language(&mut self, language: &str) -> &mut TvSearch<'a>{
        self.language = language.to_string();
        self
    }

    #[allow(dead_code)]
    pub fn include_adult(&mut self, include_adult: bool) -> &mut TvSearch<'a>{
        self.include_adult = Some(include_adult);
        self
    }

    #[allow(dead_code)]
    pub fn request(&self) -> Result<SearchResult<SearchTv>, Error>{

        let mut parameters = format!("api_key={}&query={}&page={}&language={}", self.api_key, self.query, self.page, self.language);

        if let Some(include_adult) = self.include_adult{
            parameters += "&include_adult=";
            parameters += &include_adult.to_string();
        }

        if let Some(first_air_date_year) = self.first_air_date_year{
            parameters += "&first_air_date_year=";
            parameters += &first_air_date_year.to_string();
        }
        let body = match reqwest::blocking::get(format!("https://api.themoviedb.org/3/search/tv?{}",parameters)){
            Ok(body) => body,
            Err(e) => return Err(Error::from_reqwest(e, &format!("tmdb.SearchTv({})", self.query)))
        };
        if body.status().is_success(){
            match body.json(){
                Ok(movie) => return Ok(movie),
                Err(e) => return Err(Error::from_reqwest(e, &format!("tmdb.SearchTv({}) parse body",  self.query))),
            };
        }
        let e: ErrorModel = match body.json(){
            Ok(e) => e,
            Err(e) => return Err(Error::from_reqwest(e, &format!("tmdb.SearchTv({}) parse error",  self.query))),
        };
        Err(Error::new(ErrorKind::Tmdb, e.status_message, &format!("tmdb.SearchTv({})",  self.query)))
    }
}