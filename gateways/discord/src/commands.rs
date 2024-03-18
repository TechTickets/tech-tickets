use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use serenity::all::{
    CommandDataOptionValue, CommandDataResolved, CommandInteraction, CommandOptionType, Context,
    CreateCommand, CreateCommandOption, GuildId, Http, Permissions,
};

use auth::UserRole;
use errors::{ParsingError, TicketsResult};

use crate::guilds::GuildPurpose;
use crate::impl_interactable;
use crate::interactions::InteractionContext;
use crate::shared_state::SharedAppState;

mod bootstrap;
mod dispose;
mod promote_staff;

pub struct CommandArgs {
    pub resolved: CommandDataResolved,
    options: HashMap<String, CommandDataOptionValue>,
}

pub struct ParsedCommand {
    interaction: InteractionContext,
    command_name: String,
    pub args: CommandArgs,
}

impl ParsedCommand {
    pub fn new(
        app_state: SharedAppState,
        ctx: Context,
        serenity_command: CommandInteraction,
    ) -> ParsedCommand {
        ParsedCommand {
            interaction: InteractionContext::new(app_state, ctx, &serenity_command),
            command_name: serenity_command.data.name,
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

    pub fn pop_command_arg(&mut self, arg_name: &str) -> Option<CommandDataOptionValue> {
        self.args.options.remove(arg_name)
    }

    pub async fn execute(self) -> TicketsResult<()> {
        match self.command_name.to_string().try_into()? {
            CommandType::Bootstrap => bootstrap::run_command(self).await,
            CommandType::Dispose => dispose::run_command(self).await,
            CommandType::PromoteStaff => promote_staff::run_command(self).await,
        }
    }
}

impl_interactable!(for ParsedCommand.interaction);

pub enum CommandType {
    // management
    Bootstrap,
    Dispose,
    PromoteStaff,
}

impl Display for CommandType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandType::Bootstrap => write!(f, "bootstrap"),
            CommandType::PromoteStaff => write!(f, "promote-staff"),
            CommandType::Dispose => write!(f, "dispose"),
        }
    }
}

impl From<CommandType> for String {
    fn from(command_type: CommandType) -> Self {
        command_type.to_string()
    }
}

impl TryFrom<String> for CommandType {
    type Error = ParsingError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(match value.as_str() {
            "bootstrap" => CommandType::Bootstrap,
            "promote-staff" => CommandType::PromoteStaff,
            "dispose" => CommandType::Dispose,
            _ => Err(ParsingError::InvalidCommandType(value))?,
        })
    }
}

macro_rules! command_option {
    (
        $name:expr, $description:expr$( => {
            $($addition:ident($($value:expr),*))*
        })?
    ) => {
        command_option! { CommandOptionType::String, $name, $description$( => {$($addition($($value),*))*} )? }
    };
    (
        $command_option_type:expr,
        $name:expr, $description:expr$( => {
            $($addition:ident($($value:expr),*))*
        })?
    ) => {
        CreateCommandOption::new($command_option_type, $name, $description)$($(.$addition($($value),*))*)?
    };
}

macro_rules! commands {
    (
        @raw $command_name:expr => {
            $($addition:ident($($value:expr),*))*
        }
    ) => {
        CreateCommand::new($command_name)$(.$addition($($value),*))*
    };
    (
        @set $guild_id:expr,
        $http_instance:expr,
        $(
            $command_name:expr => {
                $($addition:ident($($value:expr),*))*
            }
        )*
    ) => {
        $guild_id.set_commands(
            $http_instance,
            vec![$(commands!(@raw $command_name => {$($addition($($value),*))*})),*]
        ).await?;
    };
    (
        $guild_id:expr,
        $http_instance:expr,
        $(
            $command_name:expr => {
                $($addition:ident($($value:expr),*))*
            }
        )*
    ) => {
        {
        let guild_id = $guild_id;
        let http = $http_instance;
            $(guild_id.create_command(http,
                commands!(@raw $command_name => {$($addition($($value),*))*})
            ).await?;)*
        }
    };
}

pub async fn setup_default_commands(http: &Http, guild_id: GuildId) -> TicketsResult<()> {
    log::info!("Setting up default commands for guild {}", guild_id);
    commands! {
        @set
        guild_id,
        http,
        // todo should fail if an app with this purpose already exists for this guild
        //  if the app name exists, it should check for existence of a used purpose
        //  if there is no purpose, but there is an app, it should create a new one
        CommandType::Bootstrap => {
            description("Bootstrap the discord channels and roles.")
            add_option(
                command_option! {
                    "app_name", "The name of the application (must be unique)." => {
                        required(true)
                    }
                }
            )
            add_option(
                command_option! {
                    "purpose", "The purpose to bind the guild to." => {
                        add_string_choice(
                            "Management Discord",
                            GuildPurpose::Management.to_string()
                        )
                        add_string_choice(
                            "Consumer Discord",
                            GuildPurpose::Consumer.to_string()
                        )
                        required(true)
                    }
                }
            )
            default_member_permissions(Permissions::ADMINISTRATOR)
        }
        CommandType::Dispose => {
            description("Ensure the discord is set up correctly and up to date.")
            default_member_permissions(Permissions::ADMINISTRATOR)
        }
    }
    Ok(())
}

#[allow(unused_variables)]
pub async fn setup_application_commands(http: &Http, guild_id: GuildId) -> TicketsResult<()> {
    log::info!("Setting up application commands for guild {}", guild_id);
    commands! {
        guild_id,
        http,
    }
    Ok(())
}

pub async fn setup_management_commands(http: &Http, guild_id: GuildId) -> TicketsResult<()> {
    log::info!("Setting up management commands for guild {}", guild_id);
    commands! {
        guild_id,
        http,
        CommandType::PromoteStaff => {
            description("Promote a new staff member.")
            add_option(
                command_option! {
                    CommandOptionType::User,
                    "user", "The user to promote." => {
                        required(true)
                    }
                }
            )
            default_member_permissions(Permissions::ADMINISTRATOR)
            add_option(
                command_option! {
                    "role", "The role to promote the user to." => {
                        add_string_choice(
                            "Staff",
                            UserRole::Staff.to_string()
                        )
                        add_string_choice(
                            "Management",
                            UserRole::Management.to_string()
                        )
                        required(true)
                    }
                }
            )
            default_member_permissions(Permissions::ADMINISTRATOR)
        }
    }

    Ok(())
}
