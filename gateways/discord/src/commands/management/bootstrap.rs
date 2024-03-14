use serenity::all::{
    ChannelId, ChannelType, Context, CreateActionRow, CreateModal, EditRole, InputTextStyle,
    PermissionOverwrite, PermissionOverwriteType, Permissions, RoleId,
};
use serenity::builder::{CreateInputText, CreateInteractionResponse};
use tickets_common::requests::staff::{CreateApp, GuildPurpose, LinkDiscordGuildId, Login};
use tickets_common::requests::{SdkCall, SdkCallWithBody, SdkInvokeWithBody};

use crate::cache::channels::ChannelPurpose;
use crate::cache::roles::RolePurpose;
use crate::commands::management::promote_staff::ensure_role;
use crate::commands::CommandContext;
use crate::interactions::Interactable;
use crate::modals::ModalContext;

pub(crate) const BOOTSTRAP_CALLBACK_ID: &str = "bootstrap";
const BOOTSTRAP_APP_NAME_COMPONENT_ID: &str = "app_name";

pub async fn bootstrap(command_context: CommandContext<'_>) -> anyhow::Result<()> {
    command_context
        .respond(
            CreateInteractionResponse::Modal(
                CreateModal::new(BOOTSTRAP_CALLBACK_ID, "Tech Tickets Bootstrap").components(vec![
                    CreateActionRow::InputText(CreateInputText::new(
                        InputTextStyle::Short,
                        "App Name",
                        BOOTSTRAP_APP_NAME_COMPONENT_ID,
                    )),
                ]),
            ),
            vec![],
        )
        .await?;
    Ok(())
}

pub async fn bootstrap_callback(mut ctx: ModalContext<'_>) -> anyhow::Result<()> {
    let staff_handle = ctx.get_staff_handle(ctx.user().id).await;
    let app_name = ctx.pop_text_input(BOOTSTRAP_APP_NAME_COMPONENT_ID)?;
    let guild_id = ctx.require_guild_id()?;

    let mgmt_role = upsert_and_get_role(&ctx, guild_id, RolePurpose::Management).await?;
    let staff_role = upsert_and_get_role(&ctx, guild_id, RolePurpose::Staff).await?;

    // we might not be able to give the role to the user if they are a server admin/owner
    ensure_role(ctx.http(), guild_id, ctx.require_member()?, mgmt_role).await;

    Login::call(&staff_handle).await?;
    let created_app = CreateApp::call_with_body(
        &staff_handle,
        tickets_common::requests::staff::CreateAppBody { app_name },
    )
    .await?;

    LinkDiscordGuildId::invoke_with_body(
        &staff_handle,
        tickets_common::requests::staff::LinkDiscordGuildIdBody {
            app_id: created_app.app_id,
            guild_id: guild_id.get(),
            guild_purpose: GuildPurpose::Management,
        },
    )
    .await?;

    ctx.state()
        .shared_state
        .guild_cache
        .insert(guild_id, GuildPurpose::Management, created_app.app_id)
        .await;

    let (handler, serenity_ctx) = (ctx.interaction.app_state, ctx.interaction.ctx);

    init_management(handler, serenity_ctx, mgmt_role, guild_id).await?;
    init_staff(handler, serenity_ctx, mgmt_role, staff_role, guild_id).await?;

    ctx.respond(CreateInteractionResponse::Acknowledge, vec![])
        .await?;

    Ok(())
}

pub async fn upsert_and_get_role(
    ctx: &impl Interactable,
    guild_id: serenity::model::id::GuildId,
    role_purpose: RolePurpose,
) -> anyhow::Result<RoleId> {
    let role = ctx
        .state()
        .shared_state
        .roles_cache
        .get_role_id(guild_id, role_purpose)
        .await;

    let role = if let Some(role) = role {
        role
    } else {
        let role = guild_id
            .create_role(
                &ctx.http(),
                EditRole::new()
                    .name(role_purpose.role_name())
                    .hoist(true)
                    .colour(role_purpose.role_color())
                    .mentionable(false),
            )
            .await?;

        ctx.state()
            .shared_state
            .roles_cache
            .insert(guild_id, role_purpose, role.id)
            .await;

        role.id
    };

    Ok(role)
}

pub async fn init_staff(
    handler: &crate::state::DiscordAppState,
    ctx: &Context,
    mgmt_role: RoleId,
    staff_role: RoleId,
    guild_id: serenity::model::id::GuildId,
) -> anyhow::Result<()> {
    let category_id = create_staff_category(handler, ctx, mgmt_role, staff_role, guild_id).await?;
    create_staff_info_channel(handler, ctx, guild_id, category_id).await?;
    create_staff_bot_commands_channel(handler, ctx, mgmt_role, staff_role, guild_id, category_id)
        .await?;
    Ok(())
}

const STAFF_CATEGORY_STAFF_PERMS_BITS: u64 = Permissions::empty().bits()
    | Permissions::VIEW_CHANNEL.bits()
    | Permissions::SEND_MESSAGES.bits()
    | Permissions::ADD_REACTIONS.bits()
    | Permissions::ATTACH_FILES.bits()
    | Permissions::SEND_MESSAGES_IN_THREADS.bits()
    | Permissions::READ_MESSAGE_HISTORY.bits()
    | Permissions::USE_EXTERNAL_EMOJIS.bits();

