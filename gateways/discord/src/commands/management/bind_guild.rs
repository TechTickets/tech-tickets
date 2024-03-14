use std::str::FromStr;

use tickets_common::requests::staff::{GuildPurpose, LinkDiscordGuildId, LinkDiscordGuildIdBody};
use tickets_common::requests::SdkInvokeWithBody;
use uuid::Uuid;

use crate::commands::CommandContext;
use crate::interactions::Interactable;
use crate::response;

pub async fn bind_guild(mut command: CommandContext<'_>) -> anyhow::Result<()> {
    let staff_handle = command.get_staff_handle(command.user().id).await;

    let guild_id = command.require_guild_id()?;

    let app_id_arg = command.pop_command_arg("app_id")?;
    let app_id_str = app_id_arg.as_str().expect("app_id not decoded as string");

    let guild_purpose_arg = command.pop_command_arg("purpose")?;
    let guild_purpose = guild_purpose_arg
        .as_str()
        .map(|s| GuildPurpose::try_from(s.to_string()))
        .expect("purpose not decoded as string")?;

    let app_id = match Uuid::from_str(app_id_str) {
        Ok(app_id) => app_id,
        Err(_) => {
            anyhow::bail!("app_id is not a valid UUID");
        }
    };

    LinkDiscordGuildId::invoke_with_body(
        &staff_handle,
        LinkDiscordGuildIdBody {
            app_id,
            guild_id: guild_id.get(),
            guild_purpose,
        },
    )
    .await?;

    command
        .respond(
            response! {
                message {
                    ephemeral: true,
                    message: "Guild bound successfully."
                }
            },
            vec![],
        )
        .await?;

    Ok(())
}
