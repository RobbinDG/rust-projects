#[macro_use]
extern crate rocket;
mod cors;
mod transaction;
mod aggregates;
mod categories;

use std::env;
use crate::cors::CORS;
use crate::transaction::{get_transactions, post_transactions, post_transactions_form};
use rocket::serde::Serialize;
use rocket::tokio::io::{AsyncBufReadExt, AsyncReadExt};
use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use crate::aggregates::get_aggregates;
use crate::categories::post_category;

static MIGRATOR: Migrator = sqlx::migrate!(); // defaults to "./migrations"

#[launch]
async fn rocket() -> _ {
    dotenv::from_filename(".env").unwrap();

    let options = SqliteConnectOptions::new()
        .filename(dotenv::var("DATABASE_PATH").unwrap())
        .create_if_missing(true);

    println!("{:?}", options);

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
        .mount("/", routes![get_transactions, post_transactions, post_transactions_form, get_aggregates, post_category])
}
