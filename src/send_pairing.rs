use crate::helpers::{checksum_pairing, format_pairs, hash_seed, pair_members};
use crate::types::Context;
use crate::{log_error, ROLE_NAME};
use anyhow::{bail, ensure, Context as _, Error, Result};
use poise::futures_util::future::try_join_all;
use serenity::all::GuildChannel;

/// Run the /send_pairing command
async fn handle_send_pairing(
    ctx: Context<'_>,
    key: String,
    channel: GuildChannel,
) -> Result<String> {
    println!("{} used /send_pairing", ctx.author());

    let guild = ctx
        .guild()
        .context("This command must be called from a guild (server).")?
        .clone();
    let Some(role) = guild.role_by_name(ROLE_NAME) else {
        bail!("Could not find a role with name `{ROLE_NAME}`");
    };
    let Some((seed_str, checksum)) = key.rsplit_once("_") else {
        bail!("Invalid key. Please make sure you only use keys returned by /create_pairing.")
    };

    let seed = hash_seed(&seed_str);

    let pairs = pair_members(ctx, seed).await?;
    let pairs_str = format_pairs(&pairs);
    ensure!(
        checksum_pairing(seed, &pairs) == checksum,
        "Key mismatch. This can happen if you typed the key incorrectly, or the members with the \
        matchy meetups role have changed since this key was generated. Please call /create_pairing \
        again to get a new key."
    );

    channel
        .say(
            &ctx,
            format!(
                "Hey <@&{}>, here are the pairings for the next round of matchy meetups!\n\n{}",
                role.id, pairs_str
            ),
        )
        .await?;

    let mut messages_sent = 0;

    for pair in pairs {
        for user in &pair {
            let pairing: Vec<_> = pair.iter().filter(|u| *u != user).collect();
            let pairing_str = try_join_all(pairing.iter().map(|uid| {
                return async {
                    let u = uid.to_user(&ctx).await?;
                    Ok::<String, Error>(format!(
                        "<@{}> ({})",
                        u.id,
                        u.global_name.unwrap_or(u.name)
                    ))
                };
            }))
            .await
            .context("Unable to fetch names for user ids")?
            .join(" and ");

            let message_str = format!("Hey, thanks for joining ICSSC's Matchy Meetups. Your pairing \
                 for this round is here! Please take this opportunity to reach out to them and \
                 schedule some time to hang out in the next two weeks. \
                 Don't forget to send pics to https://discord.com/channels/760915616793755669/1199228930222194779 \
                 while you're there, and I hope you enjoy!\n\
                 \t\t\t\t\t\t\t \\- Jeffrey \n\n\n\
                 **Your pairing is with:** {pairing_str}\n\n\
                 _(responses here will not be seen; please message Jeffrey directly if you have any questions)_");
            let _ = user
                .create_dm_channel(&ctx)
                .await?
                .say(&ctx, message_str)
                .await;

            messages_sent += 1;
        }
    }
    println!("Messaged {messages_sent} users.");
    Ok(format!("Successfully messaged {messages_sent} users."))
}

/// Send a message to each member of the pairing.
#[poise::command(
    slash_command,
    track_edits,
    hide_in_help,
    required_permissions = "ADMINISTRATOR",
    on_error = "log_error"
)]
pub async fn send_pairing(
    ctx: Context<'_>,
    #[description = "A pairing key returned by /create_pairing."] key: String,
    #[description = "Channel to send a summary message with all the pairings in"]
    summary_channel: GuildChannel,
) -> Result<(), Error> {
    ctx.defer().await?;
    let resp = handle_send_pairing(ctx, key, summary_channel)
        .await
        .unwrap_or_else(|e| format!("Error: {}", e));
    println!("{resp}");
    ctx.say(resp).await?;
    Ok(())
}
