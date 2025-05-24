use chrono::NaiveDate;
use csv::StringRecord;
use rocket::form::Form;
use rocket::fs::TempFile;
use rocket::response::Responder;
use rocket::serde::json::Json;
use rocket::serde::Serialize;
use rocket::tokio::io::AsyncReadExt;
use rocket::{Request, Response, State};
use sqlx::{Pool, Sqlite, SqlitePool};
use std::io::{Cursor, Error};

#[derive(Serialize, Debug)]
#[serde(crate = "rocket::serde")]
struct Transaction {
    IBAN: String,
    currency: String,
    BIC: String,
    MTCN: i64,
    date: NaiveDate,
    interest_date: NaiveDate,
    value: f64,
    balance_after: f64,
    IBAN_other: Option<String>,
    name_other: String,
    BIC_other: Option<String>,
    code: Option<String>,
    reference: Option<String>,
    description: Option<String>,
    value_orig: Option<f64>,
    currency_orig: Option<String>,
    exchange_rate: Option<f64>,
}

#[derive(Serialize, Debug)]
#[serde(crate = "rocket::serde")]
struct TransactionWithCategory {
    IBAN: String,
    currency: String,
    BIC: String,
    MTCN: i64,
    date: NaiveDate,
    interest_date: NaiveDate,
    value: f64,
    balance_after: f64,
    IBAN_other: Option<String>,
    name_other: String,
    BIC_other: Option<String>,
    code: Option<String>,
    reference: Option<String>,
    description: Option<String>,
    value_orig: Option<f64>,
    currency_orig: Option<String>,
    exchange_rate: Option<f64>,
    category: Option<String>,
}

#[get("/transactions")]
pub async fn get_transactions(
    pool: &State<SqlitePool>,
) -> Result<Json<Vec<TransactionWithCategory>>, String> {
    let transactions = match sqlx::query_as!(
        TransactionWithCategory,
        "\
        SELECT t.*, pc.category \
        FROM transactions t \
        LEFT JOIN party_categories pc \
        ON LOWER(t.name_other) = LOWER(pc.party_name)"
    )
    .fetch_all(&**pool)
    .await
    {
        Ok(transactions) => transactions,
        Err(_) => return Err(String::from("Failed to get transactions")),
    };
    Ok(Json(transactions))
}

#[get("/transactions/<year_month>")]
pub async fn get_transactions_month(
    year_month: &str,
    pool: &State<SqlitePool>,
) -> Result<Json<Vec<TransactionWithCategory>>, String> {
    let transactions = match sqlx::query_as!(
        TransactionWithCategory,
        "\
        SELECT t.*, pc.category \
        FROM transactions t \
        LEFT JOIN party_categories pc \
        ON LOWER(t.name_other) = LOWER(pc.party_name)\
        WHERE strftime('%Y %m', date) = ?",
        year_month,
    )
    .fetch_all(&**pool)
    .await
    {
        Ok(transactions) => transactions,
        Err(_) => return Err(String::from("Failed to get transactions")),
    };
    Ok(Json(transactions))
}

#[derive(FromForm)]
struct TransactionsUploadForm<'r> {
    filename: TempFile<'r>,
}

#[post("/transactions", rank = 1, data = "<form>")]
pub async fn post_transactions_form(
    mut form: Form<TransactionsUploadForm<'_>>,
    pool: &State<Pool<Sqlite>>,
) -> Result<String, UploadError> {
    process_uploaded_tsv(&mut form.filename, pool).await
}

#[post("/transactions", rank = 2, format = "text/csv", data = "<file>")]
pub async fn post_transactions(
    mut file: TempFile<'_>,
    pool: &State<Pool<Sqlite>>,
) -> Result<String, UploadError> {
    process_uploaded_tsv(&mut file, pool).await
}

#[derive(Responder)]
pub enum UploadError {
    #[response(status = 500)]
    FileOpenFailed(&'static str),
    #[response(status = 500)]
    FileReadFailed(&'static str),
    #[response(status = 400)]
    CSVParseFailed(&'static str),
}

async fn process_uploaded_tsv(
    file: &mut TempFile<'_>,
    pool: &State<Pool<Sqlite>>,
) -> Result<String, UploadError> {
    let mut contents = String::new();
    let mut open_file = match file.open().await {
        Err(_) => {
            return Err(UploadError::FileOpenFailed("Failed to open file"));
        }
        Ok(f) => f,
    };

    if let Err(e) = open_file.read_to_string(&mut contents).await {
        return Err(UploadError::FileReadFailed("Failed to read file"));
    }

    let mut rdr = csv::Reader::from_reader(Cursor::new(contents));

    let mut success_cnt = 0;
    let mut failed_cnt = 0;

    for row in rdr.records() {
        match row {
            Ok(row) => {
                match insert_transaction(pool, row).await {
                    Ok(_) => success_cnt += 1,
                    Err(_) => failed_cnt += 1,
                };
            }
            Err(e) => {
                return Err(UploadError::CSVParseFailed("Failed to parse CSV content"));
            }
        }
    }

    Ok(format!(
        "Successfully inserted {}/{} (failed: {}) transactions",
        success_cnt,
        success_cnt + failed_cnt,
        failed_cnt
    ))
}

async fn insert_transaction(
    pool: &State<Pool<Sqlite>>,
    row: StringRecord,
) -> Result<(), sqlx::Error> {
    let r0 = row.get(0);
    let r1 = row.get(1);
    let r2 = row.get(2);
    let r3 = row.get(3);
    let r4 = row.get(4);
    let r5 = row.get(5);
    let r6 = row
        .get(6)
        .and_then(|s| s.replace('.', "").replace(',', ".").parse::<f64>().ok());
    let r7 = row.get(7).and_then(|s| {
        s.replace('.', "")
            .replace(',', ".")
            .replace('+', "")
            .parse::<f64>()
            .ok()
    });
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
    ).execute(&**pool).await?;
    Ok(())
}
