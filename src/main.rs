#[macro_use]
extern crate rocket;

use rocket_dyn_templates::{context, Template};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Person {
    id_pers: i32,
    name: String,
    hint: String,
}

#[get("/")]
fn dbcontents() -> Template {
    let conn = Connection::open("db.sqlite").unwrap();
    let mut stmt = conn.prepare("SELECT * FROM person").unwrap();
    let person_iter = stmt
        .query_map([], |row| {
            let id_pers = row.get(0)?;
            let name = row.get::<_, String>(1)?;
            let hint = row.get::<_, String>(2)?;
            Ok(Person {
                id_pers,
                name,
                hint,
            })
        })
        .unwrap();

    let mut serialized_persons = Vec::new();

    for person_result in person_iter {
        let person = person_result.unwrap();
        println!("Found person {:?}", person);
        serialized_persons.push(person);
    }
    println!("serialized_persons: {:?}", &serialized_persons);

    let context = context! {serialized_persons};
    println!("context: {:?}", &context);
    Template::render("dbcontents", &context)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![dbcontents])
        .attach(Template::fairing())
}
