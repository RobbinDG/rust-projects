#![deny(clippy::unwrap_used, unused_must_use, let_underscore_drop)]
#[macro_use]
extern crate rocket;
mod aggregates;
mod categories;
mod cors;
mod month_breakdown;
mod transaction;
mod accounts;

use crate::accounts::get_accounts;
use crate::aggregates::get_aggregates;
use crate::categories::post_category;
use crate::cors::CORS;
use crate::month_breakdown::get_breakdown;
use crate::transaction::{get_transactions, get_transactions_month, post_transactions, post_transactions_form};
use rocket::serde::Serialize;
use rocket::tokio::io::{AsyncBufReadExt, AsyncReadExt};
use rocket::{Build, Rocket};
use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

static MIGRATOR: Migrator = sqlx::migrate!(); // defaults to "./migrations"

async fn rocket_builder() -> Rocket<Build> {
    dotenv::from_filename(".env").expect("Failed to load .env file");

    let options = SqliteConnectOptions::new()
        .filename(dotenv::var("DATABASE_PATH").expect("Couldn't load DATABASE_PATH"))
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

    rocket::build().manage(pool).attach(CORS).mount(
        "/",
        routes![
            get_transactions,
            get_transactions_month,
            post_transactions,
            post_transactions_form,
            get_aggregates,
            post_category,
            get_breakdown,
            get_accounts,
        ],
    )
}

#[rocket::main]
async fn main() {
    rocket_builder().await.launch().await.expect("Failure to launch rocket");
}
