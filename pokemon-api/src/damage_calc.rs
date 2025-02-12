use crate::damage_class::DamageClass;
use crate::owned_pokemon::OwnedPokemon;
use crate::pkm_move::PkmMove;
use async_graphql::Context;
use rand::Rng;
use std::any::Any;

pub async fn calculate(
    ctx: &Context<'_>,
    attacker: OwnedPokemon,
    move_used: PkmMove,
    defender: OwnedPokemon,
) -> async_graphql::Result<u32> {
    let move_type = move_used.pkm_type(ctx).await?;
    let attacker_stats = attacker.species.pkm_stats(ctx).await?;
    let defender_stats = defender.species.pkm_stats(ctx).await?;
    let (a, d) = match move_used.damage_class().await? {
        DamageClass::Physical => (attacker_stats.atk.base_stat, defender_stats.def.base_stat),
        DamageClass::Special => (
            attacker_stats.s_atk.base_stat,
            defender_stats.s_def.base_stat,
        ),
        DamageClass::Status => (0, 1),
    };

    let level = 50;
    let power = move_used.power.unwrap_or(0);
    let targets = 1.0; // TODO for 2v2
    let weather = 1.0; // TODO weather
    let critical = 1.0; // TODO for critical hits
    let random = rand::thread_rng().gen_range(85..=100) as f64 / 100.0;
    let has_stab = attacker
        .species
        .pkm_type(ctx)
        .await?
        .iter()
        .any(|typ| &move_type == typ);
    let stab = if has_stab { 1.5 } else { 1.0 };
    let effectiveness = move_type
        .get_type_efficacy(ctx, &defender.species.pkm_type(ctx).await?)
        .await?;
    let burn = 1.0; // TODO burn

    let unmodified = dr((dr(2 * level, 5) + 2) * power * dr(a, d), 50) + 2;
    let factored = unmodified as f64 * targets * weather * critical * random * stab * effectiveness * burn;
    Ok(factored as u32)
}

fn dr(numerator: i64, denominator: i64) -> i64 {
    (numerator + denominator / 2) / denominator
}
