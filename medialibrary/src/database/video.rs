use std::collections::HashMap;
use super::{Error, generate_sql};

use super::{SqlLibrary, parse_concat};

use crate::library::video::{MediaInfo, Video, VideoResult, EpisodeMinimal, MovieMinimal};

impl SqlLibrary{

    pub fn create_video(&self, video: Video) -> Result<u64, Error>{
        let m_conn = self.conn.lock().unwrap();
        let conn = m_conn.as_ref().unwrap();
        conn.execute(
            "INSERT INTO Videos (
                path,
                media_type,
                duration,
                bit_rate,
                codec,
                width,
                height,
                size,
                adding) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, datetime('now'))",
            &[&video.path, 
            &video.media_type.to_string(),
            &video.duration.to_string(),
            &video.bit_rate.to_string(),
            &video.codec.as_ref().unwrap_or(&"".to_string()),
            &video.width.to_string(),
            &video.height.to_string(),
            &video.size.to_string()],
        )?;

        let video_id = conn.last_insert_rowid() as u64;
        for language in video.subtitles{
            conn.execute(
                "INSERT OR IGNORE INTO Subtitles (
                    video_id,
                    language) values (?1, ?2)",
                &[&video_id.to_string(), &language],
            )?;
        }

        for language in video.audios{
            conn.execute(
                "INSERT OR IGNORE INTO Audios (
                    video_id,
                    language) values (?1, ?2)",
                &[&video_id.to_string(), &language],
            )?;
        }

        Ok(video_id)
    }

    pub fn get_video(&self,user: &String, video_id: u64) -> Result<Option<Video>, Error>{
        let sql = "SELECT
                            id,
                            path,
                            media_type,
                            media_id,
                            duration,
                            bit_rate,
                            codec,
                            width ,
                            height,
                            size,
                            adding,
                            subtitles,
                            audios, 
                            WatchTimes.watch_time,
                            WatchTimes.last_watch
                        FROM VideosView
                        LEFT OUTER JOIN WatchTimes ON VideosView.id = WatchTimes.video_id AND WatchTimes.user_name = ?1
                        WHERE id = ?2";
        let m_conn = self.conn.lock().unwrap();
        let conn = m_conn.as_ref().unwrap();
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(&[user, &video_id.to_string()], |row| {
            Ok(Video{
                user: user.clone(),
                id: row.get(0)?,
                path:row.get(1)?,
                media_type: row.get(2)?,
                media_id: row.get(3)?,
                bit_rate: row.get(5)?,
                duration: row.get(4)?,
                size: row.get(9)?,
                adding: row.get(10)?,
                codec: row.get(6)?,
                width: row.get(7)?,
                height: row.get(8)?,
                subtitles: parse_concat(row.get(11)?).unwrap_or_default(),
                audios: parse_concat(row.get(12)?).unwrap_or_default(),
                watch_time: row.get(13)?,
                last_watch: row.get(14)?,
            })
        })?;

        for row in rows{
            return Ok(Some(row?));
        }

        Ok(None)
    }

    pub fn get_videos(&self, user: &String, parameters: &HashMap<String, Option<(String, String)>>,
            order_by: &Option<String>, limit: Option<u64>, offset: Option<u64>) -> Result<Vec<VideoResult>, Error>{
        let (sql, param) = generate_sql("SELECT 
                                id, 
                                path, 
                                media_type, 
                                media_id, 
                                adding, 
                                m_title, 
                                t_title, 
                                release_date, 
                                episode_number, 
                                season_number, 
                                duration,
                                codec,
                                size,
                                subtitles,
                                audios, 
                                m_id, 
                                t_id,
                                WatchTimes.last_watch as last_watch
                            FROM VideosView
                            LEFT OUTER JOIN WatchTimes ON VideosView.id = WatchTimes.video_id AND WatchTimes.user_name = ?1", 
                            &parameters, Some(user), Some("VideosView.id"), order_by, limit, offset);
        //println!("sql: {}", &sql);
        let m_conn = self.conn.lock().unwrap();
        let conn = m_conn.as_ref().unwrap();
        let mut stmt = conn.prepare(&sql)?;
    
        let rows = stmt.query_map(param.as_slice(), |row| {
            let media_id: Option<u64> = row.get(3)?;
            let info = match media_id{
                None => MediaInfo::Unknown,
                Some(_media_id) => match row.get(2)?{
                    0 => MediaInfo::Movie(MovieMinimal{
                        id: row.get(15)?,
                        title: row.get(5)?,
                        release_date: row.get(7)?,
                    }),
                    1 => MediaInfo::Tv(EpisodeMinimal{
                        id: row.get(16)?,
                        title: row.get(6)?,
                        season_number: row.get(9)?,
                        episode_number: row.get(8)?,
                    }),
                    _ => MediaInfo::Unknown,
                }
            };
            Ok(VideoResult{
                user: user.clone(),
                id: row.get(0)?,
                path: row.get(1)?,
                media_type: row.get(2)?,
                adding: row.get(4)?,
                info,
                duration: row.get(10)?,
                codec: row.get(11)?,
                size: row.get(12)?,
                subtitles: parse_concat(row.get(13)?).unwrap_or_default(),
                audios: parse_concat(row.get(14)?).unwrap_or_default(),
            })
        })?;

        let mut result = Vec::new();
        for row in rows{
            result.push(row?);
        }
        Ok(result)
    }

    pub fn edit_video_media_id(&self, video_id: u64, media_id: u64) -> Result<(), Error>{
        let m_conn = self.conn.lock().unwrap();
        let conn = m_conn.as_ref().unwrap();
        conn.execute(
            "UPDATE Videos SET media_id = ?1 WHERE id = ?2",
            &[
                &media_id.to_string(),
                &video_id.to_string()],
        )?;
        Ok(())
    }

    pub fn edit_video_path(&self, video_id: u64, path: &str) -> Result<(), Error>{
        let m_conn = self.conn.lock().unwrap();
        let conn = m_conn.as_ref().unwrap();
        conn.execute(
            "UPDATE Videos SET path = ?1 WHERE id = ?2",
            &[
                path,
                &video_id.to_string()],
        )?;
        Ok(())
    }

    pub fn set_watch_time(&self, user: String, video_id: u64, time: u64) -> Result< (), Error>{
        let m_conn = self.conn.lock().unwrap();
        let conn = m_conn.as_ref().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO WatchTimes (
                user_name,
                video_id,
                watch_time,
                last_watch
            ) values (?1, ?2, ?3, datetime('now'))",
            &[
                &user,
                &video_id.to_string(),
                &time.to_string()],
        )?;
        Ok(())
    }

    pub fn delete_video(&self, video_id: u64) -> Result<(), Error>{
        let mut m_conn = self.conn.lock().unwrap();
        let conn = m_conn.as_mut().unwrap();
        let tx = conn.transaction()?;

        tx.execute("DELETE FROM Videos
                        WHERE id=?1", &[&video_id.to_string()])?;
        
        tx.execute("DELETE FROM WatchTimes
                        WHERE video_id=?1", &[&video_id.to_string()])?;
        
        tx.execute("DELETE FROM Audios
                        WHERE video_id=?1", &[&video_id.to_string()])?;

        tx.execute("DELETE FROM Subtitles
                        WHERE video_id=?1", &[&video_id.to_string()])?;

        tx.commit()?;
        
        Ok(())
    }
}