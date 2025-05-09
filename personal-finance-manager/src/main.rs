#[macro_use]
extern crate rocket;

use std::io::Cursor;
use rocket::fs::TempFile;
use rocket::tokio::io::{AsyncBufReadExt, AsyncReadExt};
use rocket::State;
use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Pool, Sqlite};

static MIGRATOR: Migrator = sqlx::migrate!(); // defaults to "./migrations"

#[get("/hello/<name>/<age>")]
fn hello(name: &str, age: u8) -> String {
    format!("Hello, {} year old named {}!", age, name)
}

#[post("/upload_transactions", format = "text/csv", data = "<file>")]
async fn test(mut file: TempFile<'_>, /*, pool: &State<Pool<Sqlite>>*/) -> std::io::Result<String> {
    let mut contents = String::new();
    let mut open_file = match file.open().await {
        Err(e) => {
            return Ok(format!("Failed to open file: {}", e));
        }
        Ok(f) => f,
    };

    if let Err(e) = open_file.read_to_string(&mut contents).await {
        return Ok(format!("Failed to read file: {}", e));
    }

    let mut rdr = csv::Reader::from_reader(Cursor::new(contents));
    Ok(if let Some(result) = rdr.records().next() {
        match result {
            Ok(record) => format!("First line: {:?}", record),
            Err(e) => format!("CSV parse error: {}", e),
        }
    } else {
        "Empty CSV file".to_string()
    })
    // let mut buf = String::new();
    // let x = file.open().await?.read_line(&mut buf).await?;
    //
    // Ok(buf)
}

#[launch]
async fn rocket() -> _ {
    let options = SqliteConnectOptions::new()
        .filename("transactions.db")
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
        .mount("/", routes![hello, test])
}
