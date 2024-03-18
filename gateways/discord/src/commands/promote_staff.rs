use crate::commands::ParsedCommand;
use errors::{MiscError, TicketsResult};

pub async fn run_command(command: ParsedCommand) -> TicketsResult<()> {
    Err(MiscError::Unimplemented.into())
}
