#[macro_use]
extern crate rocket;

mod db_queries;
mod passwords;

use db_queries::{
    add_project, add_user, edit_project, query_all_projects, query_all_projects_for_user,
    query_all_users, query_project_by_id, query_user_by_email, query_user_by_id, User,
};
use rocket::http::{Cookie, CookieJar};
use rocket::request::{self, FlashMessage, FromRequest, Outcome, Request};
use rocket::response::{Flash, Redirect};
use rocket::{
    form::{Contextual, Form},
    http::Status,
};
use rocket_dyn_templates::{context, Template};
use std::collections::HashMap;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = std::convert::Infallible;
    async fn from_request(request: &'r Request<'_>) -> request::Outcome<User, Self::Error> {
        if let Some(cookie) = request.cookies().get_private("user_id_in_cookie") {
            if let Ok(user_id) = cookie.value().parse::<u8>() {
                match query_user_by_id(user_id) {
                    Ok(user) => {
                        return Outcome::Success(user);
                    }
                    Err(_) => return Outcome::Forward(()),
                }
            }
        }
        Outcome::Forward(())
    }
}

#[derive(FromForm, Debug)]
struct UserRegistrationForm<'v> {
    email: &'v str,
    password: &'v str,
    password1: &'v str,
}

#[derive(FromForm, Debug)]
struct AddProjectForm<'v> {
    name: &'v str,
}

#[derive(FromForm, Debug)]
struct EditProjectForm<'v> {
    name: &'v str,
    end_date: &'v str,
}

#[derive(FromForm, Debug)]
struct LoginForm<'v> {
    email: &'v str,
    password: &'v str,
}

// add warning text to template. this will be removed
fn add_warning(warning_input: &str) -> HashMap<&'static str, &str> {
    let mut warning = HashMap::new();
    warning.insert("warning", warning_input);
    warning
}

fn get_flash_msg(flash: Option<FlashMessage<'_>>) -> String {
    flash
        .map(|flash| format!("{}: {}", flash.kind(), flash.message()))
        .unwrap_or_default()
}

#[get("/")]
fn index(user: User) -> Template {
    Template::render("home", context! {user})
}

#[get("/", rank = 2)]
fn index_no_auth() -> Template {
    Template::render("home", context! {})
}

#[get("/profile")]
fn profile(user: User, flash: Option<FlashMessage<'_>>) -> Result<Template, Flash<Redirect>> {
    let msg = get_flash_msg(flash);
    let projects = query_all_projects_for_user(user.id);
    let context = context! {projects, user, msg};
    Ok(Template::render("profile", &context))
}

#[get("/profile", rank = 2)]
fn profile_no_auth() -> Flash<Redirect> {
    Flash::success(
        Redirect::to(uri!(login_get_no_auth())),
        "user from /profile not logged in; redirecting to login",
    )
}

#[get("/login")]
fn login_get(_user: User) -> Flash<Redirect> {
    Flash::success(
        Redirect::to(uri!(profile())),
        "user from /login already logged in; redirecting to /profile",
    )
}

#[get("/login", rank = 2)]
fn login_get_no_auth(flash: Option<FlashMessage<'_>>) -> Template {
    let msg = get_flash_msg(flash);
    Template::render("login", context! {msg})
}

#[post("/login", data = "<form>")]
fn login_post<'r>(
    cookies: &CookieJar<'_>,
    form: Form<Contextual<'r, LoginForm<'r>>>,
) -> (Status, Template) {
    let template = match form.value {
        // form contains data
        Some(ref submission) => {
            let user = query_user_by_email(submission.email.to_string());
            match user {
                Ok(user) => {
                    if passwords::verify_password(submission.password, user.password.as_str()) {
                        // password is correct
                        cookies.add_private(Cookie::new("user_id_in_cookie", user.id.to_string()));
                        let warning =
                            add_warning("password is correct, user logged in, user_id_in_cookie");
                        let context = context! { user, warning };
                        Template::render("success", &context)
                    } else {
                        // password is not correct
                        let warning = add_warning("user found ind db; password is not correct");
                        let context = context! {warning};
                        Template::render("login", &context)
                    }
                }
                Err(e) => {
                    // form contains data; user does not exist
                    let formatted_warning = format!("user not found in db; error: {}", e);
                    let warning = add_warning(formatted_warning.as_str());
                    let context = context! {warning};
                    Template::render("login", &context)
                }
            } // end of match user
        }
        // form does not contain data
        None => Template::render("login", &form.context),
    };
    (form.context.status(), template)
}

#[get("/logout")]
fn logout(cookies: &CookieJar<'_>) -> Flash<Redirect> {
    cookies.remove_private(Cookie::named("user_id_in_cookie"));
    Flash::success(Redirect::to(uri!(login_get_no_auth())), "user logged out")
}

// TODO all-users only be visible to admins
#[get("/all-users")]
fn all_users() -> Template {
    let all_users = query_all_users();
    let warning = add_warning("this should only be visible to admins");
    let context = context! {all_users, warning};
    Template::render("all-users", &context)
}

