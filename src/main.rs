#[macro_use]
extern crate rocket;

mod bg_func;
mod db_queries;
mod passwords;
mod serialise;

use bg_func::print_hello;
use db_queries::{
    add_project, add_user, delete_project_by_id, edit_project, query_admin_by_id,
    query_all_projects, query_all_projects_for_user, query_all_users, query_project_by_id,
    query_user_by_email, query_user_by_id, Admin, User,
};
use passwords::verify_password;
use rocket::fairing::AdHoc;
use rocket::form::{Contextual, Form};
use rocket::fs::{relative, FileServer};
use rocket::http::{Cookie, CookieJar, Status};
use rocket::request::{self, FlashMessage, FromRequest, Outcome, Request};
use rocket::response::{Flash, Redirect};
use rocket::serde::json::Json;
use rocket_dyn_templates::{context, Template};
use serialise::get_flash_msg;

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
                    Err(_) => return Outcome::Forward(Status::Unauthorized),
                }
            }
        }
        Outcome::Forward(Status::Unauthorized)
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Admin {
    type Error = std::convert::Infallible;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Admin, Self::Error> {
        if let Some(cookie) = request.cookies().get_private("user_id_in_cookie") {
            if let Ok(user_id) = cookie.value().parse::<u8>() {
                match query_admin_by_id(user_id) {
                    Ok(admin) => {
                        if admin.user.admin {
                            return Outcome::Success(admin);
                        } else {
                            return Outcome::Forward(Status::Unauthorized);
                        }
                    }
                    Err(_) => return Outcome::Forward(Status::Unauthorized),
                }
            }
        }
        Outcome::Forward(Status::Unauthorized)
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

#[get("/egg")]
fn egg() -> String {
    "🥚".to_string()
}

#[get("/json")]
fn json_route() -> Json<&'static str> {
    Json("{ \"message\": \"Hello, world!\" }")
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
fn profile(user: User, flash: Option<FlashMessage<'_>>) -> Template {
    let msg = get_flash_msg(flash);
    match query_all_projects_for_user(user.id) {
        Ok(projects) => Template::render("profile", context! {projects, user, msg}),
        Err(_) => Template::render(
            "error",
            context! {msg: "Failed to query projects or users."},
        ),
    }
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

#[derive(FromForm, Debug)]
struct LoginForm<'v> {
    email: &'v str,
    password: &'v str,
}

#[post("/login", data = "<form>")]
fn login_post<'r>(cookies: &CookieJar<'_>, form: Form<Contextual<'r, LoginForm<'r>>>) -> Template {
    match form.value {
        Some(ref submission) => {
            match query_user_by_email(submission.email.to_string()) {
                Ok(user) => {
                    match verify_password(submission.password, &user.password) {
                        Ok(valid) => {
                            if valid {
                                cookies.add_private(Cookie::new(
                                    "user_id_in_cookie",
                                    user.id.to_string(),
                                ));
                                Template::render("success", context! { user })
                            } else {
                                Template::render("login", context! {})
                            }
                        }
                        Err(_) => {
                            // Handle the password verification error
                            Template::render(
                                "error",
                                context! {msg: "Password verification failed."},
                            )
                        }
                    }
                }
                Err(_) => Template::render("login", context! {}),
            }
        }
        None => Template::render("login", context! {}),
    }
}

#[get("/logout")]
fn logout(cookies: &CookieJar<'_>) -> Redirect {
    cookies.remove_private("user_id_in_cookie");
    Redirect::to(uri!(login_get_no_auth()))
}

#[get("/user/<user_id>")]
fn user_id(user_id: u8) -> Template {
    match (
        query_user_by_id(user_id),
        query_all_projects_for_user(user_id),
    ) {
        (Ok(user), Ok(projects)) => {
            let context = context! {user, projects};
            Template::render("user-id", context)
        }
        _ => Template::render(
            "error",
            context! {msg: "Failed to query projects or users."},
        ),
    }
}

#[get("/all-projects-for-user/<id>")]
fn all_projects_for_user(id: u8) -> Template {
    match query_all_projects_for_user(id) {
        Ok(projects) => {
            let serialised_data = projects;
            Template::render("all-projects-for-user", context! {serialised_data})
        }
        Err(_) => Template::render("error", context! {msg: "Failed to query projects."}),
    }
}

#[get("/project/<id>")]
fn project_id(id: u8, user: Option<User>) -> Template {
    match user {
        // user is logged in
        Some(user) => match query_project_by_id(id) {
            // project found
            Ok(project) => {
                if user.id == project.user_id {
                    // user is the owner of the project
                    Template::render("project-id", context! {user, project})
                } else {
                    // user is not the owner of the project
                    Template::render("profile", context! {user})
                }
            }
            // project not found
            Err(_) => Template::render("profile", context! {user}),
        },
        // user is not logged in
        None => Template::render("login", context! {}),
    }
}

#[get("/add-project")]
fn add_project_get(user: Option<User>) -> Result<Redirect, Template> {
    match user {
        Some(user) => {
            let context = context! {user};
            Err(Template::render("add-project", context))
        }
        None => Ok(Redirect::to(uri!("/login"))),
    }
}

#[post("/add-project", data = "<form>")]
fn add_project_post<'r>(
    form: Form<Contextual<'r, AddProjectForm<'r>>>,
    user: Option<User>,
) -> Result<Redirect, Template> {
    match user {
        Some(user) => {
            let form_data = form.value.as_ref().unwrap();
            match add_project(form_data.name, user.id) {
                Ok(id) => Ok(Redirect::to(uri!(project_id(id)))),
                Err(_) => Err(Template::render(
                    "error",
                    context! {msg: "Failed to add project."},
                )),
            }
        }
        None => Ok(Redirect::to(uri!("/login"))),
    }
}

