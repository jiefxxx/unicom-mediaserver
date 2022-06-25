use std::io;
use std::fs::File;

use pyo3::prelude::*;
use pyo3::exceptions::PyReferenceError;

use crate::{rustmdb::{get_movie, get_person, get_tv, get_tv_episode}, database::DATABASE};

use super::{RSCPATH, movie::MovieSearch, tv::{TvSearch, EpisodeSearch}, cast::PersonSearch};


pub fn create_movie(user: &String, movie_id: u64) -> PyResult<()>{
    if MovieSearch::new(user).id(movie_id)?.exist()?{
        return Ok(())
    }
    let movie = get_movie(movie_id)?;
    let (person_ids, rsc_paths) = DATABASE.create_movie(&movie)?;
    for person_id in person_ids{
        create_person(user, person_id)?;
    }
    for rsc_path in rsc_paths{
        update_rsc(&rsc_path)?;
    }

    Ok(())
}


pub fn create_person(user: &String, person_id: u64) -> PyResult<()>{
    if PersonSearch::new(user).id(person_id)?.exist()?{
        return Ok(())
    }
    let person = get_person(person_id)?;
    let (_person_ids, rsc_paths) = DATABASE.create_person(&person)?;
    for rsc_path in rsc_paths{
        update_rsc(&rsc_path)?;
    }
    Ok(())
}

pub fn create_tv(user: &String, tv_id: u64) -> PyResult<()>{
    if TvSearch::new(user).id(tv_id)?.exist()?{
        return Ok(())
    }
    let tv = get_tv(tv_id)?;
    let (person_ids, rsc_paths) = DATABASE.create_tv(&tv)?;
    for person_id in person_ids{
        create_person(user, person_id)?;
    }
    for rsc_path in rsc_paths{
        update_rsc(&rsc_path)?;
    }
    Ok(())
}

pub fn create_episode(user: &String, tv_id: u64, season_number: u64, episode_number: u64) -> PyResult<u64>{
    if let Some(episode) = EpisodeSearch::new(user).tv(tv_id)?.season(season_number)?.episode(episode_number)?.last()?{
        return Ok(episode.id)
    }
    create_tv(user, tv_id)?;
    let episode = get_tv_episode(tv_id, season_number, episode_number)?;
    let (person_ids, rsc_paths) = DATABASE.create_episode(tv_id, &episode)?;
    for person_id in person_ids{
        create_person(user, person_id)?;
    }
    for rsc_path in rsc_paths{
        update_rsc(&rsc_path)?;
    }
    Ok(episode.id)
}

pub fn update_rsc(rsc_path: &str) -> PyResult<()>{
    if rsc_path.len() == 0{
        return Ok(())
    }

    let resp = match reqwest::blocking::get("https://image.tmdb.org/t/p/original".to_string() + rsc_path){
        Ok(resp) => resp.bytes().unwrap(),
        Err(e) => return Err(PyReferenceError::new_err(format!("reqwest error getting poster path {}", e))),
    };
    
    let mut out = File::create(RSCPATH.lock().unwrap().clone() + "/original" +rsc_path)?;

    io::copy(&mut resp.as_ref(), &mut out)?;

    Ok(())
}