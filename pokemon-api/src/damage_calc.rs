use crate::damage_class::DamageClass;
use crate::pkm_move::PkmMove;
use crate::pokemon_in_battle::PokemonInBattle;
use crate::stats::Stats;
use crate::turn_outcome::TurnStepType;
use async_graphql::Context;
use rand::Rng;

/// Calculates damage according to the gen 8 damage formula: https://bulbapedia.bulbagarden.net/wiki/Damage
pub async fn calculate(
    ctx: &Context<'_>,
    attacker: &PokemonInBattle,
    move_used: &PkmMove,
    defender: &PokemonInBattle,
) -> async_graphql::Result<(u32, TurnStepType)> {
    if let Some(accuracy) = move_used.accuracy {
        if rand::random::<f32>() > accuracy as f32 / 100.0 {
            return Ok((0, TurnStepType::Missed));
        }
    }

    let move_type = move_used.pkm_type(ctx).await?;
    let (a, d) = match move_used.damage_class().await? {
        DamageClass::Physical => (
            attacker.stat(ctx, Stats::Atk).await?,
            defender.stat(ctx, Stats::Def).await?,
        ),
        DamageClass::Special => (
            attacker.stat(ctx, Stats::SAtk).await?,
            defender.stat(ctx, Stats::SDef).await?,
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
        .pokemon(ctx)
        .await?
        .species(ctx)
        .await?
        .pkm_type(ctx)
        .await?
        .iter()
        .any(|typ| &move_type == typ);
    let stab = if has_stab { 1.5 } else { 1.0 };
    let effectiveness = move_type
        .get_type_efficacy(ctx, &defender.pokemon(ctx).await?.species(ctx).await?.pkm_type(ctx).await?)
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
