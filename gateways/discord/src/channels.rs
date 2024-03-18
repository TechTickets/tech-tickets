use std::fmt::{Display, Formatter};

use serenity::all::ChannelId;

use errors::ParsingError;

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum ChannelPurpose {
    // (management discord) manager related
    ManagementCategory,
    ManagementInfo,
    ManagementBotCommands,
    ManagementLogs,
    // (management discord) staff related
    StaffCategory,
    StaffInfo,
    StaffBotCommands,
    StaffLogs,
    // consumer related
    TicketsCategory,
}

const MANAGEMENT_CATEGORY_PURPOSE: &str = "management_category";
const MANAGEMENT_INFO_CHANNEL: &str = "management_info";
const MANAGEMENT_BOT_COMMANDS_CHANNEL: &str = "management_bot_commands";
const MANAGEMENT_LOGS_CHANNEL: &str = "management_logs";
const STAFF_CATEGORY_PURPOSE: &str = "staff_category";
const STAFF_INFO_CHANNEL: &str = "staff_info";
const STAFF_BOT_COMMANDS_CHANNEL: &str = "staff_bot_commands";
const STAFF_LOGS_CHANNEL: &str = "staff_logs";
const TICKETS_CATEGORY_PURPOSE: &str = "tickets_category";

impl Display for ChannelPurpose {
    #[rustfmt::skip] // keep these in-line for consistency
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ChannelPurpose::ManagementCategory => write!(f, "{}", MANAGEMENT_CATEGORY_PURPOSE),
            ChannelPurpose::ManagementInfo => write!(f, "{}", MANAGEMENT_INFO_CHANNEL),
            ChannelPurpose::ManagementBotCommands => write!(f, "{}", MANAGEMENT_BOT_COMMANDS_CHANNEL),
            ChannelPurpose::ManagementLogs => write!(f, "{}", MANAGEMENT_LOGS_CHANNEL),
            ChannelPurpose::StaffCategory => write!(f, "{}", STAFF_CATEGORY_PURPOSE),
            ChannelPurpose::StaffInfo => write!(f, "{}", STAFF_INFO_CHANNEL),
            ChannelPurpose::StaffBotCommands => write!(f, "{}", STAFF_BOT_COMMANDS_CHANNEL),
            ChannelPurpose::StaffLogs => write!(f, "{}", STAFF_LOGS_CHANNEL),
            ChannelPurpose::TicketsCategory => write!(f, "{}", TICKETS_CATEGORY_PURPOSE),
        }
    }
}

impl TryFrom<String> for ChannelPurpose {
    type Error = ParsingError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Ok(match s.as_str() {
            MANAGEMENT_CATEGORY_PURPOSE => ChannelPurpose::ManagementCategory,
            MANAGEMENT_INFO_CHANNEL => ChannelPurpose::ManagementInfo,
            MANAGEMENT_BOT_COMMANDS_CHANNEL => ChannelPurpose::ManagementBotCommands,
            MANAGEMENT_LOGS_CHANNEL => ChannelPurpose::ManagementLogs,
            STAFF_CATEGORY_PURPOSE => ChannelPurpose::StaffCategory,
            STAFF_INFO_CHANNEL => ChannelPurpose::StaffInfo,
            STAFF_BOT_COMMANDS_CHANNEL => ChannelPurpose::StaffBotCommands,
            STAFF_LOGS_CHANNEL => ChannelPurpose::StaffLogs,
            TICKETS_CATEGORY_PURPOSE => ChannelPurpose::TicketsCategory,
            _ => Err(ParsingError::InvalidChannelPurpose(s))?,
        })
    }
}

pub type ChannelCache = crate::cache::GenericIdCache<ChannelId, ChannelPurpose>;
