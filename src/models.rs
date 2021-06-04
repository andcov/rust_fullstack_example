// use chrono::NaiveDate;
use super::schema::{posts, users};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Queryable, Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub country: String,
    pub age: i32,
}

#[derive(Queryable, Debug)]
pub struct Post {
    pub id: i32,
    pub userid: i32,
    pub body: String,
    pub postingtime: chrono::NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "posts"]
pub struct NewPost {
    pub userid: i32,
    pub body: String,
    pub postingtime: chrono::NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub username: String,
    pub country: String,
    pub age: i32,
}
