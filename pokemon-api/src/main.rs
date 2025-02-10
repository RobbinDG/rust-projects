use std::error::Error;

use async_graphql::{
    http::GraphiQLSource, ComplexObject, Context, EmptyMutation, EmptySubscription, Object, Schema,
};
use async_graphql_poem::GraphQL;
use poem::{get, handler, listener::TcpListener, web::Html, IntoResponse, Route, Server};
use primitive_types::PkmId;
use species::Species;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Pool, Sqlite};

mod primitive_types;
mod species;
mod pkm_type;
mod pkm_stats;
mod pkm_move;

struct Query;

#[Object]
impl Query {
    async fn pokemon_species(
        &self,
        ctx: &Context<'_>,
        id: i64,
    ) -> async_graphql::Result<Option<Species>> {
        let pool = ctx.data::<Pool<Sqlite>>()?;

        let result: Option<(PkmId, String, Option<f64>)> = sqlx::query_as(
            "\
            SELECT id, identifier, evolves_from_species_id \
            FROM pokemon_species s WHERE id = $1
            ",
        )
        .bind(id)
        .fetch_one(pool)
        .await
        .ok();

        Ok(result.map(|(id, identifier, evolves_from)| {
            Species::new(id, identifier, evolves_from.map(|f| f as PkmId))
        }))
    }
}

#[handler]
async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().finish())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite://./pokemon.db")
        .await?;

    // create the schema
    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .data(pool)
        .finish();

    println!("{}", schema.sdl());

    // start the http server
    let app = Route::new().at("/", get(graphiql).post(GraphQL::new(schema)));
    println!("GraphiQL: http://localhost:8000");
    Server::new(TcpListener::bind("0.0.0.0:8000"))
        .run(app)
        .await?;
    Ok(())
}
