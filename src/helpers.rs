use crate::pair::pair;
use crate::types::{Context, Data};
use crate::ROLE_NAME;
use anyhow::{bail, Context as _, Error};
use itertools::Itertools;
use serenity::all::UserId;
use std::hash::{DefaultHasher, Hash, Hasher};

pub async fn log_error(error: poise::FrameworkError<'_, Data, Error>) {
    println!("Error: {:?}", error);
}

pub async fn pair_members(ctx: Context<'_>, seed: u64) -> anyhow::Result<Vec<Vec<UserId>>> {
    let guild = ctx
        .guild()
        .context("This command must be called from a guild (server).")?
        .clone();
    let Some(role) = guild.role_by_name(ROLE_NAME) else {
        bail!("Could not find a role with name `{ROLE_NAME}`");
    };
    let participants: Vec<_> = guild
        .members(&ctx, None, None)
        .await?
        .iter()
        .filter(|u| u.roles.iter().contains(&role.id))
        .map(|p| p.user.id)
        .collect();

    if participants.len() <= 1 {
        bail!(
            "Need at least two members to create a pairing (found {} member{} with role <@&{}>).",
            participants.len(),
            if participants.len() == 1 { "" } else { "s" },
            role.id
        );
    }
    Ok(pair(participants, seed))
}

pub fn hash_seed(seed: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);
    hasher.finish()
}

/// Generates a short checksum for a given seed & pairing, which can be used to verify that nothing
/// has changed between multiple uses.
pub fn checksum_pairing(seed: u64, pairs: &Vec<Vec<UserId>>) -> String {
    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);
    pairs.hash(&mut hasher);
    let hex = format!("{:x}", hasher.finish());
    hex[..8].to_string()
}
fn format_id(id: &UserId) -> String {
    format!("<@{id}>")
}
pub fn format_pairs(pairs: &Vec<Vec<UserId>>) -> String {
    pairs
        .iter()
        .map(|p| {
            p.iter().take(p.len() - 1).map(format_id).join(", ")
                + if p.len() > 2 { ", and " } else { " and " }
                + &format_id(p.last().expect("pairings should be non-empty"))
        })
        .collect::<Vec<_>>()
        .join("\n")
}
