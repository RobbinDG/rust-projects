use crate::realised_pokemon::RealisedPokemon;
use crate::pkm_move::PkmMove;
use crate::primitive_types::PkmMoveId;
use async_graphql::{
    http::GraphiQLSource, ComplexObject, Context, EmptySubscription, Object, Schema,
};
use async_graphql_poem::GraphQL;
use poem::{get, handler, listener::TcpListener, web::Html, IntoResponse, Route, Server};
use species::Species;
use sqlx::migrate::Migrator;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Pool, Sqlite};
use std::error::Error;

mod ability;
mod damage_calc;
mod damage_class;
mod move_effect;
mod nature;
mod realised_pokemon;
mod pkm_move;
mod pkm_stats;
mod pkm_type;
mod primitive_types;
mod species;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

struct Query;

#[Object]
impl Query {
    async fn pokemon_species(&self, ctx: &Context<'_>, id: i64) -> async_graphql::Result<Species> {
        Species::get(ctx, id).await
    }

    async fn moves(&self, ctx: &Context<'_>, id: PkmMoveId) -> async_graphql::Result<PkmMove> {
        PkmMove::get(ctx, id).await
    }
}

struct Mutation;

#[Object]
impl Mutation {
    async fn random_pokemon(&self, ctx: &Context<'_>) -> async_graphql::Result<RealisedPokemon> {
        let pool = ctx.data::<Pool<Sqlite>>()?;
        let pkm = RealisedPokemon::random(ctx).await?;
        let x = sqlx::query!(
            "INSERT INTO realised_pokemon VALUES (?, ?, ?, ?, ?, ?, ?)",
            pkm.id,
            pkm.species.id,
            pkm.move_1.id,
            pkm.move_2.id,
            pkm.move_3.id,
            pkm.move_4.id,
            pkm.nature.id,
        )
        .execute(pool)
        .await?;
        Ok(pkm)
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

    MIGRATOR.run(&pool).await?;

    // create the schema
    let schema = Schema::build(Query, Mutation, EmptySubscription)
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
