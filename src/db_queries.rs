use crate::passwords::hash_password;
use crate::serialise::{parse_date, serialise_data};
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

pub fn query_all_users() -> Result<Vec<User>, Error> {
    let conn = Connection::open("db.sqlite")?;

    let mut stmt = conn.prepare("SELECT * FROM user")?;

    let items_iter = stmt.query_map([], |row| {
        let id: u8 = row.get(0)?;
        let email: String = row.get::<_, String>(1)?;
        let password: String = row.get::<_, String>(2)?;
        let admin: bool = row.get(4)?;
        Ok(User {
            id,
            email,
            password,
            admin,
        })
    })?;

    Ok(serialise_data(items_iter))
}

pub fn query_all_projects() -> Result<Vec<Project>, Error> {
    let conn = Connection::open("db.sqlite")?;
    let mut statement = conn.prepare("SELECT * FROM project")?;
    let items_iter = statement.query_map([], |row| {
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
    })?;

    Ok(serialise_data(items_iter))
}

pub fn query_user_by_id(id: u8) -> Result<User, Error> {
    let conn = Connection::open("db.sqlite").unwrap();

    let mut stmt = conn.prepare(format!("SELECT * FROM user WHERE id = {}", id).as_str())?;

    let mut items_iter = stmt.query_map([], |row| {
        let id: u8 = row.get(0)?;
        let email: String = row.get::<_, String>(1)?;
        let password: String = row.get::<_, String>(2)?;
        let admin: bool = row.get(4)?;
        Ok(User {
            id,
            email,
            password,
            admin,
        })
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
        Ok(User {
            id,
            email,
            password,
            admin,
        })
    })?;
    if let Some(user_result) = items_iter.next() {
        user_result
    } else {
        Err(Error::QueryReturnedNoRows)
    }
}

pub fn query_all_projects_for_user(user_id: u8) -> Result<Vec<Project>, Error> {
    let user_id = user_id.to_string();
    let conn = Connection::open("db.sqlite")?;
    let mut statement = conn.prepare(
        format!(
            "SELECT * FROM project
                JOIN user ON project.user_id = user.id
                WHERE user.id = {}",
            user_id
        )
        .as_str(),
    )?;
    let items_iter = statement.query_map([], |row| {
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
    })?;

    Ok(serialise_data(items_iter))
}

pub fn query_project_by_id(id: u8) -> Result<Project, Error> {
    let id = id.to_string();
    let conn = Connection::open("db.sqlite").unwrap();
    let mut statement = conn
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

pub fn add_user(email: &str, password: &str) -> Result<(), Error> {
    match hash_password(password) {
        Ok(password) => {
            let conn = Connection::open("db.sqlite")?;
            match conn.execute(
                "INSERT INTO user (email, password) VALUES (?1, ?2)",
                params![email, password],
            ) {
                Ok(_) => Ok(()),
                Err(err) => Err(err),
            }
        }
        Err(_err) => Err(rusqlite::Error::ExecuteReturnedResults),
    }
}

pub fn add_project(name: &str, user_id: u8) -> Result<u8, Error> {
    let conn = Connection::open("db.sqlite")?;
    conn.execute(
        "INSERT INTO project (name, end_date, user_id) VALUES (?1, ?2, ?3)",
        params![name, "", user_id],
    )?;

    Ok(conn.last_insert_rowid() as u8)
}

pub fn edit_project(
    project_id: u8,
    name: &str,
    end_date: &str,
    user: User,
) -> Result<u8, rusqlite::Error> {
    let end_date = if !end_date.is_empty() {
        parse_date(end_date).map_err(|_| rusqlite::Error::ExecuteReturnedResults)?
    // Convert the error type
    } else {
        "".to_string()
    };

    let conn = Connection::open("db.sqlite")?;
    conn.execute(
        "REPLACE INTO project (id_proj, name, end_date, user_id) VALUES (?1, ?2, ?3, ?4)",
        params![project_id, name, end_date, user.id],
    )?;
    Ok(conn.last_insert_rowid() as u8)
}

pub fn delete_project_by_id(project_id: u8, user: &User) -> Result<(), Error> {
    let conn = Connection::open("db.sqlite")?;
    if user_owns_project_by_id(&conn, user.id, project_id).unwrap() {
        conn.execute(
            "DELETE FROM project WHERE id_proj = ?1",
            params![project_id],
        )?;
        Ok(())
    } else {
        Err(Error::QueryReturnedNoRows)
    }
}

fn user_owns_project_by_id(conn: &Connection, user_id: u8, project_id: u8) -> Result<bool, Error> {
    let mut statement = conn.prepare("SELECT user_id FROM project WHERE id_proj = ?1")?;
    let mut rows = statement.query(params![project_id])?;

    if let Some(row) = rows.next()? {
        let owner_id: u8 = row.get(0)?;
        if owner_id == user_id {
            return Ok(true);
        }
    }
    Ok(false)
}
