use crate::helpers::{format_pairs, hash_seed, log_error, pair_members};
use crate::types::Context;
use anyhow::Result;

async fn handle_create_pairing(ctx: Context<'_>, seed_str: String) -> Result<String> {
    let seed = hash_seed(&seed_str);

    let pairs = pair_members(ctx, seed).await?;
    let pairs_str = format_pairs(&pairs);
    let key = format!(
        "{}_{}",
        seed_str,
        crate::helpers::checksum_pairing(seed, &pairs)
    );
    let num_members: usize = pairs.iter().map(|p| p.len()).sum();
    Ok(format!(
        "{pairs_str}\nTotal paired members: {num_members}\nTo send this pairing, use this key: `{key}`"
    ))
}

/// Generate a potential pairing of users who have reacted to a message
#[poise::command(
    slash_command,
    track_edits,
    hide_in_help,
    ephemeral,
    required_permissions = "ADMINISTRATOR",
    aliases("pair"),
    on_error = "log_error"
)]
pub async fn create_pairing(
    ctx: Context<'_>,
    #[description = "A seed to use for the generated pairing (for example, use the current date)."]
    seed: String,
) -> Result<()> {
    ctx.defer_ephemeral().await?;
    let resp = handle_create_pairing(ctx, seed)
        .await
        .unwrap_or_else(|e| format!("Error: {}", e));
    println!("{resp}");
    ctx.say(resp).await?;
    Ok(())
}
