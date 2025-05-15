use rocket::serde::json::Json;
use rocket::serde::Serialize;
use rocket::State;
use sqlx::SqlitePool;

#[derive(Serialize, Debug)]
#[serde(crate = "rocket::serde")]
struct MonthAggregate {
    month_year: String,
    sum: f64,
}

#[post("/categories/<category>/<party_name>")]
pub async fn post_category(category: String, party_name: String, pool: &State<SqlitePool>) -> Result<(), String> {
    match sqlx::query!(
        "INSERT INTO party_categories VALUES (?, ?)",
        party_name, category,
    ).execute(&**pool).await {
        Ok(aggs) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}
