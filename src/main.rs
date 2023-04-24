#[macro_use]
extern crate rocket;

mod db_queries;
mod passwords;

use db_queries::{
    add_project, add_user, edit_project, query_admin_by_id, query_all_projects,
    query_all_projects_for_user, query_all_users, query_project_by_id, query_user_by_email,
    query_user_by_id, Admin, User,
};
use passwords::verify_password;
use rocket::http::{Cookie, CookieJar};
use rocket::request::{self, FlashMessage, FromRequest, Outcome, Request};
use rocket::response::{Flash, Redirect};
use rocket::{
    form::{Contextual, Form},
    fs::{relative, FileServer},
    http::Status,
};
use rocket_dyn_templates::{context, Template};

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
                            return Outcome::Forward(());
                        }
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

fn get_flash_msg(flash: Option<FlashMessage<'_>>) -> String {
    flash
        .map(|flash| format!("{}: {}", flash.kind(), flash.message()))
        .unwrap_or_default()
}

#[get("/egg")]
fn egg() -> String {
    "🥚".to_string()
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
    let projects = query_all_projects_for_user(user.id);
    Template::render("profile", context! {projects, user, msg})
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
            // TODO: something with the to_string() and as_str() calls
            // TODO: also check query_user_by_email()
            let user = query_user_by_email(submission.email.to_string());
            match user {
                Ok(user) => {
                    if verify_password(submission.password, user.password.as_str()) {
                        cookies.add_private(Cookie::new("user_id_in_cookie", user.id.to_string()));
                        Template::render("success", context! { user })
                    } else {
                        Template::render("login", context! {})
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
    cookies.remove_private(Cookie::named("user_id_in_cookie"));
    Redirect::to(uri!(login_get_no_auth()))
}

#[get("/user/<id>")]
fn user_id(id: u8) -> Template {
    let serialized_data_user = query_user_by_id(id).unwrap();
    let serialized_data_project = query_all_projects_for_user(id);
    let context = context! {serialized_data_user, serialized_data_project};
    Template::render("user-id", context)
}

#[get("/all-projects-for-user/<id>")]
fn all_projects_for_user(id: u8) -> Template {
    let serialized_data = query_all_projects_for_user(id);
    Template::render("all-projects-for-user", context! {serialized_data})
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
) -> Redirect {
    match user {
        Some(user) => {
            let form_data = form.value.as_ref().unwrap();
            let id = add_project(form_data.name, user.id);
            Redirect::to(uri!(project_id(id)))
        }
        None => Redirect::to(uri!("/login")),
    }
}

#[get("/edit/project/<id>")]
fn edit_project_get(user: Option<User>, id: u8) -> Result<Redirect, Template> {
    match user {
        Some(user) => {
            let project = query_project_by_id(id).unwrap();
            let context = context! {user, project};
            Err(Template::render("project-edit", context))
        }
        None => Ok(Redirect::to(uri!("/login"))),
    }
}

#[post("/edit/project/<id>", data = "<form>")]
fn edit_project_post<'r>(
    form: Form<Contextual<'r, EditProjectForm<'r>>>,
    user: Option<User>,
    id: u8,
) -> Redirect {
    match user {
        Some(user) => {
            let form_data = form.value.as_ref().unwrap();
            let id = edit_project(id, form_data.name, form_data.end_date, user.id);
            Redirect::to(uri!(project_id(id)))
        }
        None => Redirect::to(uri!("/login")),
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
                add_user(submission.email, submission.password);
                Template::render("add-user", &form.context)
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
    let all_users = query_all_users();
    let user_count = all_users.len();
    let admin_count = all_users.iter().filter(|user| user.admin).count();
    let context = context! {all_users, user, admin, user_count, admin_count};
    Template::render("all-users", context)
}

#[get("/all-projects")]
fn all_projects(user: User, admin: Admin) -> Template {
    let all_projects = query_all_projects();
    let all_users = query_all_users();

    let no_end_date = all_projects
        .iter()
        .filter(|project| project.end_date.is_empty())
        .count();
    let project_count = all_projects.len();
    let percentage = (no_end_date as f64 / project_count as f64) * 100.0;

    let context =
        context! {all_projects, all_users, user, admin, no_end_date, project_count, percentage};
    Template::render("all-projects", context)
}

#[catch(404)]
fn not_found() -> Template {
    Template::render("catchers/404", context! {})
}

#[catch(500)]
fn server_error() -> Template {
    Template::render("catchers/500", context! {})
}

// #[launch]
#[shuttle_runtime::main]
async fn rocket() -> shuttle_rocket::ShuttleRocket {
    let rocket = rocket::build()
        .mount(
            "/",
            routes![
                egg,
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
                all_users,
                all_projects,
            ],
        )
        .mount("/", FileServer::from(relative!("static")))
        .register("/", catchers![not_found, server_error])
        .attach(Template::fairing());
    Ok(rocket.into())
}

// #[shuttle_runtime::main]
// async fn rocket() -> shuttle_rocket::ShuttleRocket {
//     let rocket = rocket::build().mount("/hello", routes![index]);

//     Ok(rocket.into())
// }
