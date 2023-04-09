#[macro_use]
extern crate rocket;

mod db_queries;
mod passwords;

use rocket::http::{Cookie, CookieJar};
use rocket::request::{self, FromRequest, Request};
use rocket::response::Redirect;
use rocket::{
    form::{Contextual, Form},
    http::Status,
};
use rocket_dyn_templates::{context, Template};
use std::collections::HashMap;

#[get("/")]
fn home() -> Template {
    let warning = add_warning("not checking if user is logged in");
    Template::render("home", context! {warning})
}

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
fn login_get(user: Option<db_queries::User>) -> Result<Redirect, Template> {
    match user {
        Some(_user) => Ok(Redirect::to(uri!("/profile"))),
        None => Err(Template::render("login", context! {})),
    }
}

#[post("/login", data = "<form>")]
fn login_post<'r>(
    cookies: &CookieJar<'_>,
    form: Form<Contextual<'r, LoginForm<'r>>>,
) -> (Status, Template) {
    let template = match form.value {
        Some(ref submission) => {
            let user = db_queries::query_user_email(submission.email.to_string());
            if !user.is_empty()
                && passwords::verify_password(submission.password, &user[0].password)
            {
                // password is correct
                cookies.add_private(Cookie::new("user_id_in_cookie", user[0].id.to_string()));
                let warning = add_warning("password is correct, user logged in, user_id_in_cookie");
                let context = context! { user, warning };
                Template::render("success", &context)
            } else {
                // password is not correct or user does not exist
                let warning = add_warning("password is not correct or user does not exist");
                let context = context! {user, warning};
                Template::render("login", &context)
            }
        }
        None => Template::render("login", &form.context),
    };
    (form.context.status(), template)
}

#[get("/logout")]
fn logout(cookies: &CookieJar<'_>) -> Template {
    cookies.remove_private(Cookie::named("user_id_in_cookie"));
    let warning = add_warning("user logged out via GET /logout");
    Template::render("logout", &context! {warning})
}

// this is a from request guard for User struct to check is user is logged in
// this is an arbitrary validation policy
#[rocket::async_trait]
impl<'r> FromRequest<'r> for db_queries::User {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let cookies = request.cookies();
        match cookies.get_private("user_id_in_cookie") {
            Some(cookie) => {
                // Get the user ID from the cookie. It is stored as a string
                let id = cookie.value().parse::<u8>().unwrap();

                // Fetch the user from the database using the user ID
                let user_opt = db_queries::query_user_id(id);

                // Return the user if found, otherwise forward the request
                match user_opt {
                    // user successfully retrieved from db
                    Some(user) => request::Outcome::Success(user),
                    // user not found in db
                    None => request::Outcome::Forward(()),
                }
            }
            // no cookie found
            None => request::Outcome::Forward(()),
        }
    }
}

#[get("/profile")]
fn profile(user: Option<db_queries::User>) -> Result<Template, Template> {
    match user {
        Some(user) => {
            let projects = db_queries::query_all_projects_for_user(user.id);
            let user = db_queries::query_user_id(user.id);
            let context = context! {projects, user};
            Ok(Template::render("profile", &context))
        }
        None => {
            let warning = add_warning(
                "user_id_in_cookie - false; redirect to login; profile not visible, if not logged in",
            );
            Err(Template::render("login", &context! {warning}))
        }
    }
}

// add warning text to template
fn add_warning(warning_input: &str) -> HashMap<&'static str, &str> {
    let mut warning = HashMap::new();
    warning.insert("warning", warning_input);
    warning
}

#[get("/add-project")]
fn add_project_get() -> Template {
    Template::render("add-project", &context! {})
}

#[derive(FromForm, Debug)]
struct FormDataProject<'v> {
    name: &'v str,
    end: &'v str,
    user_id: u8,
}

#[post("/add-project", data = "<form>")]
fn add_project_post<'r>(form: Form<Contextual<'r, FormDataProject<'r>>>) -> (Status, Template) {
    let template = match form.value {
        Some(ref submission) => {
            db_queries::add_project(submission.name, submission.end, submission.user_id);
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
                home,
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
                logout,
                profile,
            ],
        )
        .attach(Template::fairing())
}
