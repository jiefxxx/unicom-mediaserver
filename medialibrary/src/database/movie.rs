use std::collections::HashMap;

use crate::library::cast::Cast;
use crate::library::cast::Crew;
use crate::library::genre::Genre;
use crate::library::keyword::Keyword;
use crate::library::trailer::Trailer;
use crate::rustmdb;
use super::Error;
use super::SqlLibrary;
use super::generate_sql;
use super::parse_concat;
use super::parse_watched;
use crate::library::movie::{MovieResult, Movie};


impl SqlLibrary{
    pub fn create_movie(&self, movie: &rustmdb::model::Movie) -> Result<(Vec<u64>, Vec<String>), Error>{
        let mut m_conn = self.conn.lock().unwrap();
        let conn = m_conn.as_mut().unwrap();
        let tx = conn.transaction()?;

        let mut person_ids = Vec::new();
        let mut rsc_path = Vec::new();

        tx.execute(
            "INSERT OR REPLACE INTO Movies (
                id,
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
                adult,
                updated) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, datetime('now'))",

            &[
                &movie.id.to_string(),
                &movie.original_title,
                &movie.original_language,
                &movie.title,
                &movie.release_date,
                &movie.overview.as_ref().unwrap_or(&"".to_string()),
                &movie.popularity.to_string(),
                &movie.poster_path.as_ref().unwrap_or(&"".to_string()),
                &movie.backdrop_path.as_ref().unwrap_or(&"".to_string()),
                &movie.vote_average.to_string(),
                &movie.vote_count.to_string(),
                &movie.tagline.as_ref().unwrap_or(&"".to_string()),
                &movie.status,
                &movie.adult.to_string()],
        )?;

        if let Some(backdrop_path) = &movie.backdrop_path{
            rsc_path.push(backdrop_path.clone())
        }

        if let Some(poster_path) = &movie.poster_path{
            rsc_path.push(poster_path.clone())
        }

        for genre in &movie.genres{
            tx.execute(
                "INSERT OR REPLACE INTO MovieGenres (
                    id,
                    name) values (?1, ?2)",
    
                &[
                &genre.id.to_string(),
                &genre.name],
            )?;

            tx.execute(
                "INSERT OR REPLACE INTO MovieGenreLinks (
                    genre_id,
                    movie_id) values (?1, ?2)",
    
                &[
                &genre.id.to_string(),
                &movie.id.to_string()],
            )?;
        }

        for cast in &movie.credits.cast{
            tx.execute(
                "INSERT OR REPLACE INTO MovieCasts (
                    person_id,
                    movie_id,
                    character,
                    ord) values (?1, ?2, ?3, ?4)",
    
                &[
                &cast.id.to_string(),
                &movie.id.to_string(),
                &cast.character.as_ref().unwrap_or(&"".to_string()), 
                &cast.order.to_string()],
            )?;

            person_ids.push(cast.id)
        }

        for crew in &movie.credits.crew{
            if !(crew.job == "Screenplay" ||  crew.job == "Director" || crew.job == "Producer"){
                continue
            }
    
            tx.execute(
                "INSERT OR REPLACE INTO MovieCrews (
                    person_id,
                    movie_id,
                    job) values (?1, ?2, ?3)",
    
                &[
                &crew.id.to_string(),
                &movie.id.to_string(), 
                &crew.job.to_string()],
            )?;

            person_ids.push(crew.id)
        }

        for video in &movie.videos.results{
            if video.site != "YouTube"{
                continue
            }
            tx.execute(
                "INSERT OR REPLACE INTO MovieTrailers (
                    movie_id,
                    name,
                    youtube_id) values (?1, ?2, ?3)",
    
                &[
                &movie.id.to_string(),
                &video.name,
                &video.key],
            )?;
        }

        for keyword in &movie.keywords.keywords{

            tx.execute(
                "INSERT OR REPLACE INTO movieKeywordLinks (
                    keyword_id,
                    movie_id) values (?1, ?2)",
    
                &[
                &keyword.id.to_string(),
                &movie.id.to_string()],
            )?;
    
            tx.execute(
                "INSERT OR IGNORE INTO Keywords (
                    id,
                    name) values (?1, ?2)",
    
                &[
                &keyword.id.to_string(),
                &keyword.name],
            )?;
        }

        tx.commit()?;

        Ok((person_ids, rsc_path))
    }

    pub fn get_movie(&self, user: &String, movie_id: u64) -> Result<Option<Movie>, Error>{
        let sql = "SELECT
                        id,
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
                        genres,
                        adding,
                        MovieUserWatched.watched,
                        updated
                        FROM MoviesView
                        LEFT OUTER JOIN MovieUserWatched ON MoviesView.id = MovieUserWatched.movie_id AND MovieUserWatched.user_name = ?1
                        WHERE id = ?2
                        GROUP BY MoviesView.id";
        let m_conn = self.conn.lock().unwrap();
        let conn = m_conn.as_ref().unwrap();
        let mut stmt = conn.prepare(&sql)?;
        
        let rows = stmt.query_map(&[user, &movie_id.to_string()], |row| {

            Ok(Movie{ 
                user: user.clone(),
                id: row.get(0)?, 
                original_title: row.get(1)?, 
                original_language: row.get(2)?, 
                title: row.get(3)?, 
                release_date: row.get(4)?, 
                overview: row.get(5)?, 
                popularity: row.get(6)?, 
                poster_path: row.get(7)?, 
                backdrop_path: row.get(8)?, 
                vote_average: row.get(9)?, 
                vote_count: row.get(10)?, 
                tagline: row.get(11)?, 
                status: row.get(12)?, 
                genres: parse_concat(row.get(13)?).unwrap_or_default(), 
                adding: row.get(14)?,
                watched: parse_watched(row.get(15)?),  
                updated: row.get(16)?,
                video: Vec::new(),
                cast: Vec::new(),
                crew: Vec::new(),
                trailer: Vec::new(),
                keyword: Vec::new(),
                collection: Vec::new(),

            })
        })?;

        for row in rows{
            return Ok(Some(row?));
        }

        Ok(None)
    }

    pub fn get_movies(&self, user: &String, parameters: &HashMap<String, Option<(String, String)>>, 
                order_by: &Option<String>, limit: Option<u64>, offset: Option<u64>) -> Result<Vec<MovieResult>, Error>{
        let (sql, param) = generate_sql("SELECT 
                                                    Movies.id, 
                                                    Movies.title, 
                                                    Movies.release_date, 
                                                    Movies.poster_path, 
                                                    Movies.vote_average,
                                                    GROUP_CONCAT(DISTINCT MovieGenres.name),
                                                    MAX(Videos.adding),
                                                    MovieUserWatched.watched,
                                                    Movies.backdrop_path
                                                FROM Movies
                                                INNER JOIN Videos ON Movies.id = Videos.media_id AND Videos.media_type = 0
                                                LEFT OUTER JOIN MovieGenreLinks ON Movies.id = MovieGenreLinks.movie_id
                                                LEFT OUTER JOIN MovieGenres ON MovieGenreLinks.genre_id = MovieGenres.id 
                                                LEFT OUTER JOIN MovieCasts ON Movies.id = MovieCasts.movie_id
                                                LEFT OUTER JOIN MovieCrews ON Movies.id = MovieCrews.movie_id
                                                LEFT OUTER JOIN MovieCollectionLinks ON Movies.id = MovieCollectionLinks.movie_id
                                                LEFT OUTER JOIN MovieUserWatched ON Movies.id = MovieUserWatched.movie_id AND MovieUserWatched.user_name = ?1", 
                                                &parameters, Some(user), Some("Movies.id"), order_by, limit, offset);
        // println!("sql: {}", &sql);
        let m_conn = self.conn.lock().unwrap();
        let conn = m_conn.as_ref().unwrap();
        let mut stmt = conn.prepare(&sql)?;
    
        let rows = stmt.query_map(param.as_slice(), |row| {
            Ok(MovieResult{ 
                user: user.clone(),
                id: row.get(0)?, 
                title: row.get(1)?, 
                release_date: row.get(2)?, 
                poster_path: row.get(3)?, 
                vote_average: row.get(4)?, 
                genres: parse_concat(row.get(5)?).unwrap_or_default(), 
                adding: row.get(6)?,
                watched: parse_watched(row.get(7)?),
                backdrop_path: row.get(8)?,
            })
            
        })?;

        let mut result = Vec::new();
        for row in rows{
            result.push(row?);
        }
        Ok(result)
    }

    pub fn get_movie_cast(&self, user: &String, movie_id: u64) -> Result<Vec<Cast>, Error>{
        let sql = "SELECT
                            id,
                            character,
                            ord,
                            name,
                            profile_path
                        FROM MovieCastsView
                        WHERE movie_id = ?";
        //println!("sql: {}", &sql);
        let m_conn = self.conn.lock().unwrap();
        let conn = m_conn.as_ref().unwrap();
        let mut stmt = conn.prepare(&sql)?;
    
        let rows = stmt.query_map(&[&movie_id.to_string()], |row| {
            Ok(Cast{
                user: user.clone(),
                id: row.get(0)?,
                character: row.get(1)?,
                ord: row.get(2)?,
                name: row.get(3)?,
                profile_path: row.get(4)?,
            })
            
        })?;

        let mut result = Vec::new();
        for row in rows{
            result.push(row?);
        }
        Ok(result)
    }

    pub fn get_movie_crew(&self, user: &String, movie_id: u64) -> Result<Vec<Crew>, Error>{
        let sql = "SELECT
                            id,
                            job,
                            name,
                            profile_path
                        FROM MovieCrewsView
                        WHERE movie_id = ?";
        //println!("sql: {}", &sql);
        let m_conn = self.conn.lock().unwrap();
        let conn = m_conn.as_ref().unwrap();
        let mut stmt = conn.prepare(&sql)?;
    
        let rows = stmt.query_map(&[&movie_id.to_string()], |row| {
            Ok(Crew{
                user: user.clone(),
                id: row.get(0)?,
                job: row.get(1)?,
                name: row.get(2)?,
                profile_path: row.get(3)?,
            })
            
        })?;

        let mut result = Vec::new();
        for row in rows{
            result.push(row?);
        }
        Ok(result)
    }

    pub fn get_movie_trailer(&self, movie_id: u64) -> Result<Vec<Trailer>, Error>{
        let sql = "SELECT
                            name,
                            youtube_id
                        FROM MovieTrailers
                        WHERE movie_id = ?";
        // println!("sql: {}", &sql);

        let m_conn = self.conn.lock().unwrap();
        let conn = m_conn.as_ref().unwrap();
        let mut stmt = conn.prepare(&sql)?;
    
        let rows = stmt.query_map(&[&movie_id.to_string()], |row| {
            Ok(Trailer{
                name: row.get(0)?,
                youtube_id: row.get(1)?,
            })
            
        })?;

        let mut result = Vec::new();
        for row in rows{
            result.push(row?);
        }
        Ok(result)
    }

    pub fn get_movie_keywords(&self, movie_id: u64) -> Result<Vec<Keyword>, Error>{
        let sql = "SELECT
                            name,
                            id
                        FROM MovieKeywordLinks
                        INNER JOIN Keywords ON MovieKeywordLinks.keyword_id = Keywords.id
                        WHERE movie_id = ?";
        //println!("sql: {}", &sql);
        let m_conn = self.conn.lock().unwrap();
        let conn = m_conn.as_ref().unwrap();
        let mut stmt = conn.prepare(&sql)?;
    
        let rows = stmt.query_map(&[&movie_id.to_string()], |row| {
            Ok(Keyword{
                name: row.get(0)?,
                id: row.get(1)?,
            })
            
        })?;

        let mut result = Vec::new();
        for row in rows{
            result.push(row?);
        }
        Ok(result)
    }

    pub fn genre_movie(&self) -> Result<Vec<Genre>, Error>{
        let sql = "SELECT
                            name,
                            id
                        FROM MovieGenres";
        //println!("sql: {}", &sql);
        let m_conn = self.conn.lock().unwrap();
        let conn = m_conn.as_ref().unwrap();
        let mut stmt = conn.prepare(&sql)?;
    
        let rows = stmt.query_map([], |row| {
            Ok(Genre{
                name: row.get(0)?,
                id: row.get(1)?,
            })
            
        })?;

        let mut result = Vec::new();
        for row in rows{
            result.push(row?);
        }
        Ok(result)
    }

    pub fn set_movie_watched(&self, user: String, movie_id: u64, watched: u64) -> Result< (), Error>{
        let m_conn = self.conn.lock().unwrap();
        let conn = m_conn.as_ref().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO MovieUserWatched (
                watched,
                user_name,
                movie_id) values (?1, ?2, ?3)",
            &[
                &watched.to_string(),
                &user,
                &movie_id.to_string()],
        )?;
        Ok(())
    }

    pub fn delete_movie(&self, movie_id: u64) -> Result<(), Error>{
        let mut m_conn = self.conn.lock().unwrap();
        let conn = m_conn.as_mut().unwrap();
        let tx = conn.transaction()?;

        tx.execute("DELETE FROM Movies
                        WHERE id=?1", &[&movie_id.to_string()])?;
        
        tx.execute("DELETE FROM MovieGenreLinks
                        WHERE movie_id=?1", &[&movie_id.to_string()])?;
        
        tx.execute("DELETE FROM MovieCollectionLinks
                        WHERE movie_id=?1", &[&movie_id.to_string()])?;

        tx.execute("DELETE FROM MovieKeywordLinks
                        WHERE movie_id=?1", &[&movie_id.to_string()])?;

        tx.execute("DELETE FROM MovieTrailers
                        WHERE movie_id=?1", &[&movie_id.to_string()])?;

        tx.execute("DELETE FROM MovieCasts
                        WHERE movie_id=?1", &[&movie_id.to_string()])?;

        tx.execute("DELETE FROM MovieCrews
                        WHERE movie_id=?1", &[&movie_id.to_string()])?;
        
        tx.execute("DELETE FROM MovieUserWatched
                        WHERE movie_id=?1", &[&movie_id.to_string()])?;

        tx.commit()?;
        
        Ok(())
    }
}