pub async fn create_staff_category(
    handler: &crate::state::DiscordAppState,
    ctx: &Context,
    mgmt_role: RoleId,
    staff_role: RoleId,
    guild_id: serenity::model::id::GuildId,
) -> anyhow::Result<ChannelId> {
    handler
        .create_channel_for_guild(ctx, guild_id, ChannelPurpose::StaffCategory, |c| {
            c.kind(ChannelType::Category).position(998).permissions(
                vec![
                    PermissionOverwrite {
                        allow: Permissions::all(),
                        deny: Permissions::empty(),
                        kind: PermissionOverwriteType::Role(mgmt_role),
                    },
                    PermissionOverwrite {
                        allow: Permissions::default()
                            | Permissions::from_bits_retain(STAFF_CATEGORY_STAFF_PERMS_BITS),
                        deny: Permissions::empty(),
                        kind: PermissionOverwriteType::Role(staff_role),
                    },
                    PermissionOverwrite {
                        allow: Permissions::empty(),
                        deny: Permissions::all(),
                        kind: PermissionOverwriteType::Role(guild_id.everyone_role()),
                    },
                ]
                .into_iter(),
            )
        })
        .await
}

pub async fn create_staff_info_channel(
    handler: &crate::state::DiscordAppState,
    ctx: &Context,
    guild_id: serenity::model::id::GuildId,
    category_id: ChannelId,
) -> anyhow::Result<ChannelId> {
    handler
        .create_channel_for_guild(ctx, guild_id, ChannelPurpose::StaffInfo, |c| {
            c.kind(ChannelType::Text).category(category_id).position(1)
        })
        .await
}

pub async fn create_staff_bot_commands_channel(
    handler: &crate::state::DiscordAppState,
    ctx: &Context,
    mgmt_role: RoleId,
    staff_role: RoleId,
    guild_id: serenity::model::id::GuildId,
    category_id: ChannelId,
) -> anyhow::Result<ChannelId> {
    handler
        .create_channel_for_guild(ctx, guild_id, ChannelPurpose::StaffBotCommands, |c| {
            c.kind(ChannelType::Text)
                .category(category_id)
                .position(2)
                .permissions(
                    vec![
                        PermissionOverwrite {
                            allow: Permissions::all(),
                            deny: Permissions::empty(),
                            kind: PermissionOverwriteType::Role(mgmt_role),
                        },
                        PermissionOverwrite {
                            allow: Permissions::default()
                                | Permissions::from_bits_retain(STAFF_CATEGORY_STAFF_PERMS_BITS)
                                | Permissions::USE_APPLICATION_COMMANDS,
                            deny: Permissions::empty(),
                            kind: PermissionOverwriteType::Role(staff_role),
                        },
                        PermissionOverwrite {
                            allow: Permissions::empty(),
                            deny: Permissions::all(),
                            kind: PermissionOverwriteType::Role(guild_id.everyone_role()),
                        },
                    ]
                    .into_iter(),
                )
        })
        .await
}

pub async fn init_management(
    handler: &crate::state::DiscordAppState,
    ctx: &Context,
    mgmt_role: RoleId,
    guild_id: serenity::model::id::GuildId,
) -> anyhow::Result<()> {
    let category_id = create_management_category(handler, ctx, mgmt_role, guild_id).await?;

    create_management_info_channel(handler, ctx, guild_id, category_id).await?;
    create_management_bot_commands_channel(handler, ctx, guild_id, category_id).await?;

    Ok(())
}

pub async fn create_management_category(
    handler: &crate::state::DiscordAppState,
    ctx: &Context,
    mgmt_role: RoleId,
    guild_id: serenity::model::id::GuildId,
) -> anyhow::Result<ChannelId> {
    handler
        .create_channel_for_guild(ctx, guild_id, ChannelPurpose::ManagementCategory, |c| {
            c.kind(ChannelType::Category).position(999).permissions(
                vec![
                    PermissionOverwrite {
                        allow: Permissions::all(),
                        deny: Permissions::empty(),
                        kind: PermissionOverwriteType::Role(mgmt_role),
                    },
                    PermissionOverwrite {
                        allow: Permissions::empty(),
                        deny: Permissions::all(),
                        kind: PermissionOverwriteType::Role(guild_id.everyone_role()),
                    },
                ]
                .into_iter(),
            )
        })
        .await
}

pub async fn create_management_info_channel(
    handler: &crate::state::DiscordAppState,
    ctx: &Context,
    guild_id: serenity::model::id::GuildId,
    category_id: ChannelId,
) -> anyhow::Result<ChannelId> {
    handler
        .create_channel_for_guild(ctx, guild_id, ChannelPurpose::ManagementInfo, |c| {
            c.kind(ChannelType::Text).category(category_id).position(1)
        })
        .await
}

pub async fn create_management_bot_commands_channel(
    handler: &crate::state::DiscordAppState,
    ctx: &Context,
    guild_id: serenity::model::id::GuildId,
    category_id: ChannelId,
) -> anyhow::Result<ChannelId> {
    handler
        .create_channel_for_guild(ctx, guild_id, ChannelPurpose::ManagementBotCommands, |c| {
            c.kind(ChannelType::Text).category(category_id).position(2)
        })
        .await
}
