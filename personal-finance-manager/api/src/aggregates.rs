use rocket::serde::json::Json;
use rocket::serde::Serialize;
use rocket::State;
use sqlx::SqlitePool;

#[derive(Serialize, Debug)]
#[serde(crate = "rocket::serde")]
struct MonthAggregate {
    year_month: String,
    sum: f64,
}

#[get("/aggregates")]
pub async fn get_aggregates(pool: &State<SqlitePool>) -> Result<Json<Vec<MonthAggregate>>, String> {
    match sqlx::query_as!(
        MonthAggregate,
        "SELECT COALESCE(strftime('%Y %m', date), '') AS year_month, COALESCE(sum(value), 0.0) AS sum \
        FROM transactions \
        GROUP BY strftime('%Y%m', date)\
        ORDER BY year_month ASC;\
        "
    ).fetch_all(&**pool).await {
        Ok(aggs) => Ok(Json(aggs)),
        Err(e) => Err(e.to_string()),
    }
}
