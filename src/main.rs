#[macro_use]
extern crate rocket;

mod db_queries;

use rocket_dyn_templates::{context, Template};

use rocket::{
    form::{Contextual, Form},
    http::Status,
};

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

#[get("/add-user")]
fn add_user_get() -> Template {
    Template::render("add-user", &context! {})
}

#[derive(FromForm, Debug)]
struct FormDataUser<'v> {
    name: &'v str,
    data: &'v str,
}

#[post("/add-user", data = "<form>")]
fn add_user_post<'r>(form: Form<Contextual<'r, FormDataUser<'r>>>) -> (Status, Template) {
    let template = match form.value {
        Some(ref submission) => {
            db_queries::add_user(submission.name, submission.data);
            Template::render("add-user", &form.context)
        }
        None => Template::render("add-user", &form.context),
    };

    (form.context.status(), template)
}

#[get("/add-project")]
fn add_project_get() -> Template {
    Template::render("add-project", &context! {})
}

#[derive(FromForm, Debug)]
struct FormDataProject<'v> {
    name: &'v str,
    start: &'v str,
    end: &'v str,
    id_pers: u8,
}

#[post("/add-project", data = "<form>")]
fn add_project_post<'r>(form: Form<Contextual<'r, FormDataProject<'r>>>) -> (Status, Template) {
    let template = match form.value {
        Some(ref submission) => {
            db_queries::add_project(
                submission.name,
                submission.start,
                submission.end,
                submission.id_pers,
            );
            Template::render("add-project", &form.context)
        }
        None => Template::render("add-project", &form.context),
    };

    (form.context.status(), template)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount(
            "/",
            routes![
                all_users,
                all_projects,
                user_id,
                all_projects_for_user,
                add_user_get,
                add_user_post,
                add_project_get,
                add_project_post
            ],
        )
        .attach(Template::fairing())
}
