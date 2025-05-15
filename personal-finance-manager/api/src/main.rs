#[macro_use]
extern crate rocket;
mod cors;
mod transaction;
mod aggregates;

use crate::cors::CORS;
use crate::transaction::{get_transactions, post_transactions, post_transactions_form};
use rocket::serde::Serialize;
use rocket::tokio::io::{AsyncBufReadExt, AsyncReadExt};
use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use crate::aggregates::get_aggregates;

static MIGRATOR: Migrator = sqlx::migrate!(); // defaults to "./migrations"

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
        .mount("/", routes![get_transactions, post_transactions, post_transactions_form, get_aggregates])
}
