use crate::damage_class::DamageClass;
use crate::pkm_move::PkmMove;
use crate::realised_pokemon::RealisedPokemon;
use crate::turn_outcome::TurnStepType;
use async_graphql::Context;
use rand::Rng;

/// Calculates damage according to the gen 8 damage formula: https://bulbapedia.bulbagarden.net/wiki/Damage
pub async fn calculate(
    ctx: &Context<'_>,
    attacker: &RealisedPokemon,
    move_used: &PkmMove,
    defender: &RealisedPokemon,
) -> async_graphql::Result<(u32, TurnStepType)> {
    if let Some(accuracy) = move_used.accuracy {
        if rand::random::<f32>() > accuracy as f32 / 100.0 {
            return Ok((0, TurnStepType::Missed));
        }
    }

    let move_type = move_used.pkm_type(ctx).await?;
    let attacker_stats = attacker.species(ctx).await?.pkm_stats(ctx).await?;
    let defender_stats = defender.species(ctx).await?.pkm_stats(ctx).await?;
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
        .species(ctx)
        .await?
        .pkm_type(ctx)
        .await?
        .iter()
        .any(|typ| &move_type == typ);
    let stab = if has_stab { 1.5 } else { 1.0 };
    let effectiveness = move_type
        .get_type_efficacy(ctx, &defender.species(ctx).await?.pkm_type(ctx).await?)
        .await?;
    if effectiveness <= 0.001 {
        return Ok((0, TurnStepType::Immune));
    }
    let burn = 1.0; // TODO burn

    let unmodified = dr((dr(2 * level, 5) + 2) * power * dr(a, d), 50) + 2;
    let factored =
        unmodified as f64 * targets * weather * critical * random * stab * effectiveness * burn;
    Ok((factored as u32, TurnStepType::Damage))
}

fn dr(numerator: i64, denominator: i64) -> i64 {
    (numerator + denominator / 2) / denominator
}
