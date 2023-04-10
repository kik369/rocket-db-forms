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
    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        if let Some(cookie) = request.cookies().get_private("user_id_in_cookie") {
            if let Ok(user_id) = cookie.value().parse::<u8>() {
                match query_user_by_id(user_id) {
                    Ok(user) => return Outcome::Success(user),
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

#[get("/")]
fn index(user: User, flash: Option<FlashMessage<'_>>) -> Flash<Redirect> {
    let msg = flash
        .map(|flash| format!("{}: {}", flash.kind(), flash.message()))
        .unwrap_or("no flash message".to_string());
    println!("flash msg: {:#?}", msg);
    println!("user: {:#?}", user);

    Flash::success(Redirect::to(uri!(profile)), "user is logged in")
}

#[get("/", rank = 2)]
fn no_auth_index(flash: Option<FlashMessage<'_>>) -> Flash<Redirect> {
    let msg = flash
        .map(|flash| format!("{}: {}", flash.kind(), flash.message()))
        .unwrap_or("no flash message".to_string());
    println!("flash msg: {:#?}", msg);
    Flash::error(
        Redirect::to(uri!(login_get_no_auth)),
        "user is not logged in",
    )
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

#[get("/login")]
fn login_get_auth(_user: Option<User>, flash: Option<FlashMessage<'_>>) -> Flash<Redirect> {
    let msg = flash
        .map(|flash| format!("{}: {}", flash.kind(), flash.message()))
        .unwrap_or("no flash message".to_string());
    println!("msg: {}", msg);

    Flash::success(Redirect::to(uri!(profile)), "user is logged in")
}
#[get("/login", rank = 2)]
fn login_get_no_auth(flash: Option<FlashMessage<'_>>) -> Template {
    let msg = flash
        .map(|flash| format!("{}: {}", flash.kind(), flash.message()))
        .unwrap_or("no flash message".to_string());
    println!("flash msg: {:#?}", msg);

    Template::render("login", context! {})
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
fn logout(cookies: &CookieJar<'_>) -> Template {
    cookies.remove_private(Cookie::named("user_id_in_cookie"));
    let warning = add_warning("user logged out via GET /logout");
    Template::render("logout", &context! {warning})
}

// retrieves Option<User>
#[get("/profile")]
fn profile(user: Option<User>, flash: Option<FlashMessage<'_>>) -> Result<Template, Template> {
    let msg = flash
        .map(|flash| format!("{}: {}", flash.kind(), flash.message()))
        .unwrap_or("no flash message".to_string());
    println!("flash msg: {:#?}", msg);
    match user {
        Some(user) => {
            let projects = query_all_projects_for_user(user.id);
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

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount(
            "/",
            routes![
                index,
                no_auth_index,
                all_users,
                all_projects,
                user_id,
                all_projects_for_user,
                add_user_get,
                add_user_post,
                add_project_get,
                add_project_post,
                login_get_auth,
                login_get_no_auth,
                login_post,
                logout,
                profile,
                project_id,
                edit_project_get,
                edit_project_post,
            ],
        )
        .attach(Template::fairing())
}
