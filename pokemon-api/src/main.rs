use crate::pkm_move::PkmMove;
use crate::primitive_types::{PkmMoveId, RealisedId};
use crate::realised_pokemon::RealisedPokemon;
use crate::singles_battle::SinglesBattle;
use async_graphql::{
    http::GraphiQLSource, ComplexObject, Context, EmptySubscription, Object, Schema,
};
use async_graphql_poem::GraphQL;
use poem::{get, handler, listener::TcpListener, web::Html, IntoResponse, Route, Server};
use species::Species;
use sqlx::migrate::Migrator;
use sqlx::sqlite::SqlitePoolOptions;
use std::error::Error;

mod ability;
mod damage_calc;
mod damage_class;
mod move_effect;
mod nature;
mod pkm_move;
mod pkm_stats;
mod pkm_type;
mod pokemon_in_battle;
mod primitive_types;
mod realised_pokemon;
mod singles_battle;
mod species;
mod turn_choice;

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

    async fn battle(&self, ctx: &Context<'_>, id: i64) -> async_graphql::Result<SinglesBattle> {
        SinglesBattle::get(ctx, id).await
    }
}

struct Mutation;

#[Object]
impl Mutation {
    async fn random_pokemon(&self, ctx: &Context<'_>) -> async_graphql::Result<RealisedPokemon> {
        let pkm = RealisedPokemon::random(ctx).await?;
        pkm.insert(ctx).await?;
        Ok(pkm)
    }

    async fn start_battle(
        &self,
        ctx: &Context<'_>,
        team_a: Vec<RealisedId>,
        team_b: Vec<RealisedId>,
    ) -> async_graphql::Result<SinglesBattle> {
        SinglesBattle::insert(ctx, team_a, team_b).await
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
