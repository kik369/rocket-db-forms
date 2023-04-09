use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use crate::passwords;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: u8,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    id_proj: Option<u8>,
    name: String,
    start_date: String,
    end_date: String,
    user_id: u8,
}

fn serialize_data<T, E, I>(items: I, mut vector: Vec<T>) -> Vec<T>
where
    I: Iterator<Item = Result<T, E>>,
    E: Debug,
{
    for item in items {
        match item {
            Ok(item) => vector.push(item),
            Err(e) => println!(
                "Encountered error while serializing database items: {:?}",
                e
            ),
        }
    }
    vector
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

    let serialized_data: Vec<User> = Vec::new();
    serialize_data(items_iter, serialized_data)
}

pub fn query_user_id(id: u8) -> Option<User> {
    let connection = Connection::open("db.sqlite").unwrap();
    let mut statement = connection
        .prepare(format!("SELECT * FROM user WHERE id = {}", id).as_str())
        .unwrap();
    let mut items_iter = statement
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

    // let serialized_data: Vec<User> = Vec::new();
    // let user = serialize_data(items_iter, serialized_data);
    // user.into_iter().next()
    items_iter.next().transpose().unwrap_or(None)
}

// pub fn query_user_pass(pass: String) -> Option<User> {
//     let connection = Connection::open("db.sqlite").unwrap();
//     let mut statement = connection
//         .prepare(format!("SELECT * FROM user WHERE password = '{}'", pass).as_str())
//         .unwrap();
//     let mut items_iter = statement
//         .query_map([], |row| {
//             let id: u8 = row.get(0)?;
//             let email: String = row.get::<_, String>(1)?;
//             let password: String = row.get::<_, String>(2)?;
//             Ok(User {
//                 id: id,
//                 email: email,
//                 password: password,
//             })
//         })
//         .unwrap();

//     // Return the first user found, if any
//     items_iter.next().transpose().unwrap_or(None)
// }

pub fn query_all_projects() -> Vec<Project> {
    let connection = Connection::open("db.sqlite").unwrap();
    let mut statement = connection.prepare("SELECT * FROM project").unwrap();
    let items_iter = statement
        .query_map([], |row| {
            let id_proj: Option<u8> = row.get(0)?;
            let name: String = row.get::<_, String>(1)?;
            let start_date: String = row.get::<_, String>(2)?;
            let end_date: String = row.get::<_, String>(3)?;
            let user_id: u8 = row.get(4)?;
            Ok(Project {
                id_proj,
                name,
                start_date,
                end_date,
                user_id,
            })
        })
        .unwrap();

    let serialized_data: Vec<Project> = Vec::new();
    serialize_data(items_iter, serialized_data)
}

pub fn query_all_projects_for_user(id: u8) -> Vec<Project> {
    let id = id.to_string();
    let connection = Connection::open("db.sqlite").unwrap();
    let mut statement = connection
        .prepare(
            format!(
                "SELECT * FROM project
                JOIN user ON project.user_id = user.id
                WHERE user.id = {}",
                id
            )
            .as_str(),
        )
        .unwrap();
    let items_iter = statement
        .query_map([], |row| {
            let id_proj: Option<u8> = row.get(0)?;
            let name: String = row.get::<_, String>(1)?;
            let start_date: String = row.get::<_, String>(2)?;
            let end_date: String = row.get::<_, String>(3)?;
            let user_id: u8 = row.get(4)?;
            Ok(Project {
                id_proj,
                name,
                start_date,
                end_date,
                user_id,
            })
        })
        .unwrap();

    let serialized_data: Vec<Project> = Vec::new();
    serialize_data(items_iter, serialized_data)
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

pub fn add_project(name: &str, end_date: &str, user_id: u8) {
    let connection = Connection::open("db.sqlite").unwrap();
    match connection.execute(
        "INSERT INTO project (name, end_date, user_id) VALUES (?1, ?2, ?3)",
        params![name, end_date, user_id],
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

    let serialized_data: Vec<User> = Vec::new();
    serialize_data(items_iter, serialized_data)
}
