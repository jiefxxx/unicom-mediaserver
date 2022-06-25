use std::collections::HashMap;

use crate::library::collection::{Collection, CollectionResult};

use super::SqlLibrary;
use super::{Error, generate_sql};

impl SqlLibrary{
    pub fn create_collection(&self, user: &String, name: String)  -> Result<Collection, Error>{
        let m_conn = self.conn.lock().unwrap();
        m_conn.as_ref().unwrap().execute(
            "INSERT INTO Collections (
                name,
                description,
                creator,
                creation_date,
                poster_path) values (?1, '', ?2, datetime('now'), '')",
            &[&name, user] 
        )?;

        let id = m_conn.as_ref().unwrap().last_insert_rowid() as u64;

        drop(m_conn);

        Ok(self.get_collection(user, id)?.unwrap())
    }

    pub fn update_collection(&self, user: &String, collection: &Collection)  -> Result<Collection, Error>{
        let m_conn = self.conn.lock().unwrap();
        m_conn.as_ref().unwrap().execute(
            "INSERT OR REPLACE INTO Collections (
                id,
                name,
                description,
                creator,
                creation_date,
                poster_path) values (?1, ?2, ?3, ?4, ?5, ?6)",
            &[&collection.id.to_string(), 
                    &collection.name,
                    &collection.description,
                    &collection.creator,
                    &collection.creation_date,
                    &collection.poster_path,] 
        )?;
        let id = m_conn.as_ref().unwrap().last_insert_rowid() as u64;

        drop(m_conn);

        Ok(self.get_collection(user, id)?.unwrap())
    }

    pub fn add_movie_collection(&self, collection_id: u64, movie_id: u64)  -> Result<(), Error>{
        let m_conn = self.conn.lock().unwrap();
        m_conn.as_ref().unwrap().execute(
            "INSERT INTO MovieCollectionLinks (
                movie_id,
                collection_id) values (?1, ?2)",
            &[&movie_id.to_string(), &collection_id.to_string()] 
        )?;
        Ok(())
    }

    pub fn add_tv_collection(&self, collection_id: u64, tv_id: u64)  -> Result<(), Error>{
        let m_conn = self.conn.lock().unwrap();
        m_conn.as_ref().unwrap().execute(
            "INSERT INTO TvCollectionLinks (
                tv_id,
                collection_id) values (?1, ?2)",
            &[&tv_id.to_string(), &collection_id.to_string()] 
        )?;
        Ok(())
    }

    pub fn get_collection(&self, user: &String, collection_id: u64)-> Result<Option<Collection>, Error>{
        let sql = "SELECT 
                            id,
                            name,
                            description,
                            creator,
                            creation_date,
                            poster_path
                        FROM Collections
                        WHERE id= ?1";
        
        let m_conn = self.conn.lock().unwrap();
        let conn = m_conn.as_ref().unwrap();
        let mut stmt = conn.prepare(&sql)?;
        
        let rows = stmt.query_map(&[&collection_id.to_string()], |row| {

            Ok(Collection{
                user: user.clone(),
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                creator: row.get(3)?,
                creation_date: row.get(4)?,
                poster_path: row.get(5)?,
                movie: Vec::new(),
                tv: Vec::new(),
            })
        })?;

        for row in rows{
            return Ok(Some(row?));
        }

        Ok(None)
    }

    pub fn get_collections(&self, user: &String, parameters: &HashMap<String, 
                        Option<(String, String)>>, order_by: &Option<String>, limit: Option<u64>, offset: Option<u64>) -> Result<Vec<CollectionResult>, Error>{
        let (sql, param) = generate_sql("SELECT 
                                                    Collections.id,
                                                    Collections.name,
                                                    Collections.creator,
                                                    Collections.creation_date,
                                                    Collections.poster_path
                                                FROM Collections
                                                LEFT OUTER JOIN MovieCollectionLinks ON Collections.id = MovieCollectionLinks.collection_id
                                                LEFT OUTER JOIN TvCollectionLinks ON Collections.id = TvCollectionLinks.collection_id
                                                ", &parameters, None, Some("Collections.id"), order_by, limit, offset);
        // println!("sql: {}", &sql);
        let m_conn = self.conn.lock().unwrap();
        let conn = m_conn.as_ref().unwrap();
        let mut stmt = conn.prepare(&sql)?;
    
        let rows = stmt.query_map(param.as_slice(), |row| {
            Ok(CollectionResult{ 
                user: user.clone(),
                id: row.get(0)?,
                name: row.get(1)?,
                creator: row.get(2)?,
                creation_date: row.get(3)?,
                poster_path: row.get(4)?, 
                })
            
        })?;

        let mut result = Vec::new();
        for row in rows{
            result.push(row?);
        }
        Ok(result)
    }

    pub fn delete_collection(&self, collection_id: u64) -> Result<(), Error>{
        let mut m_conn = self.conn.lock().unwrap();
        let conn = m_conn.as_mut().unwrap();
        let tx = conn.transaction()?;

        tx.execute("DELETE FROM Collections
                        WHERE id=?1", &[&collection_id.to_string()])?;
        
        tx.commit()?;
        
        Ok(())
    }
}