use std::error::Error;

use crate::pkm_move::PkmMove;
use crate::primitive_types::PkmMoveId;
use async_graphql::{
    http::GraphiQLSource, ComplexObject, Context, EmptyMutation, EmptySubscription, Object, Schema,
};
use async_graphql_poem::GraphQL;
use poem::{get, handler, listener::TcpListener, web::Html, IntoResponse, Route, Server};
use species::Species;
use sqlx::sqlite::SqlitePoolOptions;
use crate::owned_pokemon::OwnedPokemon;

mod primitive_types;
mod species;
mod pkm_type;
mod pkm_stats;
mod pkm_move;
mod move_effect;
mod owned_pokemon;
mod nature;
mod ability;
mod damage_calc;
mod damage_class;

struct Query;

#[Object]
impl Query {
    async fn pokemon_species(
        &self,
        ctx: &Context<'_>,
        id: i64,
    ) -> async_graphql::Result<Species> {
        Species::get(ctx, id).await
    }

    async fn random_pokemon(&self, ctx: &Context<'_>) -> async_graphql::Result<OwnedPokemon> {
        OwnedPokemon::random(ctx).await
    }

    async fn moves(
        &self,
        ctx: &Context<'_>,
        id: PkmMoveId,
    ) -> async_graphql::Result<PkmMove> {
        PkmMove::get(ctx, id).await
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
