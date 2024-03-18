use errors::{MiscError, TicketsResult};

use crate::commands::ParsedCommand;

pub async fn run_command(command: ParsedCommand) -> TicketsResult<()> {
    Err(MiscError::Unimplemented.into())
}
