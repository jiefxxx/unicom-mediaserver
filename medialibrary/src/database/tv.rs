use std::collections::HashMap;

use crate::database::parse_watched;
use crate::library::cast::Cast;
use crate::library::cast::Crew;
use crate::library::genre::Genre;
use crate::library::keyword::Keyword;
use crate::library::trailer::Trailer;
use crate::library::tv::Episode;
use crate::library::tv::Tv;
use crate::library::tv::TvResult;
use crate::library::tv::Season;
use crate::rustmdb;
use super::Error;
use super::SqlLibrary;
use super::generate_sql;
use super::parse_concat;


impl SqlLibrary{
    pub fn create_tv(&self ,tv: &rustmdb::model::Tv) -> Result<(Vec<u64>, Vec<String>), Error>{

        let mut m_conn = self.conn.lock().unwrap();
        let conn = m_conn.as_mut().unwrap();
        let tx = conn.transaction()?;

        let mut person_ids = Vec::new();
        let mut rsc_path = Vec::new();
        // println!("adding tv {:?}", tv);
        tx.execute(
            "INSERT OR REPLACE INTO Tvs (
                id,
                original_title,
                original_language,
                title,
                release_date,
                overview,
                popularity,
                poster_path,
                backdrop_path,
                status,
                vote_average,
                vote_count,
                in_production, 
                number_of_episodes,
                number_of_seasons,
                episode_run_time,
                updated) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, datetime('now'))",

