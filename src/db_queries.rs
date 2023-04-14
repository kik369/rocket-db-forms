use crate::passwords;
use chrono::NaiveDateTime;
use rusqlite::{params, Connection, Error};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: u8,
    pub email: String,
    pub password: String,
    pub admin: bool,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Admin {
    pub user: User,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    pub id_proj: Option<u8>,
    pub name: String,
    pub start_date: String,
    pub end_date: String,
    pub user_id: u8,
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
    let conn = Connection::open("db.sqlite").unwrap();

    let mut stmt = conn.prepare("SELECT * FROM user").unwrap();

    let items_iter = stmt
        .query_map([], |row| {
            let id: u8 = row.get(0)?;
            let email: String = row.get::<_, String>(1)?;
            let password: String = row.get::<_, String>(2)?;
            let admin: bool = row.get(4)?;
            Ok(User {
                id: id,
                email: email,
                password: password,
                admin: admin,
            })
        })
        .unwrap();

    let serialized_data: Vec<User> = Vec::new();
    serialize_data(items_iter, serialized_data)
}

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

pub fn query_user_by_id(id: u8) -> Result<User, Error> {
    let conn = Connection::open("db.sqlite").unwrap();

    let mut stmt = conn.prepare(format!("SELECT * FROM user WHERE id = {}", id).as_str())?;

    let mut items_iter = stmt.query_map([], |row| {
        let id: u8 = row.get(0)?;
        let email: String = row.get::<_, String>(1)?;
        let password: String = row.get::<_, String>(2)?;
        let admin: bool = row.get(4)?;
        let user = User {
            id,
            email,
            password,
            admin,
        };
        Ok(user)
    })?;
    if let Some(user_result) = items_iter.next() {
        user_result
    } else {
        Err(Error::QueryReturnedNoRows)
    }
}

pub fn query_admin_by_id(id: u8) -> Result<Admin, Error> {
    let conn = Connection::open("db.sqlite").unwrap();

    let mut stmt = conn.prepare(format!("SELECT * FROM user WHERE id = {}", id).as_str())?;

    let mut items_iter = stmt.query_map([], |row| {
        let id: u8 = row.get(0)?;
        let email: String = row.get::<_, String>(1)?;
        let password: String = row.get::<_, String>(2)?;
        let admin: bool = row.get(4)?;
        let admin = Admin {
            user: User {
                id,
                email,
                password,
                admin,
            },
        };
        Ok(admin)
    })?;
    if let Some(admin_result) = items_iter.next() {
        admin_result
    } else {
        Err(Error::QueryReturnedNoRows)
    }
}

pub fn query_user_by_email(email: String) -> Result<User, Error> {
    let conn = Connection::open("db.sqlite").unwrap();

    let mut stmt =
        conn.prepare(format!("SELECT * FROM user WHERE email = '{}'", email).as_str())?;
    let mut items_iter = stmt.query_map([], |row| {
        let id: u8 = row.get(0)?;
        let email: String = row.get::<_, String>(1)?;
        let password: String = row.get::<_, String>(2)?;
        let admin: bool = row.get(4)?;
        let user = User {
            id,
            email,
            password,
            admin,
        };
        Ok(user)
    })?;
    if let Some(user_result) = items_iter.next() {
        user_result
    } else {
        Err(Error::QueryReturnedNoRows)
    }
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

pub fn query_project_by_id(id: u8) -> Result<Project, Error> {
    let id = id.to_string();
    let connection = Connection::open("db.sqlite").unwrap();
    let mut statement = connection
        .prepare(format!("SELECT * FROM project WHERE id_proj = {}", id).as_str())
        .unwrap();
    let mut items_iter = statement.query_map([], |row| {
        let id_proj: Option<u8> = row.get(0)?;
        let name: String = row.get::<_, String>(1)?;
        let start_date: String = row.get::<_, String>(2)?;
        let end_date: String = row.get::<_, String>(3)?;
        let user_id: u8 = row.get(4)?;
        let project = Project {
            id_proj,
            name,
            start_date,
            end_date,
            user_id,
        };
        Ok(project)
    })?;
    if let Some(project_result) = items_iter.next() {
        project_result
    } else {
        Err(Error::QueryReturnedNoRows)
    }
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

pub fn add_project(name: &str, user_id: u8) {
    let connection = Connection::open("db.sqlite").unwrap();
    match connection.execute(
        "INSERT INTO project (name, end_date, user_id) VALUES (?1, ?2, ?3)",
        params![name, "", user_id],
    ) {
        Ok(updated) => println!("{} rows were updated", updated),
        Err(err) => println!("update failed: {}", err),
    }
}

pub fn edit_project(id: u8, name: &str, end_date: &str, user_id: u8) {
    let end_date = parse_date(end_date);
    let connection = Connection::open("db.sqlite").unwrap();
    match connection.execute(
        "REPLACE INTO project (id_proj, name, end_date, user_id) VALUES (?1, ?2, ?3, ?4)",
        params![id, name, end_date, user_id],
    ) {
        Ok(updated) => println!("{} rows were updated", updated),
        Err(err) => println!("update failed: {}", err),
    }
}

// parses from "2020-01-01T00:00:00" to "2020-01-01 00:00:00"
// "2020-01-01T00:00:00" is the format that the datepicker returns
// "2020-01-01 00:00:00" is the format generated by 'DATETIME DEFAULT CURRENT_TIMESTAMP' in sqlite
fn parse_date(date: &str) -> String {
    let parsed_end_date = NaiveDateTime::parse_from_str(&date, "%Y-%m-%dT%H:%M:%S")
        .expect("Failed to parse date string");
    let formatted_date_time = parsed_end_date.format("%Y-%m-%d %H:%M:%S").to_string();
    formatted_date_time
}
