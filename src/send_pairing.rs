use crate::log_error;
use crate::pair::pair;
use crate::types::{Context, Error};
use poise::futures_util::future::join_all;

/// Send a message to each member of the pairing.
#[poise::command(
    slash_command,
    track_edits,
    hide_in_help,
    ephemeral,
    required_permissions = "ADMINISTRATOR",
    on_error = "log_error"
)]
pub async fn send_pairing(
    ctx: Context<'_>,
    #[description = "Link to a message with reactions -- a pairing will be made between users who reacted."]
    message_link: String,
    #[description = "Temproary confirmation key"] temporary_confirm: Option<String>,
) -> Result<(), Error> {
    // TODO: remove this (once we can validate that we don't resend pairings)
    //  a better architecture is create_pairing() generates a pairing ID
    //  send_pairing sends the ID and checks that it hasn't been sent before
    if temporary_confirm.filter(|s| *s == "confirmiamjeffrey") == None {
        ctx.say("Danger! This command will send messages to each paired member! Pass in the confirm key to confirm.")
            .await?;
        return Ok(());
    }
    println!("{message_link}");
    let message_id = message_link
        .split("/")
        .last()
        .unwrap()
        .trim()
        .parse::<u64>();
    let resp = match message_id {
        Ok(id) => {
            let message = ctx.channel_id().message(&ctx, id).await?;

            let reactions =
                join_all(message.reactions.iter().map(|r| {
                    message.reaction_users(&ctx, r.reaction_type.clone(), Some(100), None)
                }))
                .await
                .iter()
                .filter_map(|r| r.as_ref().ok())
                .flatten()
                .map(|u| u.id)
                .collect::<Vec<_>>();
            ctx.defer_ephemeral().await;
            if reactions.len() <= 1 {
                format!(
                    "Need at least two reactions to create a pairing (message has {} reaction{}).",
                    reactions.len(),
                    if reactions.len() % 2 == 0 { "" } else { "s" }
                )
            } else {
                let mut messages_sent = 0;
                let pairs = pair(reactions);
                for pair in pairs {
                    for user in &pair {
                        let pairing: Vec<_> = pair.iter().filter(|u| *u != user).collect();
                        let pairing_str = if pairing.len() == 1 {
                            // TODO: parallelize these async requests and check API ratelimiting
                            // TODO: the display name should be whatever the server
                            let pairing1 = pairing[0].to_user(&ctx).await?;
                            format!(
                                "<@{}> ({})",
                                pairing1.id,
                                pairing1.global_name.unwrap_or(pairing1.name)
                            )
                        } else {
                            let pairing1 = pairing[0].to_user(&ctx).await?;
                            let pairing2 = pairing[1].to_user(&ctx).await?;
                            format!(
                                "<@{}> ({}) and <${}> ({})",
                                pairing1.id,
                                pairing1.global_name.unwrap_or(pairing1.name),
                                pairing2.id,
                                pairing2.global_name.unwrap_or(pairing2.name)
                            )
                        };

                        let message_str = format!("Hey, thanks for joining ICSSC's Matchy Meetups. Your pairing \
                             for this round is with {pairing_str}! Please take this opportunity to reach out to them and \
                             schedule some time to hang out in the next two weeks. \
                             Don't forget to send pics to https://discord.com/channels/760915616793755669/1199228930222194779 \
                             while you're there, and I hope you enjoy!\n\n\
                             \t\t\t \\- Jeffrey \n\n\
                             _(responses here will not be seen; please message Jeffrey directly if there are any issues)_");
                        let _ = user
                            .create_dm_channel(&ctx)
                            .await
                            .unwrap()
                            .say(&ctx, message_str)
                            .await;

                        messages_sent += 1;
                    }
                }
                format!("Successfully messaged {messages_sent} users.")
            }
        }
        Err(_) => "Error: unable to parse link.".to_string(),
    };
    println!("{resp}");
    ctx.say(resp).await?;
    Ok(())
}
