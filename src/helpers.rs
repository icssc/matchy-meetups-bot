use crate::pair::pair;
use crate::types::{Context, Data};
use crate::ROLE_NAME;
use anyhow::{bail, Context as _, Error, Result};
use itertools::Itertools;
use serenity::all::{Guild, RoleId, UserId};
use std::hash::{DefaultHasher, Hash, Hasher};

/// Logs an error to stdout.
pub async fn log_error(error: poise::FrameworkError<'_, Data, Error>) {
    println!("Error: {:?}", error);
}

/// Returns a vector of all guild members with the specified role ID.
async fn guild_members_with_role(
    ctx: &Context<'_>,
    guild: &Guild,
    role_id: RoleId,
) -> Result<Vec<UserId>> {
    // max number of pages to try to fetch (to avoid infinite loops in the event of the server
    // response format changing in a way that breaks the end-of-page detection)
    const MAX_PAGES: u64 = 20;

    // maximum number of members to request per page
    const PAGE_LIMIT: u64 = 1000;

    let mut last_member = None;
    let mut members_with_role = Vec::new();

    for _ in 0..MAX_PAGES {
        let page = guild.members(&ctx, Some(PAGE_LIMIT), last_member).await?;

        members_with_role.extend(
            page.iter()
                .filter(move |u| u.roles.iter().contains(&role_id))
                .map(|p| p.user.id),
        );

        if page.len() < PAGE_LIMIT as usize {
            break;
        }
        last_member = Some(
            page.last()
                .expect("page is never empty here if PAGE_LIMIT > 0")
                .user
                .id,
        );
    }

    Ok(members_with_role)
}

/// Pairs members with ROLE_NAME in the guild together. The result is a vector of "pairs"
/// If the number of members to be paired is odd, one "pair" will have three members.
pub async fn pair_members(ctx: Context<'_>, seed: u64) -> Result<Vec<Vec<UserId>>> {
    let guild = ctx
        .guild()
        .context("This command must be called from a guild (server).")?
        .clone();
    let Some(role) = guild.role_by_name(ROLE_NAME) else {
        bail!("Could not find a role with name `{ROLE_NAME}`");
    };
    let participants = guild_members_with_role(&ctx, &guild, role.id).await?;
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

/// Hashes a string into a u64 that can be used as a seed
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

/// Formats an ID for display as a ping in discord
fn format_id(id: &UserId) -> String {
    format!("<@{id}>")
}

/// Formats a pairing into a string suitable for a discord message
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
