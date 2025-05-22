use rocket::serde::json::Json;
use rocket::serde::Serialize;
use rocket::State;
use sqlx::SqlitePool;

#[derive(Serialize, Debug)]
#[serde(crate = "rocket::serde")]
struct BreakdownItem {
    category: String,
    breakdown_value: f64,
}

#[derive(Serialize, Debug)]
#[serde(crate = "rocket::serde")]
struct MonthBreakdown {
    year_month: String,
    items: Vec<BreakdownItem>
}

#[get("/breakdowns/<year_month>")]
pub async fn get_breakdown(year_month: String, pool: &State<SqlitePool>) -> Result<Json<MonthBreakdown>, String> {
    match sqlx::query_as!(
        BreakdownItem,
        "SELECT COALESCE(pc.category, 'Other') AS category, COALESCE(sum(t.value), 0.0) AS breakdown_value \
        FROM transactions t \
        LEFT JOIN party_categories AS pc ON t.name_other = pc.party_name \
        WHERE strftime('%Y %m', date) = ? \
        GROUP BY pc.category \
        ORDER BY ABS(breakdown_value) DESC;\
        ",
        year_month,
    ).fetch_all(&**pool).await {
        Ok(items) => Ok(Json(MonthBreakdown {year_month, items})),
        Err(e) => Err(e.to_string()),
    }
}