#[get("/edit/project/<project_id>")]
fn edit_project_get(user: Option<User>, project_id: u8) -> Result<Redirect, Template> {
    match user {
        Some(user) => {
            let project = query_project_by_id(project_id).unwrap();
            let context = context! {user, project};
            Err(Template::render("project-edit", context))
        }
        None => Ok(Redirect::to(uri!("/login"))),
    }
}

#[post("/edit/project/<project_id>", data = "<form>")]
fn edit_project_post<'r>(
    form: Form<Contextual<'r, EditProjectForm<'r>>>,
    user: Option<User>,
    project_id: u8,
) -> Result<Redirect, Template> {
    match user {
        Some(user) => {
            let form_data = form.value.as_ref().unwrap();
            match edit_project(project_id, form_data.name, form_data.end_date, user) {
                Ok(updated_project_id) => Ok(Redirect::to(uri!(project_id(updated_project_id)))),
                Err(_) => Err(Template::render(
                    "error",
                    context! {msg: "Failed to edit project."},
                )),
            }
        }
        None => Ok(Redirect::to(uri!("/login"))),
    }
}

#[get("/delete/project/<project_id>")]
fn delete_project(user: Option<User>, project_id: u8) -> Result<Flash<Redirect>, Template> {
    match user {
        // user is logged in
        Some(user) => match delete_project_by_id(project_id, &user) {
            Ok(_) => Ok(Flash::success(
                Redirect::to(uri!(profile())),
                "Project deleted",
            )),
            Err(_e) => Err(Template::render(
                "profile",
                context! { msg: format!("You can't delete this project, because you're not the owner of this project."), user },
            )),
        },
        // user is not logged in
        None => Err(Template::render(
            "login",
            context! {msg: "You need to be logged in to delete a project."},
        )),
    }
}

#[get("/add-user")]
fn add_user_get(user: Option<User>) -> Result<Redirect, Template> {
    match user {
        Some(_user) => Ok(Redirect::to(uri!("/profile"))),
        None => Err(Template::render("add-user", context! {})),
    }
}

#[post("/add-user", data = "<form>")]
fn add_user_post<'r>(form: Form<Contextual<'r, UserRegistrationForm<'r>>>) -> (Status, Template) {
    let template = match form.value {
        Some(ref submission) => {
            if submission.password == submission.password1 {
                match add_user(submission.email, submission.password) {
                    Ok(_) => Template::render("success", context! {}),
                    Err(e) => Template::render(
                        "add-user",
                        context! {msg: format!("Failed to add user. Error: {}", e)},
                    ),
                }
            } else {
                Template::render("add-user", context! {})
            }
        }
        None => Template::render("add-user", &form.context),
    };

    (form.context.status(), template)
}

#[get("/all-users")]
fn all_users(user: User, admin: Admin) -> Template {
    match query_all_users() {
        Ok(all_users) => {
            let user_count = all_users.len();
            let admin_count = all_users.iter().filter(|user| user.admin).count();
            let context = context! {all_users, user, admin, user_count, admin_count};
            Template::render("all-users", context)
        }
        Err(_) => Template::render("error", context! {msg: "Failed to query users."}),
    }
}

#[get("/all-projects")]
fn all_projects(user: User, admin: Admin) -> Template {
    match (query_all_projects(), query_all_users()) {
        (Ok(all_projects), Ok(all_users)) => {
            let no_end_date = all_projects
                .iter()
                .filter(|project| project.end_date.is_empty())
                .count();
            let project_count = all_projects.len();
            let percentage = if project_count > 0 {
                (no_end_date as f64 / project_count as f64) * 100.0
            } else {
                0.0
            };

            let context = context! {all_projects, all_users, user, admin, no_end_date, project_count, percentage};
            Template::render("all-projects", context)
        }
        _ => Template::render(
            "error",
            context! {msg: "Failed to query projects or users."},
        ),
    }
}

#[catch(404)]
fn not_found() -> Template {
    Template::render("catchers/404", context! {})
}

#[catch(500)]
fn server_error() -> Template {
    Template::render("catchers/500", context! {})
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount(
            "/",
            routes![
                egg,
                json_route,
                index,
                index_no_auth,
                profile,
                profile_no_auth,
                login_get,
                login_get_no_auth,
                login_post,
                logout,
                user_id,
                all_projects_for_user,
                add_user_get,
                add_user_post,
                add_project_get,
                add_project_post,
                project_id,
                edit_project_get,
                edit_project_post,
                delete_project,
                all_users,
                all_projects,
            ],
        )
        .register("/", catchers![not_found, server_error])
        .attach(Template::fairing())
        .mount("/", FileServer::from(relative!("static")))
        .attach(AdHoc::on_liftoff("Liftoff Message", |_| {
            Box::pin(async move {
                print_hello();
            })
        }))
}
