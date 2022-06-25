use std::{fmt, collections::HashMap, str::FromStr, sync::{Mutex, Arc}};

use rusqlite::{Connection, ToSql};

mod video;
mod movie;
mod tv;
mod person;
mod collection;

lazy_static! {
    pub static ref DATABASE: Arc<SqlLibrary> = Arc::new(SqlLibrary::new());
}

#[derive(Debug)]
pub struct SqlLibrary{
    conn: Mutex<Option<Connection>>,
}

impl SqlLibrary{
    pub fn new() ->  SqlLibrary{
        SqlLibrary{
            conn: Mutex::new(None),
        }
    }

    pub fn connect(&self, path: &str){
        let mut conn = self.conn.lock().unwrap();
        *conn = Some(Connection::open(path).unwrap());
        drop(conn);
        self.init_db().unwrap();
        
    }

    //video part

    fn init_db(&self) -> Result<(), rusqlite::Error>{
        let m_conn = self.conn.lock().unwrap();
        let conn = m_conn.as_ref().unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS Videos (
                id INTEGER PRIMARY KEY NOT NULL,
                path TEXT NOT NULL UNIQUE,
                media_type INTEGER,
                media_id INTEGER,
                duration INTEGER,
                bit_rate INTEGER,
                codec TEXT,
                width INTEGER,
                height INTEGER,
                size INTEGER,
                adding TEXT)",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS WatchTimes (
                video_id INTEGER NOT NULL,
                user_name INTEGER NOT NULL,
                watch_time INTEGER,
                last_watch TEXT,
                unique(video_id, user_name))",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS Audios (
                video_id INTEGER NOT NULL,
                language TEXT,
                unique(video_id, language))",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS Subtitles (
                video_id INTEGER NOT NULL,
                language TEXT,
                unique(video_id, language))",
            [],
        )?;

        conn.execute("DROP VIEW IF EXISTS VideosView",[])?;
        conn.execute(
            "CREATE VIEW VideosView
                AS 
                SELECT
                    Videos.id as id,
                    path,
                    media_type,
                    media_id,
                    duration,
                    bit_rate,
                    codec,
                    width,
                    height,
                    Movies.id as m_id,
                    Tvs.id as t_id,
                    Movies.title as m_title,
                    Tvs.title as t_title,
                    episode_number,
                    season_number,
                    Movies.release_date as release_date,
                    size,
                    adding,
                    GROUP_CONCAT(Subtitles.language) as subtitles,
                    GROUP_CONCAT(Audios.language) as audios
                FROM
                    Videos
                LEFT OUTER JOIN Audios ON Videos.id = Audios.video_id
                LEFT OUTER JOIN Subtitles ON Videos.id = Subtitles.video_id
                LEFT OUTER JOIN Movies ON Videos.media_type = 0 AND Videos.media_id = Movies.id
                LEFT OUTER JOIN Episodes ON Videos.media_type = 1 AND Videos.media_id = Episodes.id
                LEFT OUTER JOIN Tvs ON Episodes.tv_id = Tvs.id
                GROUP BY videos.id",
                []
        )?;

        // Movie Part

        conn.execute(
            "CREATE TABLE IF NOT EXISTS Movies (
                id INTEGER PRIMARY KEY NOT NULL,
                original_title TEXT,
                original_language TEXT,
                title TEXT,
                release_date TEXT,
                overview TEXT,
                popularity FLOAT,
                poster_path TEXT,
                backdrop_path TEXT,
                vote_average FLOAT,
                vote_count INTEGER,
                tagline TEXT,
                status TEXT,
                adult BOOL,
                updated TEXT)",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS MovieGenres (
                id INTEGER PRIMARY KEY NOT NULL,
                name TEXT)",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS MovieGenreLinks (
                movie_id INTEGER NOT NULL,
                genre_id INTEGER NOT NULL,
                unique(movie_id,genre_id))",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS MovieCollectionLinks (
                movie_id INTEGER NOT NULL,
                collection_id INTEGER NOT NULL,
                unique(movie_id, collection_id))",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS MovieKeywordLinks (
                movie_id INTEGER NOT NULL,
                keyword_id INTEGER NOT NULL,
                unique(movie_id,keyword_id))",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS MovieTrailers (
                movie_id INTEGER NOT NULL,
                name TEXT,
                youtube_id TEXT,
                unique(movie_id,youtube_id))",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS MovieCasts (
                movie_id INTEGER NOT NULL,
                person_id TEXT,
                character TEXT,
                ord INTEGER,
                unique(movie_id,person_id,character))",
            [],
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS MovieCrews (
                movie_id INTEGER NOT NULL,
                person_id TEXT,
                job TEXT,
                unique(movie_id,person_id,job))",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS MovieUserWatched (
                movie_id INTEGER NOT NULL,
                user_name TEXT,
                watched INTEGER,
                vote_user FLOAT,
                unique(movie_id,user_name))",
            [],
        )?;

        conn.execute("DROP VIEW IF EXISTS MoviesView",[])?;
        conn.execute(
            "CREATE VIEW MoviesView
                AS 
                SELECT
                    Movies.id as id,
                    original_title,
                    original_language,
                    title,
                    release_date,
                    overview,
                    popularity,
                    poster_path,
                    backdrop_path,
                    vote_average,
                    vote_count,
                    tagline,
                    status,
                    updated,
                    GROUP_CONCAT(DISTINCT MovieGenres.name) as genres,
                    MAX(Videos.adding) as adding
                FROM
                    Movies
                INNER JOIN Videos ON Movies.id = Videos.media_id AND Videos.media_type = 0
                LEFT OUTER JOIN MovieGenreLinks ON Movies.id = MovieGenreLinks.movie_id
                LEFT OUTER JOIN MovieGenres ON MovieGenreLinks.genre_id = MovieGenres.id
                 

                GROUP BY Movies.id",
                []
        )?;

        conn.execute("DROP VIEW IF EXISTS MovieCastsView",[])?;
        conn.execute(
            "CREATE VIEW MovieCastsView
                AS 
                SELECT
                    Persons.id as id,
                    character,
                    movie_id,
                    ord,
                    name,
                    profile_path
                FROM
                    MovieCasts
                LEFT OUTER JOIN Persons ON MovieCasts.person_id = Persons.id

                ",
                []
        )?;

        conn.execute("DROP VIEW IF EXISTS MovieCrewsView",[])?;
        conn.execute(
            "CREATE VIEW MovieCrewsView
                AS 
                SELECT
                    Persons.id as id,
                    job,
                    movie_id,
                    name,
                    profile_path
                FROM
                    MovieCrews
                LEFT OUTER JOIN Persons ON MovieCrews.person_id = Persons.id

                ",
                []
        )?;

        // Tv Part

        conn.execute(
            "CREATE TABLE IF NOT EXISTS Tvs (
                id INTEGER PRIMARY KEY NOT NULL,
                original_title TEXT,
                original_language TEXT,
                title TEXT,
                release_date TEXT,
                overview TEXT,
                popularity FLOAT,
                poster_path TEXT,
                backdrop_path TEXT,
                status TEXT,
                vote_average FLOAT,
                vote_count INTEGER,
                in_production BOOL, 
                number_of_episodes INTEGER,
                number_of_seasons INTEGER,
                episode_run_time INTEGER,
                updated TEXT)",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS TvGenres (
                id INTEGER PRIMARY KEY NOT NULL,
                name TEXT)",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS TvGenreLinks (
                tv_id INTEGER NOT NULL,
                genre_id INTEGER NOT NULL,
                unique(tv_id,genre_id))",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS TvCollectionLinks (
                tv_id INTEGER NOT NULL,
                collection_id INTEGER NOT NULL,
                unique(tv_id, collection_id))",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS TvKeywordLinks (
                tv_id INTEGER NOT NULL,
                keyword_id INTEGER NOT NULL,
                unique(tv_id,keyword_id))",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS TvTrailers (
                tv_id INTEGER NOT NULL,
                name TEXT,
                youtube_id TEXT,
                unique(tv_id,youtube_id))",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS TvCasts (
                tv_id INTEGER NOT NULL,
                person_id TEXT,
                character TEXT,
                ord INTEGER,
                unique(tv_id, person_id, character))",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS TvCrews (
                tv_id INTEGER NOT NULL,
                person_id TEXT,
                job TEXT,
                unique(tv_id, person_id, job))",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS Seasons (
                id INTEGER PRIMARY KEY NOT NULL,
                tv_id INTEGER NOT NULL,
                season_number INTEGER NOT NULL,
                episode_count INTEGER,
                title TEXT,
                overview TEXT,
                poster_path TEXT,
                release_date TEXT,
                updated TEXT)",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS Episodes (
                id INTEGER PRIMARY KEY NOT NULL,
                season_id INTEGER NOT NULL,
                tv_id INTEGER NOT NULL,
                season_number INTEGER NOT NULL,
                episode_number INTEGER NOT NULL,
                release_date TEXT,
                title TEXT,
                overview TEXT,
                vote_average FLOAT,
                vote_count INTEGER,
                updated TEXT)",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS EpisodeCasts (
                episode_id INTEGER NOT NULL,
                person_id INTEGER NOT NULL,
                character TEXT,
                ord INTEGER,
                unique(episode_id,person_id,character))",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS EpisodeCrews (
                episode_id INTEGER NOT NULL,
                person_id INTEGER NOT NULL,
                job TEXT,
                unique(episode_id,person_id,job))",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS EpisodesUserWatched (
                episode_id INTEGER NOT NULL,
                user_name TEXT,
                watched INTEGER,
                vote_user FLOAT,
                unique(episode_id,user_name))",
            [],
        )?;

        conn.execute("DROP VIEW IF EXISTS TvsView",[])?;
        conn.execute(
            "CREATE VIEW IF NOT EXISTS TvsView
                AS 
                SELECT
                    Tvs.id as id,
                    Tvs.original_title as original_title,
                    original_language,
                    Tvs.title as title,
                    Tvs.release_date as release_date,
                    Tvs.overview as overview,
                    popularity,
                    poster_path,
                    backdrop_path,
                    status,
                    Tvs.vote_average as vote_average,
                    Tvs.vote_count as vote_count,
                    number_of_episodes,
                    number_of_seasons,
                    episode_run_time,
                    Tvs.updated as updated,
                    GROUP_CONCAT(DISTINCT TvGenres.name) as genres,
                    MAX(Videos.adding) as adding
                FROM
                    Tvs
                LEFT OUTER JOIN TvGenreLinks ON Tvs.id = TvGenreLinks.tv_id
                LEFT OUTER JOIN TvGenres ON TvGenreLinks.genre_id = TvGenres.id
                LEFT OUTER JOIN Episodes ON Tvs.id = Episodes.tv_id
                INNER JOIN Videos ON Videos.media_id = Episodes.id AND Videos.media_type = 1

                GROUP BY Tvs.id",
                []
        )?;
        conn.execute("DROP VIEW IF EXISTS SeasonsView",[])?;
        conn.execute(
            "CREATE VIEW SeasonsView
                AS 
                SELECT
                    Seasons.id as id,
                    Seasons.tv_id as tv_id,
                    Seasons.season_number as season_number,
                    Seasons.episode_count as episode_count,
                    Seasons.title as title,
                    Seasons.overview as overview,
                    Seasons.poster_path as poster_path,
                    Seasons.release_date as release_date,
                    Seasons.updated as updated
                FROM
                    Seasons
                INNER JOIN Episodes ON Episodes.season_id = Seasons.id
                INNER JOIN Videos ON Videos.media_id = Episodes.id AND Videos.media_type = 1

                GROUP BY Seasons.id",
                []
        )?;

        conn.execute("DROP VIEW IF EXISTS EpisodesView",[])?;
        conn.execute(
            "CREATE VIEW EpisodesView
                AS 
                SELECT
                    Episodes.id as id,
                    tv_id,
                    season_number,
                    episode_number,
                    release_date,
                    title,
                    overview,
                    vote_average,
                    vote_count,
                    updated
                FROM
                    Episodes
                INNER JOIN Videos ON Videos.media_id = Episodes.id AND Videos.media_type = 1
                
                GROUP BY Videos.id",
                []
        )?;

        conn.execute("DROP VIEW IF EXISTS TvCastsView",[])?;
        conn.execute(
            "CREATE VIEW IF NOT EXISTS TvCastsView
                AS 
                SELECT
                    Persons.id as id,
                    character,
                    tv_id,
                    ord,
                    name,
                    profile_path
                FROM
                    TvCasts
                LEFT OUTER JOIN Persons ON TvCasts.person_id = Persons.id

                ",
                []
        )?;
        
        conn.execute("DROP VIEW IF EXISTS TvCrewsView",[])?;
        conn.execute(
            "CREATE VIEW IF NOT EXISTS TvCrewsView
                AS 
                SELECT
                    Persons.id as id,
                    tv_id,
                    job,
                    name,
                    profile_path
                FROM
                    TvCrews
                LEFT OUTER JOIN Persons ON TvCrews.person_id = Persons.id

                ",
                []
        )?;

        conn.execute("DROP VIEW IF EXISTS EpisodeCastsView",[])?;
        conn.execute(
            "CREATE VIEW IF NOT EXISTS EpisodeCastsView
                AS 
                SELECT
                    Persons.id as id,
                    character,
                    episode_id,
                    ord,
                    name,
                    profile_path
                FROM
                    EpisodeCasts
                LEFT OUTER JOIN Persons ON EpisodeCasts.person_id = Persons.id

                ",
                []
        )?;
        
        conn.execute("DROP VIEW IF EXISTS EpisodeCrewsView",[])?;
        conn.execute(
            "CREATE VIEW IF NOT EXISTS EpisodeCrewsView
                AS 
                SELECT
                    Persons.id as id,
                    episode_id,
                    job,
                    name,
                    profile_path
                FROM
                    EpisodeCrews
                LEFT OUTER JOIN Persons ON EpisodeCrews.person_id = Persons.id

                ",
                []
        )?;


        //Person Part
        conn.execute(
            "CREATE TABLE IF NOT EXISTS Persons (
                id INTEGER PRIMARY KEY NOT NULL,
                birthday TEXT,
                known_for_department TEXT,
                deathday TEXT,
                name TEXT,
                gender INTEGER,
                biography TEXT,
                popularity FLOAT,
                place_of_birth TEXT,
                profile_path TEXT)",
            []
        )?;

        //keywords
        conn.execute(
            "CREATE TABLE IF NOT EXISTS Keywords (
                id INTEGER PRIMARY KEY NOT NULL,
                name TEXT)",
            [],
        )?;

        //user

        //Person Part
        conn.execute(
            "CREATE TABLE IF NOT EXISTS Collections (
                id INTEGER PRIMARY KEY NOT NULL,
                name TEXT,
                description TEXT,
                creator TEXT,
                creation_date TEXT,
                poster_path TEXT,
                unique(name, creator))",
            []
        )?;
        

        Ok(())
    }
}

