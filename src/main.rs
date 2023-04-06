#[macro_use]
extern crate rocket;

mod db_queries;

use rocket_dyn_templates::{context, Template};

#[get("/all-users")]
fn all_users() -> Template {
    let serialized_data = db_queries::query_all_users();
    let context = context! {serialized_data};
    Template::render("all-users", &context)
}

#[get("/all-projects")]
fn all_projects() -> Template {
    let serialized_data = db_queries::query_all_projects();
    let context = context! {serialized_data};
    Template::render("all-projects", &context)
}

#[get("/user/<id>")]
fn user_id(id: u8) -> Template {
    let serialized_data = db_queries::query_user_id(id);
    let serialized_data2 = db_queries::query_all_projects_for_user(id);
    let context = context! {serialized_data, serialized_data2};
    Template::render("user-id", &context)
}

#[get("/all-projects-for-user/<id>")]
fn all_projects_for_user(id: u8) -> Template {
    let serialized_data = db_queries::query_all_projects_for_user(id);
    let context = context! {serialized_data};
    Template::render("all-projects-for-user", &context)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount(
            "/",
            routes![all_users, all_projects, user_id, all_projects_for_user],
        )
        .attach(Template::fairing())
}
