#[macro_use]
extern crate rocket;

mod db_queries;
mod passwords;

use rocket::http::{Cookie, CookieJar};
use rocket_dyn_templates::{context, Template};

use rocket::{
    form::{Contextual, Form},
    http::Status,
};

#[get("/all-users")]
fn all_users() -> Template {
    let serialized_data = db_queries::query_all_users();
    Template::render("all-users", context! {serialized_data})
}

#[get("/all-projects")]
fn all_projects() -> Template {
    let serialized_data = db_queries::query_all_projects();
    Template::render("all-projects", context! {serialized_data})
}

#[get("/user/<id>")]
fn user_id(id: u8) -> Template {
    let serialized_data_user = db_queries::query_user_id(id);
    let serialized_data_project = db_queries::query_all_projects_for_user(id);
    let context = context! {serialized_data_user, serialized_data_project};
    Template::render("user-id", &context)
}

#[get("/all-projects-for-user/<id>")]
fn all_projects_for_user(id: u8) -> Template {
    let serialized_data = db_queries::query_all_projects_for_user(id);
    Template::render("all-projects-for-user", context! {serialized_data})
}

#[get("/add-user")]
fn add_user_get() -> Template {
    Template::render("add-user", &context! {})
}

#[derive(FromForm, Debug)]
struct FormDataUser<'v> {
    email: &'v str,
    password: &'v str,
}

#[post("/add-user", data = "<form>")]
fn add_user_post<'r>(form: Form<Contextual<'r, FormDataUser<'r>>>) -> (Status, Template) {
    let template = match form.value {
        Some(ref submission) => {
            db_queries::add_user(submission.email, submission.password);
            Template::render("add-user", &form.context)
        }
        None => Template::render("add-user", &form.context),
    };

    (form.context.status(), template)
}

#[derive(FromForm, Debug)]
struct LoginForm<'v> {
    email: &'v str,
    password: &'v str,
}

#[get("/login")]
fn login_get() -> Template {
    Template::render("login", &context! {})
}

#[post("/login", data = "<form>")]
fn login_post<'r>(
    cookies: &CookieJar<'_>,
    form: Form<Contextual<'r, LoginForm<'r>>>,
) -> (Status, Template) {
    let template = match form.value {
        Some(ref submission) => {
            let user = db_queries::query_user_email(submission.email);
            println!("user = {:?}", user);
            if !user.is_empty() && passwords::verify_password(submission.password, &user) {
                println!("Password is correct");
                cookies.add_private(Cookie::new("user_logged_in", user.to_string()));
                let context = context! {user};
                Template::render("success", &context)
            } else {
                println!("Password is incorrect or user does not exist");
                let context = context! {user};
                Template::render("login", &context)
            }
            // let context = context! {user};
            // Template::render("success", &context)
        }
        None => Template::render("login", &form.context),
    };
    println!("form.context.status() = {:?}", form.context.status());
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
                add_project_post,
                login_get,
                login_post,
            ],
        )
        .attach(Template::fairing())
}
