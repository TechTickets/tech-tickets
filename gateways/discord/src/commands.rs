mod management;

pub(crate) use management::bootstrap::{bootstrap_callback, BOOTSTRAP_CALLBACK_ID};

use std::collections::HashMap;
use std::fmt::Display;

use crate::cache::roles::RolePurpose;
use crate::impl_interactable;
use crate::interactions::InteractionContext;
use crate::state::DiscordAppState;
use serenity::all::{
    CommandDataOptionValue, CommandDataResolved, CommandInteraction, CommandOptionType, Context,
    CreateCommand, CreateCommandOption, CreateInteractionResponse,
    CreateInteractionResponseMessage, GuildId, Permissions,
};
use tickets_common::requests::staff::GuildPurpose;

const PROMOTE_STAFF_COMMAND_NAME: &str = "promote-staff";
const CREATE_APP_COMMAND_NAME: &str = "create-app";
const BIND_GUILD_COMMAND_NAME: &str = "bind-guild";
const DISPOSE_COMMAND_NAME: &str = "dispose";
const BOOTSTRAP_COMMAND_NAME: &str = "bootstrap";

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum CommandType {
    PromoteStaff,
    CreateApp,
    BindGuild,
    Dispose,
    Bootstrap,
}

impl From<String> for CommandType {
    fn from(command_name: String) -> Self {
        match command_name.as_str() {
            PROMOTE_STAFF_COMMAND_NAME => CommandType::PromoteStaff,
            CREATE_APP_COMMAND_NAME => CommandType::CreateApp,
            BIND_GUILD_COMMAND_NAME => CommandType::BindGuild,
            DISPOSE_COMMAND_NAME => CommandType::Dispose,
            BOOTSTRAP_COMMAND_NAME => CommandType::Bootstrap,
            _ => unreachable!(),
        }
    }
}

impl Display for CommandType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandType::PromoteStaff => write!(f, "{}", PROMOTE_STAFF_COMMAND_NAME),
            CommandType::CreateApp => write!(f, "{}", CREATE_APP_COMMAND_NAME),
            CommandType::BindGuild => write!(f, "{}", BIND_GUILD_COMMAND_NAME),
            CommandType::Dispose => write!(f, "{}", DISPOSE_COMMAND_NAME),
            CommandType::Bootstrap => write!(f, "{}", BOOTSTRAP_COMMAND_NAME),
        }
    }
}

pub struct CommandArgs {
    pub resolved: CommandDataResolved,
    options: HashMap<String, CommandDataOptionValue>,
}

pub struct CommandContext<'a> {
    interaction: InteractionContext<'a>,
    pub args: CommandArgs,
}

impl<'a> CommandContext<'a> {
    pub fn new<'b>(
        app_state: &'b DiscordAppState,
        ctx: &'b Context,
        serenity_command: CommandInteraction,
    ) -> CommandContext<'b> {
        CommandContext {
            interaction: InteractionContext::new(app_state, ctx, &serenity_command),
            args: CommandArgs {
                resolved: serenity_command.data.resolved,
                options: serenity_command
                    .data
                    .options
                    .into_iter()
                    .map(|option| (option.name, option.value))
                    .collect(),
            },
        }
    }

    pub fn pop_command_arg(&mut self, arg_name: &str) -> anyhow::Result<CommandDataOptionValue> {
        self.args
            .options
            .remove(arg_name)
            .ok_or_else(|| anyhow::anyhow!("Missing required argument: {}", arg_name))
    }
}

impl_interactable!(for CommandContext::<'a>.interaction);

impl CommandType {
    pub async fn handle_command_interaction<'a>(&self, command: CommandContext<'a>) {
        let (ctx_copy, id, token) = (
            command.interaction.ctx,
            command.interaction.interaction_id,
            command.interaction.token.to_string(),
        );
        if let Err(err) = match self {
            CommandType::PromoteStaff => management::promote_staff::promote_staff(command).await,
            CommandType::CreateApp => management::create_app::create_app(command).await,
            CommandType::BindGuild => management::bind_guild::bind_guild(command).await,
            CommandType::Bootstrap => management::bootstrap::bootstrap(command).await,
            _ => unimplemented!(),
        } {
            if let Err(err) = ctx_copy
                .http
                .create_interaction_response(
                    id,
                    &token,
                    &CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .ephemeral(true)
                            .content(format!(
                                "An error occurred while processing your command. {}",
                                err
                            )),
                    ),
                    vec![],
                )
                .await
            {
                log::error!("Failed to send error response: {}", err);
            }
        }
    }
}

pub async fn ready(ctx: &Context, guild_id: GuildId) -> anyhow::Result<()> {
    // management commands
    guild_id
        .create_command(
            &ctx.http,
            CreateCommand::new(PROMOTE_STAFF_COMMAND_NAME)
                .description("Promote a new staff member.")
                .add_option(
                    CreateCommandOption::new(
                        CommandOptionType::User,
                        "user",
                        "The user to promote.",
                    )
                    .required(true),
                )
                .add_option(
                    CreateCommandOption::new(
                        CommandOptionType::String,
                        "role",
                        "The role to promote the user to.",
                    )
                    .add_string_choice(
                        RolePurpose::Management.role_name(),
                        RolePurpose::Management.to_string(),
                    )
                    .add_string_choice(
                        RolePurpose::Staff.role_name(),
                        RolePurpose::Staff.to_string(),
                    )
                    .required(true),
                )
                .default_member_permissions(Permissions::ADMINISTRATOR),
        )
        .await?;

    guild_id
        .create_command(
            &ctx.http,
            CreateCommand::new(CREATE_APP_COMMAND_NAME)
                .description("Create a new application.")
                .add_option(
                    CreateCommandOption::new(
                        CommandOptionType::String,
                        "name",
                        "The name of the application (must be unique).",
                    )
                    .required(true),
                )
                .default_member_permissions(Permissions::ADMINISTRATOR),
        )
        .await?;

    guild_id
        .create_command(
            &ctx.http,
            CreateCommand::new(BIND_GUILD_COMMAND_NAME)
                .description("Bind a guild to an app with a purpose.")
                .add_option(
                    CreateCommandOption::new(
                        CommandOptionType::String,
                        "purpose",
                        "The purpose to bind the guild to.",
                    )
                    .add_string_choice("Management Discord", GuildPurpose::Management.to_string())
                    .add_string_choice("Consumer Discord", GuildPurpose::Consumer.to_string())
                    .required(true),
                )
                .add_option(
                    CreateCommandOption::new(
                        CommandOptionType::String,
                        "app_id",
                        "The app to be bound.",
                    )
                    .required(true),
                )
                .default_member_permissions(Permissions::ADMINISTRATOR),
        )
        .await?;

    guild_id
        .create_command(
            &ctx.http,
            CreateCommand::new(DISPOSE_COMMAND_NAME)
                .description("Useful for requesting configuration based updates.")
                .default_member_permissions(Permissions::ADMINISTRATOR),
        )
        .await?;

    guild_id
        .create_command(
            &ctx.http,
            CreateCommand::new(BOOTSTRAP_COMMAND_NAME)
                .description("Bootstrap the discord.")
                .default_member_permissions(Permissions::ADMINISTRATOR),
        )
        .await?;

    Ok(())
}
