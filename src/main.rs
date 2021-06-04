#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate diesel;
extern crate dotenv;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;

use std::vec;

use country_emoji::flag;
use diesel::prelude::*;
use rocket::response::Redirect;
use rocket_contrib::templates::Template;
use serde::{Deserialize, Serialize};

pub mod models;
pub mod schema;

use self::schema::*;

#[database("database")]
struct DbConn(rocket_contrib::databases::diesel::MysqlConnection);

#[derive(Serialize, Deserialize)]
struct TeraUser {
    user: models::User,
    posts: Vec<String>,
}

#[get("/user/<username>")]
fn get_user_info(conn: DbConn, username: String) -> Template {
    let user: Result<models::User, _> = users::table
        .filter(users::username.eq(&username))
        .get_result(&*conn);

    match user {
        Ok(user) => {
            let results: Vec<(models::User, models::Post)> = users::table
                .inner_join(posts::table.on(users::id.eq(posts::userid)))
                .filter(users::username.eq(&username))
                .get_results(&*conn)
                .unwrap();

            let mut context = TeraUser {
                user: user,
                posts: vec![],
            };
            context.user.country =
                flag(&context.user.country.to_uppercase()).unwrap_or_else(|| "🏳".to_owned());
            if results.len() > 0 {
                for (_, p) in results {
                    context
                        .posts
                        .push(format!("{} [{}]", p.body, p.postingtime));
                }
            }

            Template::render("user", context)
        }
        _ => {
            let context: std::collections::HashMap<String, String> =
                std::collections::HashMap::new();
            Template::render("user_not_found", context)
        }
    }
}

#[derive(Serialize, Deserialize)]
struct TeraUsers {
    users: Vec<models::User>,
}

#[get("/users?<country>")]
fn get_users(conn: DbConn, country: Option<String>) -> Template {
    let results: Result<Vec<models::User>, _> = match country {
        Some(cnt) => users::table
            .filter(users::country.eq(&cnt))
            .get_results(&*conn),
        None => users::table.get_results(&*conn),
    };

    let mut context = TeraUsers { users: vec![] };
    for mut u in results.unwrap() {
        u.country = flag(&u.country.to_uppercase()).unwrap_or_else(|| "🏳".to_owned());
        context.users.push(u);
    }

    Template::render("users", context)
}

#[get("/newpost?<username>&<text>")]
fn create_post(conn: DbConn, username: String, text: String) -> Redirect {
    let userid: Result<i32, _> = users::table
        .select(users::id)
        .filter(users::username.eq(&username))
        .get_result(&*conn);

    match userid {
        Ok(userid) => {
            let post = models::NewPost {
                userid,
                body: text,
                postingtime: chrono::offset::Utc::now().naive_utc(),
            };

            let _ = diesel::insert_into(posts::table)
                .values(post)
                .execute(&*conn);
        }
        _ => {}
    }

    Redirect::to("/")
}

#[get("/newuser?<username>&<country>&<age>")]
fn create_user(conn: DbConn, username: String, country: String, age: i32) -> Redirect {
    let user = models::NewUser {
        username,
        country: country_emoji::code(&country).unwrap_or_else(|| "zz").to_owned(),
        age,
    };

    let _ = diesel::insert_into(users::table)
        .values(user)
        .execute(&*conn);

    Redirect::to("/users")
}

#[get("/newuser")]
fn new_user_page() -> Template {
    let context: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    Template::render("new_user", context)
}

#[derive(Serialize, Deserialize)]
struct TeraPosts {
    posts: Vec<TeraPost>,
}

#[derive(Serialize, Deserialize)]
struct TeraPost {
    author: String,
    body: String,
    time: String,
}

#[get("/")]
fn index(conn: DbConn) -> Template {
    let posts = posts::table
        .inner_join(users::table.on(users::id.eq(posts::userid)))
        .select((users::username, posts::body, posts::postingtime))
        .order(posts::postingtime.desc())
        .get_results(&*conn)
        .unwrap()
        .into_iter()
        .map(
            |(author, body, postingtime): (String, String, chrono::NaiveDateTime)| TeraPost {
                author,
                body,
                time: format!("{}", postingtime),
            },
        )
        .collect::<Vec<TeraPost>>();

    let context = TeraPosts { posts };
    Template::render("index", context)
}

fn main() {
    rocket::ignite()
        .attach(DbConn::fairing())
        .attach(Template::fairing())
        .mount(
            "/",
            routes![
                index,
                get_user_info,
                get_users,
                create_post,
                create_user,
                new_user_page
            ],
        )
        .launch();
}
