use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::passwords;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: u8,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    id_proj: Option<u8>,
    name: String,
    start: String,
    end: String,
    id_pers: u8,
}

pub fn query_all_users() -> Vec<User> {
    let connection = Connection::open("db.sqlite").unwrap();
    let mut statement = connection.prepare("SELECT * FROM user").unwrap();
    let items_iter = statement
        .query_map([], |row| {
            let id: u8 = row.get(0)?;
            let email: String = row.get::<_, String>(1)?;
            let password: String = row.get::<_, String>(2)?;
            Ok(User {
                id: id,
                email: email,
                password: password,
            })
        })
        .unwrap();

    let mut serialized_data: Vec<User> = Vec::new();

    for user_result in items_iter {
        let user = user_result.unwrap();
        serialized_data.push(user);
    }
    serialized_data
}

pub fn query_user_id(id: u8) -> Vec<User> {
    let id = id.to_string();
    let connection = Connection::open("db.sqlite").unwrap();
    let mut statement = connection
        .prepare(format!("SELECT * FROM user WHERE id = {}", id).as_str())
        .unwrap();
    let items_iter = statement
        .query_map([], |row| {
            let id: u8 = row.get(0)?;
            let email: String = row.get::<_, String>(1)?;
            let password: String = row.get::<_, String>(2)?;
            Ok(User {
                id: id,
                email: email,
                password: password,
            })
        })
        .unwrap();

    let mut serialized_data: Vec<User> = Vec::new();

    for user_result in items_iter {
        let user = user_result.unwrap();
        serialized_data.push(user);
    }
    serialized_data
}

pub fn query_all_projects() -> Vec<Project> {
    let connection = Connection::open("db.sqlite").unwrap();
    let mut statement = connection.prepare("SELECT * FROM project").unwrap();
    let items_iter = statement
        .query_map([], |row| {
            let id_proj: Option<u8> = row.get(0)?;
            let name: String = row.get::<_, String>(1)?;
            let start: String = row.get::<_, String>(2)?;
            let end: String = row.get::<_, String>(3)?;
            let id_pers: u8 = row.get(4)?;
            Ok(Project {
                id_proj,
                name,
                start,
                end,
                id_pers,
            })
        })
        .unwrap();

    let mut serialized_data: Vec<Project> = Vec::new();

    for project_result in items_iter {
        let project = project_result.unwrap();
        serialized_data.push(project);
    }
    serialized_data
}

pub fn query_all_projects_for_user(id: u8) -> Vec<Project> {
    let id = id.to_string();
    let connection = Connection::open("db.sqlite").unwrap();
    let mut statement = connection
        .prepare(
            format!(
                "SELECT * FROM project
                JOIN person ON project.id_pers = person.id_pers
                WHERE person.id_pers = {}",
                id
            )
            .as_str(),
        )
        .unwrap();
    let items_iter = statement
        .query_map([], |row| {
            let id_proj: Option<u8> = row.get(0)?;
            let name: String = row.get::<_, String>(1)?;
            let start: String = row.get::<_, String>(2)?;
            let end: String = row.get::<_, String>(3)?;
            let id_pers: u8 = row.get(4)?;
            Ok(Project {
                id_proj,
                name,
                start,
                end,
                id_pers,
            })
        })
        .unwrap();

    let mut serialized_data: Vec<Project> = Vec::new();

    for project_result in items_iter {
        let project = project_result.unwrap();
        serialized_data.push(project);
    }
    serialized_data
}

pub fn add_user(email: &str, password: &str) {
    let password = passwords::hash_password(password);
    let connection = Connection::open("db.sqlite").unwrap();
    match connection.execute(
        "INSERT INTO user (email, password) VALUES (?1, ?2)",
        params![email, password],
    ) {
        Ok(updated) => println!("{} rows were updated", updated),
        Err(err) => println!("Update failed: {}", err),
    }
}

pub fn add_project(name: &str, start: &str, end: &str, id_pers: u8) {
    let connection = Connection::open("db.sqlite").unwrap();
    match connection.execute(
        "INSERT INTO project (name, start_date, end_date, id_pers) VALUES (?1, ?2, ?3, ?4)",
        params![name, start, end, id_pers],
    ) {
        Ok(updated) => println!("{} rows were updated", updated),
        Err(err) => println!("update failed: {}", err),
    }
}

pub fn query_user_email(email: String) -> Vec<User> {
    let email = email.to_string();
    let connection = Connection::open("db.sqlite").unwrap();
    let mut statement = connection
        .prepare(format!("SELECT * FROM user WHERE email = '{}'", email).as_str())
        .unwrap();
    let items_iter = statement
        .query_map([], |row| {
            let id: u8 = row.get(0)?;
            let email: String = row.get::<_, String>(1)?;
            let password: String = row.get::<_, String>(2)?;
            Ok(User {
                id: id,
                email: email,
                password: password,
            })
        })
        .unwrap();

    let mut serialized_data: Vec<User> = Vec::new();

    for user_result in items_iter {
        let user = user_result.unwrap();
        serialized_data.push(user);
    }
    println!("query_user_email serialized_data{:?}", serialized_data);
    serialized_data
}
