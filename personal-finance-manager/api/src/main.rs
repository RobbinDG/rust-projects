mod cors;
mod transaction;

#[macro_use]
extern crate rocket;

use crate::cors::CORS;
use rocket::fs::TempFile;
use rocket::serde::{json::Json, Serialize};
use rocket::tokio::io::{AsyncBufReadExt, AsyncReadExt};
use rocket::State;
use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Pool, Sqlite, SqlitePool};
use std::io::Cursor;
use std::time::SystemTime;
use chrono::NaiveDate;
use crate::transaction::{get_transactions, post_transactions};

static MIGRATOR: Migrator = sqlx::migrate!(); // defaults to "./migrations"

#[get("/hello/<name>/<age>")]
fn hello(name: &str, age: u8) -> String {
    format!("Hello, {} year old named {}!", age, name)
}

// #[post("/parties")]
// async fn parties(name: &str, category: &str, pool: &State<SqlitePool>) -> std::io::Result<()> {
//     sqlx::query!("INSERT INTO parties VALUES (?, ?)", name, category).execute(&**pool).await.unwrap();
//     Ok(())
// }

#[launch]
async fn rocket() -> _ {
    let options = SqliteConnectOptions::new()
        .filename("../transactions.db")
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .expect("Database connection failed");

    MIGRATOR
        .run(&pool)
        .await
        .expect("Failed to run database migrations");

    rocket::build()
        .manage(pool)
        .attach(CORS)
        .mount("/", routes![hello, get_transactions, post_transactions])
}
