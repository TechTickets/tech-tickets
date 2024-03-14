use serenity::all::{EditMember, GuildId, Member, RoleId};
use tickets_common::requests::staff::Login;
use tickets_common::requests::SdkCall;

use crate::cache::roles::RolePurpose;
use crate::commands::CommandContext;
use crate::interactions::Interactable;
use crate::response;

pub async fn ensure_role(
    http: &serenity::http::Http,
    guild_id: GuildId,
    member: &Member,
    staff_role: RoleId,
) {
    let mut member_roles = member.roles.to_vec();

    let needs_role = !member_roles.iter().any(|r| *r == staff_role);

    if needs_role {
        member_roles.push(staff_role);

        if let Err(err) = guild_id
            .edit_member(
                http,
                member.user.id,
                EditMember::new().roles(member_roles.as_slice()),
            )
            .await
        {
            log::error!("Error applying role: {:#?}", err);
        }
    }
}

pub async fn promote_staff(mut command: CommandContext<'_>) -> anyhow::Result<()> {
    let promoted_user = command
        .pop_command_arg("user")?
        .as_user_id()
        .expect("User did not decode as UserId.");

    let purpose_arg = command.pop_command_arg("role")?;
    let purpose_str = purpose_arg
        .as_str()
        .expect("Role did not decode as string.");
    let purpose: RolePurpose = purpose_str.to_string().try_into()?;

    let staff_handle = command.get_staff_handle(promoted_user).await;

    let guild_id = command.require_guild_id()?;
    let member = guild_id.member(command.http(), promoted_user).await?;

    let role = command
        .state()
        .shared_state
        .roles_cache
        .get_role_id(guild_id, purpose)
        .await;
    let role = if let Some(role) = role {
        role
    } else {
        command
            .respond(
                response! {
                    message {
                        ephemeral: true,
                        message: "Role not found. Please setup your discord properly."
                    }
                },
                vec![],
            )
            .await?;

        return Ok(());
    };

    ensure_role(&command.interaction.ctx.http, guild_id, &member, role).await;

    Login::call(&staff_handle).await?;

    command
        .respond(
            response! {
                message {
                    ephemeral: true,
                    message: "Staff promoted successfully."
                }
            },
            vec![],
        )
        .await?;

    Ok(())
}
