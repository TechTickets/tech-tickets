use serenity::all::{Context, EventHandler, GuildId, Interaction, ModalInteraction, Ready};
use tickets_common::requests::staff::{GetBulkGuildData, GetBulkGuildDataBody};
use tickets_common::requests::SdkCallWithBody;

use crate::commands::{bootstrap_callback, CommandContext, CommandType};
use crate::modals::ModalContext;
use crate::response;
use crate::state::{DiscordAppState, WebsocketAppState};

#[serenity::async_trait]
impl EventHandler for DiscordAppState {
    async fn ready(&self, ctx: Context, ready: Ready) {
        log::info!("Populating Guild Cache");

        self.shared_state
            .guild_cache
            .populate(
                GetBulkGuildData::call_with_body(
                    &self.ticket_system_client,
                    GetBulkGuildDataBody {
                        guild_ids: ready.guilds.iter().map(|g| g.id.get()).collect(),
                    },
                )
                .await
                .expect("Failed to get guild data")
                .guild_data
                .into_iter()
                .map(|gd| (GuildId::from(gd.guild_id), gd.guild_purpose, gd.app_id)),
            )
            .await;

        log::info!("Setting up Commands");

        for guild in &ready.guilds {
            crate::commands::ready(&ctx, guild.id)
                .await
                .expect("Failed to setup commands");
        }

        log::info!("Initializing Channel Cache");

        self.shared_state
            .channel_cache
            .init_channel_cache(&self.postgres_client, ready.guilds.iter().map(|g| g.id))
            .await
            .expect("Failed to init channel cache");

        log::info!("Initializing Role Cache");

        self.shared_state
            .roles_cache
            .init_roles_cache(&ctx, ready.guilds.iter().map(|g| g.id))
            .await
            .expect("Failed to init role cache");

        log::info!("Discord Bot Ready! Initializing Websocket Real-Time Updates.");
        // todo start websocket thread
        let state = WebsocketAppState::create(&ctx.http, &self.shared_state);
        tokio::spawn(async move {
            // todo initialize and run websocket against redis database
        });
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        #[allow(clippy::single_match)]
        match interaction {
            Interaction::Command(interaction) => {
                let command_type = CommandType::from(interaction.data.name.clone());
                let command_context = CommandContext::new(self, &ctx, interaction);

                command_type
                    .handle_command_interaction(command_context)
                    .await;
            }
            Interaction::Modal(interaction) => {
                let ModalInteraction { id, token, .. } = &interaction;
                let id = *id;
                let token = token.to_string();

                if let Err(err) = match interaction.data.custom_id.as_str() {
                    crate::commands::BOOTSTRAP_CALLBACK_ID => {
                        bootstrap_callback(ModalContext::new(self, &ctx, interaction)).await
                    }
                    _ => Ok(()),
                } {
                    if let Err(err) = ctx
                        .http
                        .create_interaction_response(
                            id,
                            &token,
                            &response! {
                                message {
                                    ephemeral: true,
                                    message: format!("Error in modal interaction. {}", err)
                                }
                            },
                            vec![],
                        )
                        .await
                    {
                        log::error!("Failed to send error response: {}", err);
                    }
                }
            }
            _ => {}
        }
    }
}

// async fn publish_mgmt_info(
//     ctx: &Context,
//     channel: ChannelId,
//     commands_channel: ChannelId,
// ) -> anyhow::Result<()> {
//     channel
//         .send_message(
//             &ctx.http,
//             CreateMessage::new()
//                 .embed(
//                     CreateEmbed::new()
//                         .title("Management Info Channel")
//                         .description("Welcome to the management info channel!\n\
//                         Here you'll find important information about managing the Tech Tickets bot.")
//                         .image("https://t4.ftcdn.net/jpg/05/71/83/47/360_F_571834789_ujYbUnH190iUokdDhZq7GXeTBRgqYVwa.jpg")
//                         .color(Color::TEAL)
//                         .author(
//                             CreateEmbedAuthor::new("[Tech Tickets]")
//                                 .url("https://tech-tickets.help/")
//                                 // placeholder icon url
//                                 .icon_url("https://t3.ftcdn.net/jpg/05/16/27/58/360_F_516275801_f3Fsp17x6HQK0xQgDQEELoTuERO4SsWV.jpg")
//                         )
//                         .field(
//                             "Command Information",
//                             format!("Information on commands can be found in the slash-commands descriptions.\n\
//                                    Use <#{}> for executing bot commands.", commands_channel),
//                             false,
//                         ),
//                 ),
//         )
//         .await?;
//     Ok(())
// }
