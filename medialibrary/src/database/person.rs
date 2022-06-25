use std::collections::HashMap;

use crate::library::cast::Person;
use crate::library::cast::PersonResult;
use crate::rustmdb;
use super::Error;
use super::SqlLibrary;
use super::generate_sql;


impl SqlLibrary{

    pub fn create_person(&self, person: &rustmdb::model::Person) -> Result<(Vec<u64>, Vec<String>), Error>{
        let mut m_conn = self.conn.lock().unwrap();
        let conn = m_conn.as_mut().unwrap();
        let tx = conn.transaction()?;

        let person_ids = Vec::new();
        let mut rsc_path = Vec::new();

        tx.execute(
            "INSERT OR REPLACE INTO Persons (
                id,
                birthday,
                known_for_department,
                deathday,
                name,
                gender,
                biography,
                popularity,
                place_of_birth,
                profile_path) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",

            &[
            &person.id.to_string(),
            &person.birthday.as_ref().unwrap_or(&"".to_string()),
            &person.known_for_department.as_ref().unwrap_or(&"".to_string()),
            &person.deathday.as_ref().unwrap_or(&"".to_string()),
            &person.name,
            &person.gender.to_string(),
            &person.biography,
            &person.popularity.to_string(),
            &person.place_of_birth.as_ref().unwrap_or(&"".to_string()),
            &person.profile_path.as_ref().unwrap_or(&"".to_string())],
        )?;

        if let Some(profile_path) = &person.profile_path{
            rsc_path.push(profile_path.clone())
        }

        tx.commit()?;

        Ok((person_ids, rsc_path))
    }

    pub fn get_person(&self, user: &String, person_id: u64) -> Result<Option<Person>, Error>{
        let sql = "SELECT 
                            id,
                            birthday,
                            known_for_department,
                            deathday,
                            name,
                            gender,
                            biography,
                            popularity,
                            place_of_birth,
                            profile_path 
                        FROM Persons
                        WHERE Persons.id = ?1
                        GROUP BY Persons.id ";

        //println!("sql: {}", &sql);
        let m_conn = self.conn.lock().unwrap();
        let conn = m_conn.as_ref().unwrap();
        let mut stmt = conn.prepare(&sql)?;
    
        let rows = stmt.query_map([&person_id.to_string()], |row| {
            Ok(Person{
                user: user.clone(),
                id: row.get(0)?,
                birthday: row.get(1)?,
                known_for_department: row.get(2)?,
                deathday: row.get(3)?,
                name: row.get(4)?,
                gender: row.get(5)?,
                biography: row.get(6)?,
                popularity:row.get(7)?,
                place_of_birth: row.get(8)?,
                profile_path: row.get(9)?,
                cast_movie: Vec::new(),
                crew_movie: Vec::new(),
                cast_tv:  Vec::new(),
                crew_tv:  Vec::new(),
            })
            
        })?;

        for person in rows{
            return Ok(Some(person?))
        }

        Ok(None)
    }

    pub fn get_persons(&self, user: &String, parameters: &HashMap<String, Option<(String, String)>>,
                    order_by: &Option<String>, limit: Option<u64>, offset: Option<u64>) -> Result<Vec<PersonResult>, Error>{
        let (sql, param) = generate_sql("SELECT 
                                                    id,
                                                    name,
                                                    birthday,
                                                    profile_path 
                                                FROM Persons
                                                ", parameters, None, Some("Persons.id"), order_by, limit, offset);

        //println!("sql: {}", &sql);
        let m_conn = self.conn.lock().unwrap();
        let conn = m_conn.as_ref().unwrap();
        let mut stmt = conn.prepare(&sql)?;
    
        let rows = stmt.query_map(param.as_slice(), |row| {
            Ok(PersonResult{
                user: user.clone(),
                id: row.get(0)?,
                name: row.get(1)?,
                birthday: row.get(2)?,
                profile_path: row.get(3)?,
            })
            
        })?;

        let mut result = Vec::new();
        for row in rows{
            result.push(row?);
        }
        Ok(result)
    }

    pub fn delete_person(&self, person_id: u64) -> Result<(), Error>{
        let mut m_conn = self.conn.lock().unwrap();
        let conn = m_conn.as_mut().unwrap();
        let tx = conn.transaction()?;

        tx.execute("DELETE FROM Persons
                        WHERE id=?1", &[&person_id.to_string()])?;
        
        tx.commit()?;
        
        Ok(())
    }
}