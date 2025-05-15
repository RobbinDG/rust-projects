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

#[get("/aggregates")]
pub async fn get_aggregates(pool: &State<SqlitePool>) -> Result<Json<Vec<MonthAggregate>>, String> {
    match sqlx::query_as!(
        MonthAggregate,
        "SELECT COALESCE(strftime('%m %Y', date), '') AS month_year, coalesce(sum(value), 0.0) AS sum FROM transactions GROUP BY strftime('%m%Y', date);"
    ).fetch_all(&**pool).await {
        Ok(aggs) => Ok(Json(aggs)),
        Err(e) => Err(e.to_string()),
    }
}
