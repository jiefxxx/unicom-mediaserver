extern crate reqwest;
extern crate serde;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct ErrorModel{
    pub status_code: u64,
    pub status_message: String,
}


#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Person {
    pub birthday: Option<String>,
    pub known_for_department: Option<String>,
    pub deathday: Option<String>,
    pub id: u64,
    pub name: String,
    pub also_known_as: Vec<String>,
    pub gender: u8,
    pub biography: String,
    pub popularity: f64,
    pub place_of_birth: Option<String>,
    pub profile_path:  Option<String>,
    pub adult: bool,
    pub imdb_id: Option<String>,
    pub homepage: Option<String>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct SearchResult<T>{
    pub page: u64,
    pub total_results: u64,
    pub total_pages: u64,
    pub results: Vec<T>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct SearchMovie {
    pub id: u64,
    pub title: String,
    pub original_title: String,
    pub original_language: String,
    pub overview: Option<String>,
    pub release_date: Option<String>,
    pub genre_ids: Vec<u16>,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub popularity: f64,
    pub adult: bool,
    pub vote_count: u64,
    pub vote_average: f64,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Genre {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct ProductionCompanie {
    pub id: u64,
    pub name: String,
    pub logo_path: Option<String>,
    pub origin_country: String,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct ProductionCountrie {
    pub iso_3166_1: String,
    pub name: String,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Language {
    pub iso_639_1: String,
    pub name: String,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Cast {
    pub adult: bool,
    pub gender: Option<u8>,
    pub id: u64,
    pub known_for_department: Option<String>,
    pub name: String,
    pub original_name: String,
    pub popularity: f64,
    pub profile_path: Option<String>,
    pub cast_id: Option<u64>,
    pub character: Option<String>,
    pub credit_id: String,
    pub order: u64,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Crew {
    pub adult: bool,
    pub gender: Option<u8>,
    pub id: u64,
    pub known_for_department: Option<String>,
    pub name: String,
    pub original_name: String,
    pub popularity: f64,
    pub profile_path: Option<String>,
    pub credit_id: String,
    pub department: String,
    pub job: String,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Credits {
    pub cast: Vec<Cast>,
    pub crew: Vec<Crew>,
}
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Keyword {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct KeywordsMovie {
    pub keywords: Vec<Keyword>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct KeywordsTv {
    pub results: Vec<Keyword>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Video {
    pub iso_639_1: String,
    pub iso_3166_1: String,
    pub name: String,
    pub key: String,
    pub published_at: String,
    pub site: String,
    pub size: u64,
    pub _type: Option<String>,
    pub id: String,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Videos {
    pub results: Vec<Video>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Movie {
    pub id: u64,
    pub budget: u64,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub homepage: Option<String>,
    pub title: String,
    pub original_title: String,
    pub original_language: String,
    pub overview: Option<String>,
    pub release_date: String,
    pub popularity: f64,
    pub adult: bool,
    pub vote_count: u64,
    pub vote_average: f64,
    pub tagline: Option<String>,
    pub status: String,
    pub genres: Vec<Genre>,
    pub production_companies: Vec<ProductionCompanie>,
    pub production_countries: Vec<ProductionCountrie>,
    pub spoken_languages: Vec<Language>,
    pub credits: Credits,
    pub videos: Videos,
    pub keywords: KeywordsMovie,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct CreatedBy {
    pub gender: Option<u8>,
    pub id: u64,
    pub name: String,
    pub profile_path: Option<String>,
    pub credit_id: String,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct LastEpisodeToAir {
   pub air_date: String,
   pub episode_number: u64,
   pub id: u64,
   pub name: String,
   pub overview: String,
   pub production_code: String,
   pub season_number: u64,
   pub still_path: Option<String>,
   pub vote_average: f64,
   pub vote_count: u64,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Network {
   pub name: String,
   pub id: u64,
   pub logo_path: Option<String>,
   pub origin_country: String,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Season {
    pub air_date: Option<String>,
    pub episode_count: u64,
    pub name: String,
    pub id: u64,
    pub poster_path: Option<String>,
    pub season_number: u64,
    pub overview: Option<String>,
}


#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Tv {
    pub id: u64,
    pub backdrop_path: Option<String>,
    pub poster_path: Option<String>,
    pub episode_run_time: Vec<u64>,
    pub first_air_date: Option<String>,
    pub genres: Vec<Genre>,
    pub homepage: Option<String>,
    pub in_production: bool,
    pub languages: Vec<String>,
    pub last_air_date: Option<String>,
    pub name: String,
    pub number_of_episodes: u64,
    pub number_of_seasons: u64,
    pub origin_country: Vec<String>,
    pub original_language: String,
    pub original_name: String,
    pub overview: Option<String>,
    pub popularity: f64,
    pub production_companies: Vec<ProductionCompanie>,
    pub production_countries: Vec<ProductionCountrie>,
    pub spoken_languages: Vec<Language>,
    pub credits: Credits,
    pub status: String,
    pub tagline: String,
    pub vote_count: u64,
    pub vote_average: f64,
    pub created_by: Vec<CreatedBy>,
    pub last_episode_to_air: Option<LastEpisodeToAir>,
    pub networks: Vec<Network>,
    pub seasons: Vec<Season>,
    pub videos: Videos,
    pub keywords: KeywordsTv,
}


#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct GuestStar{
    pub id: u64,
    pub name: String,
    pub credit_id: String,
    pub character: String,
    pub order: u64, 
    pub profile_path: Option<String>, 
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct TvEpisode{
    pub air_date: String,
    pub guest_stars: Vec<GuestStar>,
    pub name: String,
    pub overview: Option<String>,
    pub id: u64,
    pub production_code: Option<String>,
    pub season_number: u64,
    pub episode_number: u64,
    pub still_path: Option<String>,
    pub vote_average: f64,
    pub vote_count: u64,
    pub credits: Credits,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct SearchTv {
    pub id: u64,
    pub name: String,
    pub original_name: String,
    pub original_language: String,
    pub original_country: Option<Vec<String>>,
    pub overview: Option<String>,
    pub first_air_date: Option<String>,
    pub genre_ids: Vec<u16>,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub popularity: f64,
    pub vote_count: u64,
    pub vote_average: f64,
}