// TODO all-projects only be visible to admins
#[get("/all-projects")]
fn all_projects() -> Template {
    let all_projects = query_all_projects();
    let warning = add_warning("this should only be visible to admins");
    let context = context! {all_projects, warning};
    Template::render("all-projects", &context)
}

#[get("/user/<id>")]
fn user_id(id: u8) -> Template {
    let serialized_data_user = query_user_by_id(id).unwrap();
    let serialized_data_project = query_all_projects_for_user(id);
    let context = context! {serialized_data_user, serialized_data_project};
    Template::render("user-id", &context)
}

#[get("/all-projects-for-user/<id>")]
fn all_projects_for_user(id: u8) -> Template {
    let serialized_data = query_all_projects_for_user(id);
    Template::render("all-projects-for-user", context! {serialized_data})
}

#[get("/project/<id>")]
fn project_id(id: u8, user: Option<User>) -> Result<Template, Template> {
    match user {
        // user is logged in
        Some(user) => match query_project_by_id(id) {
            Ok(project) => {
                if user.id == project.user_id {
                    let warning = add_warning("user.id == project.user_id; display project page");
                    let context = context! {user, project, warning};
                    Ok(Template::render("project-id", &context))
                } else {
                    let warning = add_warning("user.id != project.user_id; this is not your project or project does not exist");
                    let context = context! {user, warning};
                    Err(Template::render("profile", &context))
                }
            }
            Err(_) => Err(Template::render("profile", &context! {user})),
        },
        // user is not logged in
        None => Ok(Template::render("login", context! {})),
    }
}

#[get("/add-project")]
fn add_project_get(user: Option<User>) -> Result<Redirect, Template> {
    match user {
        Some(user) => {
            let warning = add_warning("you are logged in; you can add a project");
            let context = context! {user, warning};
            Err(Template::render("add-project", &context))
        }
        None => Ok(Redirect::to(uri!("/login"))),
    }
}

#[post("/add-project", data = "<form>")]
fn add_project_post<'r>(
    form: Form<Contextual<'r, AddProjectForm<'r>>>,
    user: Option<User>,
) -> Result<Template, Redirect> {
    match user {
        Some(user) => {
            let form_data = form.value.as_ref().unwrap();
            add_project(form_data.name, user.id);

            let formatted_warning = format!(
                "you are logged in; project '<b>{}</b>' added",
                form_data.name
            );
            let warning = add_warning(formatted_warning.as_str());
            let context = context! {user, warning};
            Ok(Template::render("add-project", &context))
        }
        None => Err(Redirect::to(uri!("/login"))),
    }
}

#[get("/edit/project/<id>")]
fn edit_project_get(user: Option<User>, id: u8) -> Result<Redirect, Template> {
    match user {
        Some(user) => {
            let project = query_project_by_id(id).unwrap();
            let warning = add_warning("you are logged in; you can EDIT this project");
            let context = context! {user, project, warning};
            Err(Template::render("project-edit", &context))
        }
        None => Ok(Redirect::to(uri!("/login"))),
    }
}

#[post("/edit/project/<id>", data = "<form>")]
fn edit_project_post<'r>(
    form: Form<Contextual<'r, EditProjectForm<'r>>>,
    user: Option<User>,
    id: u8,
) -> Result<Redirect, Template> {
    match user {
        Some(user) => {
            let form_data = form.value.as_ref().unwrap();
            edit_project(id, form_data.name, form_data.end_date, user.id);
            let warning = add_warning("project updated");
            let context = context! {user, warning};
            Err(Template::render("project-updated", &context))
        }
        None => Ok(Redirect::to(uri!("/login"))),
    }
}

#[get("/add-user")]
fn add_user_get(user: Option<User>) -> Result<Redirect, Template> {
    match user {
        Some(_user) => Ok(Redirect::to(uri!("/profile"))),
        None => {
            let warning = add_warning("not logged in; register a new user");
            let context = &context! {warning};
            Err(Template::render("add-user", &context))
        }
    }
}

#[post("/add-user", data = "<form>")]
fn add_user_post<'r>(form: Form<Contextual<'r, UserRegistrationForm<'r>>>) -> (Status, Template) {
    let template = match form.value {
        Some(ref submission) => {
            if submission.password == submission.password1 {
                add_user(submission.email, submission.password);
                Template::render("add-user", &form.context)
            } else {
                let warning = add_warning("passwords do not match");
                let context = context! {warning};
                Template::render("add-user", &context)
            }
        }
        None => Template::render("add-user", &form.context),
    };

    (form.context.status(), template)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount(
            "/",
            routes![
                index,
                index_no_auth,
                profile,
                profile_no_auth,
                login_get,
                login_get_no_auth,
                login_post,
                all_users,
                all_projects,
                user_id,
                all_projects_for_user,
                add_user_get,
                add_user_post,
                add_project_get,
                add_project_post,
                logout,
                project_id,
                edit_project_get,
                edit_project_post,
            ],
        )
        .attach(Template::fairing())
}
