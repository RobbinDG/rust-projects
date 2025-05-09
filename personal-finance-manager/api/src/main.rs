#[macro_use]
extern crate rocket;

use rocket::fs::TempFile;
use rocket::tokio::io::{AsyncBufReadExt, AsyncReadExt};
use rocket::State;
use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Pool, Sqlite, SqlitePool};
use std::io::Cursor;

static MIGRATOR: Migrator = sqlx::migrate!(); // defaults to "./migrations"

#[get("/hello/<name>/<age>")]
fn hello(name: &str, age: u8) -> String {
    format!("Hello, {} year old named {}!", age, name)
}

#[get("/transactions")]
async fn get_transactions(pool: &State<SqlitePool>) {}

#[post("/transactions", format = "text/csv", data = "<file>")]
async fn post_transactions(mut file: TempFile<'_>, pool: &State<Pool<Sqlite>>) -> std::io::Result<String> {
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

    for row in rdr.records() {
        match row {
            Ok(row) => {
                let r0 = row.get(0);
                let r1 = row.get(1);
                let r2 = row.get(2);
                let r3 = row.get(3);
                let r4 = row.get(4);
                let r5 = row.get(5);
                let r6 = row.get(6);
                let r7 = row.get(7);
                let r8 = row.get(8);
                let r9 = row.get(9);
                let r12 = row.get(12);
                let r13 = row.get(13);
                let r15 = row.get(15);
                let r19 = row.get(19);
                let r23 = row.get(23);
                let r24 = row.get(24);
                let r25 = row.get(25);
                sqlx::query!(
                    "INSERT OR IGNORE INTO transactions VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    r0,
                    r1,
                    r2,
                    r3,
                    r4,
                    r5,
                    r6,
                    r7,
                    r8,
                    r9,
                    r12,
                    r13,
                    r15,
                    r19,
                    r23,
                    r24,
                    r25,
                ).execute(&**pool).await.unwrap();
            }
            Err(e) => {
                return Ok(format!("CSV parse error: {}", e));
            }
        }
    }
    Ok(if let Some(result) = rdr.records().next() {
        match result {
            Ok(record) => format!("First line: {:?}", record),
            Err(e) => format!("CSV parse error: {}", e),
        }
    } else {
        "Empty CSV file".to_string()
    })
}

#[post("/parties")]
async fn parties(name: &str, category: &str, pool: &State<SqlitePool>) -> std::io::Result<()> {
    sqlx::query!("INSERT INTO parties VALUES (?, ?)", name, category).execute(&**pool).await.unwrap();
    Ok(())
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
        .mount("/", routes![hello, post_transactions])
}
