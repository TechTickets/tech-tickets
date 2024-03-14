use tickets_common::requests::staff::{CreateApp, CreateAppBody};
use tickets_common::requests::SdkCallWithBody;

use crate::commands::CommandContext;
use crate::interactions::Interactable;
use crate::response;

pub async fn create_app(mut command: CommandContext<'_>) -> anyhow::Result<()> {
    let staff_handle = command.get_staff_handle(command.user().id).await;

    let app_name_arg = command.pop_command_arg("app_name")?;
    let app_name = app_name_arg
        .as_str()
        .expect("app_name not decoded as string");

    let response = CreateApp::call_with_body(
        &staff_handle,
        CreateAppBody {
            app_name: app_name.to_string(),
        },
    )
    .await?;

    command
        .respond(
            response! {
                message {
                    ephemeral: true,
                    message: format!(
                        "Application created successfully. App ID: `{}`",
                        response.app_id
                    )
                }
            },
            vec![],
        )
        .await?;
    Ok(())
}