            &[
            &tv.id.to_string(),
            &tv.original_name,
            &tv.original_language,
            &tv.name,
            &tv.first_air_date.as_ref().unwrap_or(&"".to_string()),
            &tv.overview.as_ref().unwrap_or(&"".to_string()),
            &tv.popularity.to_string(),
            &tv.poster_path.as_ref().unwrap_or(&"".to_string()),
            &tv.backdrop_path.as_ref().unwrap_or(&"".to_string()),
            &tv.status,
            &tv.vote_average.to_string(),
            &tv.vote_count.to_string(),
            &tv.in_production.to_string(),
            &tv.number_of_episodes.to_string(),
            &tv.number_of_seasons.to_string(),
            &tv.episode_run_time.get(0).unwrap_or(&0).to_string()],
        )?;

        for season in &tv.seasons{
            // println!("season {}", season.season_number);
            tx.execute(
                "INSERT OR REPLACE INTO Seasons (
                    id,
                    tv_id,
                    season_number,
                    episode_count,
                    title,
                    overview,
                    poster_path,
                    release_date,
                    updated) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, datetime('now'))",
    
                &[
                &season.id.to_string(),
                &tv.id.to_string(),
                &season.season_number.to_string(),
                &season.episode_count.to_string(),
                &season.name,
                &season.overview.as_ref().unwrap_or(&"".to_string()),
                &season.poster_path.as_ref().unwrap_or(&"".to_string()),
                &season.air_date.as_ref().unwrap_or(&"".to_string()),],
            )?;
            
            if let Some(poster_path) = &season.poster_path{
                rsc_path.push(poster_path.clone())
            }
        }

        if let Some(backdrop_path) = &tv.backdrop_path{
            rsc_path.push(backdrop_path.clone())
        }

        if let Some(poster_path) = &tv.poster_path{
            rsc_path.push(poster_path.clone())
        }

        for genre in &tv.genres{
            tx.execute(
                "INSERT OR REPLACE INTO TvGenres (
                    id,
                    name) values (?1, ?2)",
    
                &[
                &genre.id.to_string(),
                &genre.name],
            )?;

            tx.execute(
                "INSERT OR REPLACE INTO TvGenreLinks (
                    genre_id,
                    tv_id) values (?1, ?2)",
    
                &[
                &genre.id.to_string(),
                &tv.id.to_string()],
            )?;
        }

        for cast in &tv.credits.cast{   
            tx.execute(
                "INSERT OR REPLACE INTO TvCasts (
                    person_id,
                    tv_id,
                    character,
                    ord) values (?1, ?2, ?3, ?4)",
    
                &[
                &cast.id.to_string(),
                &tv.id.to_string(),
                &cast.character.as_ref().unwrap_or(&"".to_string()), 
                &cast.order.to_string()],
            )?;

            person_ids.push(cast.id)
        }

        for crew in &tv.credits.crew{
            if !(crew.job == "Screenplay" ||  crew.job == "Director" || crew.job == "Producer"){
                continue
            }
    
            tx.execute(
                "INSERT OR REPLACE INTO TvCrews (
                    person_id,
                    tv_id,
                    job) values (?1, ?2, ?3)",
    
                &[
                &crew.id.to_string(),
                &tv.id.to_string(), 
                &crew.job.to_string()],
            )?;

            person_ids.push(crew.id)
        }

        for crew in &tv.created_by{
    
            tx.execute(
                "INSERT OR REPLACE INTO TvCrews (
                    person_id,
                    tv_id,
                    job) values (?1, ?2, ?3)",
    
                &[
                &crew.id.to_string(),
                &tv.id.to_string(), 
                "Creator"],
            )?;

            person_ids.push(crew.id)
        }

        for video in &tv.videos.results{
            if video.site != "YouTube"{
                continue
            }
            tx.execute(
                "INSERT OR REPLACE INTO TvTrailers (
                    tv_id,
                    name,
                    youtube_id) values (?1, ?2, ?3)",
    
                &[
                &tv.id.to_string(),
                &video.name,
                &video.key],
            )?;
        }

        for keyword in &tv.keywords.results{

            tx.execute(
                "INSERT OR REPLACE INTO TvKeywordLinks (
                    keyword_id,
                    tv_id) values (?1, ?2)",
    
                &[
                &keyword.id.to_string(),
                &tv.id.to_string()],
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

    pub fn create_episode(&self, tv_id: u64, episode: &rustmdb::model::TvEpisode) -> Result<(Vec<u64>, Vec<String>), Error>{
        let season_id = self.get_season_id(tv_id, episode.season_number).unwrap().unwrap();
        
        let mut m_conn = self.conn.lock().unwrap();
        let conn = m_conn.as_mut().unwrap();
        let tx = conn.transaction()?;

        let mut person_ids = Vec::new();
        let rsc_path = Vec::new();
        
        tx.execute(
            "INSERT OR REPLACE INTO Episodes (
                id,
                season_id,
                tv_id,
                season_number,
                episode_number,
                release_date,
                title,
                overview,
                vote_average,
                vote_count,
                updated) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, datetime('now'))",

            &[
            &episode.id.to_string(),
            &season_id.to_string(),
            &tv_id.to_string(),
            &episode.season_number.to_string(),
            &episode.episode_number.to_string(),
            &episode.air_date,
            &episode.name,
            &episode.overview.as_ref().unwrap_or(&"".to_string()),
            &episode.vote_average.to_string(),
            &episode.vote_count.to_string()],
        )?;

        for cast in &episode.credits.cast{
            tx.execute(
                "INSERT OR REPLACE INTO EpisodeCasts (
                    person_id,
                    episode_id,
                    character,
                    ord) values (?1, ?2, ?3, ?4)",
    
                &[
                &cast.id.to_string(),
                &episode.id.to_string(),
                &cast.character.as_ref().unwrap_or(&"".to_string()), 
                &cast.order.to_string()],
            )?;

            person_ids.push(cast.id)
        }

        for crew in &episode.credits.crew{
            if !(crew.job == "Screenplay" ||  crew.job == "Director" || crew.job == "Producer"){
                continue
            }
    
            tx.execute(
                "INSERT OR REPLACE INTO EpisodeCrews (
                    person_id,
                    episode_id,
                    job) values (?1, ?2, ?3)",
    
                &[
                &crew.id.to_string(),
                &episode.id.to_string(), 
                &crew.job.to_string()],
            )?;

            person_ids.push(crew.id)
        }

        tx.commit()?;

        Ok((person_ids, rsc_path))
    }

    pub fn get_season_id(&self, tv_id: u64, season_number: u64) -> Result<Option<u64>, Error> {
        // println!("get season id {} {}", &tv_id, &season_number);
        let m_conn = self.conn.lock().unwrap();
        let conn = m_conn.as_ref().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id from Seasons
             WHERE tv_id = ?1 and season_number = ?2",
        )?;
    
        let rows = stmt.query_map(&[&tv_id.to_string(), &season_number.to_string()], |row| row.get(0))?;
        for row in rows{
            return Ok(Some(row?))
        }
        Ok(None)
    }

    pub fn get_tv(&self, user: &String, tv_id: u64) -> Result<Option<Tv>, Error>{
        //println!("get_tv {:?}", &tv_id);
        let sql = "SELECT
                            TvsView.id, 
                            TvsView.original_title, 
                            TvsView.original_language, 
                            TvsView.title, 
                            TvsView.release_date, 
                            TvsView.overview, 
                            TvsView.popularity, 
                            TvsView.poster_path, 
                            TvsView.backdrop_path, 
                            TvsView.vote_average, 
                            TvsView.vote_count, 
                            TvsView.status, 
                            TvsView.genres, 
                            TvsView.number_of_episodes, 
                            TvsView.number_of_seasons, 
                            TvsView.episode_run_time, 
                            TvsView.adding,
                            MIN(COALESCE(EpisodesUserWatched.watched, 0)),
                            TvsView.updated
                        FROM TvsView
                        LEFT OUTER JOIN Episodes ON TvsView.id = Episodes.tv_id
                        LEFT OUTER JOIN EpisodesUserWatched ON Episodes.id = EpisodesUserWatched.episode_id AND EpisodesUserWatched.user_name = ?1
                        WHERE TvsView.id = ?2";

        let m_conn = self.conn.lock().unwrap();
        let conn = m_conn.as_ref().unwrap();
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(&[user, &tv_id.to_string()], |row| {
            Ok(Tv{ 
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
                status: row.get(11)?, 
                genres: parse_concat(row.get(12)?).unwrap_or_default(), 
                number_of_episodes: row.get(13)?, 
                number_of_seasons: row.get(14)?, 
                episode_run_time: row.get(15)?, 
                adding: row.get(16)?,
                watched: parse_watched(row.get(17)?),
                updated: row.get(18)?,
                seasons: Vec::new(),
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

    pub fn get_tvs(&self,user: &String, parameters: &HashMap<String, Option<(String, String)>>, 
                    order_by: &Option<String>, limit: Option<u64>, offset: Option<u64>) -> Result<Vec<TvResult>, Error>{
        let (sql, param) = generate_sql("SELECT 
                                                    Tvs.id,
                                                    Tvs.title,
                                                    Tvs.release_date,
                                                    Tvs.poster_path,
                                                    Tvs.vote_average,
                                                    GROUP_CONCAT(DISTINCT TvGenres.name),
                                                    MAX(Videos.adding),
                                                    MIN(COALESCE(EpisodesUserWatched.watched, 0)),
                                                    Tvs.backdrop_path
                                                FROM Tvs
                                                LEFT OUTER JOIN Episodes ON Tvs.id = Episodes.tv_id
                                                INNER JOIN Videos ON Videos.media_id = Episodes.id AND Videos.media_type = 1
                                                LEFT OUTER JOIN TvGenreLinks ON Tvs.id = TvGenreLinks.tv_id
                                                LEFT OUTER JOIN TvGenres ON TvGenreLinks.genre_id = TvGenres.id
                                                LEFT OUTER JOIN TvCasts ON Tvs.id = TvCasts.tv_id
                                                LEFT OUTER JOIN TvCrews ON Tvs.id = TvCrews.tv_id
                                                LEFT OUTER JOIN TvCollectionLinks ON Tvs.id = TvCollectionLinks.tv_id
                                                LEFT OUTER JOIN EpisodesUserWatched ON Episodes.id = EpisodesUserWatched.episode_id AND EpisodesUserWatched.user_name = ?1
                                                ", parameters, Some(user), Some("Tvs.id"), order_by, limit, offset);

        //println!("sql: {}", &sql);
        let m_conn = self.conn.lock().unwrap();
        let conn = m_conn.as_ref().unwrap();
        let mut stmt = conn.prepare(&sql)?;
    
        let rows = stmt.query_map(param.as_slice(), |row| {
            Ok(TvResult{
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

    

    pub fn get_seasons(&self, user: &String, tv_id: u64) -> Result<Vec<Season>, Error>{
        let sql = "SELECT
                            SeasonsView.season_number,
                            SeasonsView.episode_count,
                            SeasonsView.title,
                            SeasonsView.overview,
                            SeasonsView.poster_path,
                            SeasonsView.release_date,
                            SeasonsView.tv_id,
                            MIN(COALESCE(EpisodesUserWatched.watched, 0)),
                            SeasonsView.updated
                        FROM SeasonsView
                        LEFT OUTER JOIN Episodes ON SeasonsView.tv_id = Episodes.tv_id AND SeasonsView.season_number = Episodes.season_number
                        LEFT OUTER JOIN EpisodeCasts ON Episodes.id = EpisodeCasts.episode_id
                        LEFT OUTER JOIN EpisodeCrews ON Episodes.id = EpisodeCrews.episode_id
                        LEFT OUTER JOIN EpisodesUserWatched ON Episodes.id = EpisodesUserWatched.episode_id AND EpisodesUserWatched.user_name = ?1
                        WHERE SeasonsView.tv_id = ?2
                        GROUP BY SeasonsView.id";
        // println!("sql: {}", &sql);
        let m_conn = self.conn.lock().unwrap();
        let conn = m_conn.as_ref().unwrap();
        let mut stmt = conn.prepare(&sql)?;
    
        let rows = stmt.query_map(&[user, &tv_id.to_string()], |row| {
            Ok(Season{
                user: user.clone(),
                season_number: row.get(0)?,
                episode_count: row.get(1)?,
                title: row.get(2)?,
                overview: row.get(3)?,
                poster_path: row.get(4)?,
                release_date: row.get(5)?,
                tv_id: row.get(6)?,
                watched: parse_watched(row.get(7)?),
                updated: row.get(8)?,
                tv: None,
                episodes: Vec::new(),
            })
            
        })?;

        let mut result = Vec::new();
        for row in rows{
            result.push(row?);
        }
        Ok(result)
    }

    pub fn get_episodes(&self, user: &String, parameters: &HashMap<String, Option<(String, String)>>, 
                order_by: &Option<String>, limit: Option<u64>, offset: Option<u64>) -> Result<Vec<Episode>, Error>{
        let (sql, param) = generate_sql("SELECT 
                                                    Episodes.season_number,
                                                    Episodes.episode_number,
                                                    Episodes.release_date,
                                                    Episodes.title,
                                                    Episodes.overview,
                                                    Episodes.vote_average,
                                                    Episodes.vote_count,
                                                    Episodes.id,
                                                    Episodes.tv_id,
                                                    EpisodesUserWatched.watched,
                                                    Episodes.updated,
                                                    Tvs.title,
                                                    Tvs.poster_path
                                                FROM Episodes
                                                INNER JOIN Videos ON Videos.media_id = Episodes.id AND Videos.media_type = 1
                                                LEFT OUTER JOIN Tvs ON Episodes.tv_id = Tvs.id
                                                LEFT OUTER JOIN EpisodeCasts ON Episodes.id = EpisodeCasts.episode_id
                                                LEFT OUTER JOIN EpisodeCrews ON Episodes.id = EpisodeCrews.episode_id
                                                LEFT OUTER JOIN EpisodesUserWatched ON Episodes.id = EpisodesUserWatched.episode_id AND EpisodesUserWatched.user_name = ?1
                                                ", parameters, Some(user), Some("Episodes.id"), order_by, limit, offset);
        //println!("sql: {}", &sql);
        let m_conn = self.conn.lock().unwrap();
        let conn = m_conn.as_ref().unwrap();
        let mut stmt = conn.prepare(&sql)?;
    
        let rows = stmt.query_map(param.as_slice(), |row| {
            Ok(Episode{
                user: user.clone(),
                season_number: row.get(0)?,
                episode_number: row.get(1)?,
                release_date: row.get(2)?,
                title: row.get(3)?,
                overview: row.get(4)?,
                vote_average: row.get(5)?,
                vote_count: row.get(6)?,
                id: row.get(7)?,
                tv_id: row.get(8)?,
                watched: parse_watched(row.get(9)?),
                updated: row.get(10)?,
                tv_title: row.get(11)?,
                poster_path: row.get(12)?,
                tv: None,
                season: None,
                video: Vec::new(),
                cast: Vec::new(),
                crew: Vec::new(),
            })
            
        })?;

        let mut result = Vec::new();
        for row in rows{
            result.push(row?);
        }
        Ok(result)
    }

    pub fn get_season(&self, user: &String, tv_id: u64, season_number: u64) -> Result<Option<Season>, Error>{
        for season in self.get_seasons(user, tv_id)? {
            if season.season_number == season_number{
                return Ok(Some(season))
            }
        }
        Ok(None)
    }
    

    pub fn get_tv_cast(&self, user: &String, tv_id: u64) -> Result<Vec<Cast>, Error>{
        let sql = "SELECT
                            id,
                            character,
                            ord,
                            name,
                            profile_path
                        FROM TvCastsView
                        WHERE tv_id = ?1";
        //println!("sql: {}", &sql);
        let m_conn = self.conn.lock().unwrap();
        let conn = m_conn.as_ref().unwrap();
        let mut stmt = conn.prepare(&sql)?;
    
        let rows = stmt.query_map(&[&tv_id.to_string()], |row| {
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

    pub fn get_tv_crew(&self, user: &String, tv_id: u64) -> Result<Vec<Crew>, Error>{
        let sql = "SELECT
                            id,
                            job,
                            name,
                            profile_path
                        FROM TvCrewsView
                        WHERE tv_id = ?1";
        //println!("sql: {}", &sql);
        let m_conn = self.conn.lock().unwrap();
        let conn = m_conn.as_ref().unwrap();
        let mut stmt = conn.prepare(&sql)?;
    
        let rows = stmt.query_map(&[&tv_id.to_string()], |row| {
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

    pub fn get_episode_cast(&self, user: &String, episode_id: u64) -> Result<Vec<Cast>, Error>{
        let sql = "SELECT
                            id,
                            character,
                            ord,
                            name,
                            profile_path
                        FROM EpisodeCastsView
                        WHERE episode_id = ?1";
        //println!("sql: {}", &sql);
        let m_conn = self.conn.lock().unwrap();
        let conn = m_conn.as_ref().unwrap();
        let mut stmt = conn.prepare(&sql)?;
    
        let rows = stmt.query_map(&[&episode_id.to_string()], |row| {
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

    pub fn get_episode_crew(&self, user: &String, episode_id: u64) -> Result<Vec<Crew>, Error>{
        let sql = "SELECT
                            id,
                            job,
                            name,
                            profile_path
                        FROM EpisodeCrewsView
                        WHERE episode_id = ?1";
        //println!("sql: {}", &sql);
        let m_conn = self.conn.lock().unwrap();
        let conn = m_conn.as_ref().unwrap();
        let mut stmt = conn.prepare(&sql)?;
    
        let rows = stmt.query_map(&[&episode_id.to_string()], |row| {
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

    pub fn get_tv_trailer(&self, tv_id: u64) -> Result<Vec<Trailer>, Error>{
        let sql = "SELECT
                            name,
                            youtube_id
                        FROM TvTrailers
                        WHERE tv_id = ?";
        // println!("sql: {}", &sql);
        let m_conn = self.conn.lock().unwrap();
        let conn = m_conn.as_ref().unwrap();
        let mut stmt = conn.prepare(&sql)?;
    
        let rows = stmt.query_map(&[&tv_id.to_string()], |row| {
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

    pub fn get_tv_keywords(&self, tv_id: u64) -> Result<Vec<Keyword>, Error>{
        let sql = "SELECT
                            name,
                            id
                        FROM TvKeywordLinks
                        INNER JOIN Keywords ON TvKeywordLinks.keyword_id = Keywords.id
                        WHERE tv_id = ?";
        //println!("sql: {}", &sql);
        let m_conn = self.conn.lock().unwrap();
        let conn = m_conn.as_ref().unwrap();
        let mut stmt = conn.prepare(&sql)?;
    
        let rows = stmt.query_map(&[&tv_id.to_string()], |row| {
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

    pub fn genre_tv(&self) -> Result<Vec<Genre>, Error>{
        let sql = "SELECT
                            name,
                            id
                        FROM TvGenres";
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

    pub fn set_episode_watched(&self, user: String, movie_id: u64, watched: u64) -> Result< (), Error>{
        let m_conn = self.conn.lock().unwrap();
        let conn = m_conn.as_ref().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO EpisodesUserWatched (
                watched,
                user_name,
                episode_id) values (?1, ?2, ?3)",
            &[
                &watched.to_string(),
                &user,
                &movie_id.to_string()],
        )?;
        Ok(())
    }

    pub fn delete_tv(&self, tv_id: u64) -> Result<(), Error>{
        let mut m_conn = self.conn.lock().unwrap();
        let conn = m_conn.as_mut().unwrap();
        let tx = conn.transaction()?;

        tx.execute("DELETE FROM Tvs
                        WHERE id=?1", &[&tv_id.to_string()])?;
        
        tx.execute("DELETE FROM TvGenreLinks
                        WHERE tv_id=?1", &[&tv_id.to_string()])?;
        
        tx.execute("DELETE FROM TvCollectionLinks
                        WHERE tv_id=?1", &[&tv_id.to_string()])?;

        tx.execute("DELETE FROM TvKeywordLinks
                        WHERE tv_id=?1", &[&tv_id.to_string()])?;

        tx.execute("DELETE FROM TvTrailers
                        WHERE tv_id=?1", &[&tv_id.to_string()])?;

        tx.execute("DELETE FROM TvCasts
                        WHERE tv_id=?1", &[&tv_id.to_string()])?;

        tx.execute("DELETE FROM TvCrews
                        WHERE tv_id=?1", &[&tv_id.to_string()])?;
        
        tx.execute("DELETE FROM Seasons
                        WHERE tv_id=?1", &[&tv_id.to_string()])?;

        tx.commit()?;
        
        Ok(())
    }

    pub fn delete_episode(&self, episode_id: u64) -> Result<(), Error>{
        let mut m_conn = self.conn.lock().unwrap();
        let conn = m_conn.as_mut().unwrap();
        let tx = conn.transaction()?;

        tx.execute("DELETE FROM Episodes
                        WHERE id=?1", &[&episode_id.to_string()])?;
        
        tx.execute("DELETE FROM EpisodeCasts
                        WHERE episode_id=?1", &[&episode_id.to_string()])?;
        
        tx.execute("DELETE FROM EpisodeCrews
                        WHERE episode_id=?1", &[&episode_id.to_string()])?;

        tx.execute("DELETE FROM EpisodesUserWatched
                        WHERE episode_id=?1", &[&episode_id.to_string()])?;

        tx.commit()?;
        
        Ok(())
    }
}