pub fn parse_concat<T: FromStr>( row: Option<String>) -> Option<Vec<T>>{
    if let Some(row) = row{
        return Some(row.split(",").map(|s| {
            let data = s.parse();
            match data {
                Ok(data) => data,
                Err(_) => todo!(),
            }
        }).collect())
    }
    None
}

pub fn parse_watched(row: Option<u64>) -> u64{
    if let Some(value) =  row{
        return value
    }
    return 0
}

pub fn generate_sql<'a>(head: &str, parameters: &'a HashMap<String, Option<(String, String)>>, user: Option<&'a String>,
                group_by: Option<&'a str>, order_by: &'a Option<String>, limit: Option<u64>, offset: Option<u64>) -> (String, Vec<&'a dyn ToSql>){
    let mut param :Vec<&dyn ToSql> = Vec::new();
    let mut sql = head.to_string();
    if let Some(user) = user{
        param.push(user);
    }
    if parameters.len() > 0{
        sql += " WHERE ";
        let mut counter = 1;
        for (name, value) in parameters{
            if counter > 1{
                sql += "AND "
            }
            
            if let Some((operator, value)) = value{
                param.push(value);
                sql += &format!("{} {} ?{} ", name, operator, &param.len());
                
            }
            else{
                sql += &format!("{} IS NULL ", name);
            }
            
            counter += 1;
        }
        sql += "\n"
    }

    if let Some(group_by) = group_by{
        sql += &format!("GROUP BY {} \n", group_by);
    }

    if let Some(order_by) = order_by{
        sql += &format!("ORDER BY {} \n", order_by);
    }

    if let Some(limit) = limit{
        sql += &format!("LIMIT {}", limit);
        if let Some(offset) = offset{
            sql += &format!(" OFFSET {} \n", offset)
        }
        else{
            sql += "\n"
        }
    }


    //sql += ";";

    // println!("sql: {}", &sql);
    (sql, param)
}

#[derive(Debug)]
pub enum ErrorKind{
    Unknwon,
}

#[derive(Debug)]
pub struct Error{
    kind: ErrorKind,
    description: String,
    location: String,
}


impl Error{
    pub fn new(kind: ErrorKind, description: String, location: &str) -> Error{
        Error{
            kind,
            description,
            location: location.to_string(), 
        }
    }

    pub fn from_reqwest(e: rusqlite::Error, location: &str) -> Error{
        Error::new(ErrorKind::Unknwon, e.to_string(), location)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error: {:?} at {} {}", &self.kind, &self.location, &self.description)
    }
}

impl std::convert::From<rusqlite::Error> for Error {
    fn from(err: rusqlite::Error) -> Error {
        Error::from_reqwest(err, "Undefined")
    }
}