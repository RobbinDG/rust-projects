use rocket::serde::json::Json;
use rocket::serde::Serialize;
use rocket::State;
use sqlx::SqlitePool;

#[derive(Serialize, Debug)]
#[serde(crate = "rocket::serde")]
struct Account {
    iban: String,
    holder: String,
    name: String,
}

#[get("/accounts")]
pub async fn get_accounts(pool: &State<SqlitePool>) -> Result<Json<Vec<Account>>, String> {
    match sqlx::query_as!(
        Account,
        "SELECT * FROM accounts"
    ).fetch_all(&**pool).await {
        Ok(acs) => Ok(Json(acs)),
        Err(e) => Err(e.to_string()),
    }
}
