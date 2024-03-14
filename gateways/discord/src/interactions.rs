use serenity::all::{
    ChannelId, CommandInteraction, Context, CreateAttachment, CreateInteractionResponse, GuildId,
    InteractionId, ModalInteraction, User, UserId,
};

use crate::state::DiscordAppState;

#[macro_export]
macro_rules! response {
    (
        message {
            $(ephemeral: $ephemeral:literal,)?
            message: $message:expr
        }
    ) => {
        serenity::builder::CreateInteractionResponse::Message(
            serenity::builder::CreateInteractionResponseMessage::new()
                $(.ephemeral($ephemeral))?
                .content($message),
        )
    };
}

pub struct InteractionContextParts {
    interaction_id: InteractionId,
    member: Option<Box<serenity::model::guild::Member>>,
    guild_id: Option<GuildId>,
    pub user: User,
    pub channel_id: ChannelId,
    token: String,
}

macro_rules! impl_into_parts_rev {
    (for $type:ident { memberMap: |$member_ident:ident| { $member_mapper:expr } }) => {
        impl From<$type> for InteractionContextParts {
            fn from(value: $type) -> Self {
                let $type {
                    id,
                    member,
                    guild_id,
                    user,
                    channel_id,
                    token,
                    ..
                } = value;

                InteractionContextParts {
                    interaction_id: id,
                    member: member.map(|$member_ident| $member_mapper),
                    guild_id,
                    user,
                    channel_id,
                    token,
                }
            }
        }

        impl From<&$type> for InteractionContextParts {
            fn from(value: &$type) -> Self {
                let $type {
                    id,
                    member,
                    guild_id,
                    user,
                    channel_id,
                    token,
                    ..
                } = value;

                InteractionContextParts {
                    interaction_id: *id,
                    guild_id: guild_id.as_ref().copied(),
                    member: member.as_ref().cloned().map(|$member_ident| $member_mapper),
                    user: user.clone(),
                    channel_id: *channel_id,
                    token: token.to_string(),
                }
            }
        }
    };
}

impl_into_parts_rev!(for ModalInteraction {
    memberMap: |member| {
        Box::new(member)
    }
});
impl_into_parts_rev!(for CommandInteraction {
    memberMap: |member| {
        member
    }
});

pub struct InteractionContext<'a> {
    pub app_state: &'a DiscordAppState,
    pub ctx: &'a Context,
    pub interaction_id: InteractionId,
    member: Option<Box<serenity::model::guild::Member>>,
    guild_id: Option<GuildId>,
    pub user: User,
    pub channel_id: ChannelId,
    pub token: String,
}

pub trait Interactable {
    fn http(&self) -> &serenity::http::Http;

    fn state(&self) -> &DiscordAppState;

    fn interaction_id(&self) -> InteractionId;

    fn guild_id(&self) -> Option<GuildId>;

    fn user(&self) -> &User;

    fn channel_id(&self) -> ChannelId;

    fn token(&self) -> &str;

    async fn get_staff_handle(&self, user: UserId) -> crate::cache::users::User;

    async fn respond(
        &self,
        response: CreateInteractionResponse,
        attachments: Vec<CreateAttachment>,
    ) -> anyhow::Result<()>;

    fn require_guild_id(&self) -> anyhow::Result<GuildId>;

    fn require_member(&self) -> anyhow::Result<&serenity::model::guild::Member>;
}

#[macro_export]
macro_rules! impl_interactable {
    (for $type:ident$(::<$($generics:tt),*>)?.$field:ident) => {
        impl$(<$($generics)*>)? $crate::interactions::Interactable for $type$(<$($generics)*>)? {
            fn http(&self) -> &serenity::http::Http {
                self.$field.http()
            }

            fn state(&self) -> &DiscordAppState {
                self.$field.state()
            }

            fn interaction_id(&self) -> serenity::model::id::InteractionId {
                self.$field.interaction_id()
            }

            fn guild_id(&self) -> Option<serenity::model::id::GuildId> {
                self.$field.guild_id()
            }

            fn user(&self) -> &serenity::model::user::User {
                self.$field.user()
            }

            fn channel_id(&self) -> serenity::model::id::ChannelId {
                self.$field.channel_id()
            }

            fn token(&self) -> &str {
                self.$field.token()
            }

            async fn get_staff_handle(&self, user: serenity::model::id::UserId) -> $crate::cache::users::User {
                self.$field.get_staff_handle(user).await
            }

            async fn respond(
                &self,
                response: serenity::builder::CreateInteractionResponse,
                attachments: Vec<serenity::builder::CreateAttachment>,
            ) -> anyhow::Result<()> {
                self.$field.respond(response, attachments).await
            }

            fn require_guild_id(&self) -> anyhow::Result<serenity::model::id::GuildId> {
                self.$field.require_guild_id()
            }

            fn require_member(&self) -> anyhow::Result<&serenity::model::guild::Member> {
                self.$field.require_member()
            }
        }
    };
}

impl<'a> InteractionContext<'a> {
    pub fn new<'b>(
        app_state: &'b DiscordAppState,
        ctx: &'b Context,
        parts: impl Into<InteractionContextParts>,
    ) -> InteractionContext<'b> {
        let InteractionContextParts {
            interaction_id,
            member,
            guild_id,
            user,
            channel_id,
            token,
        } = parts.into();
        InteractionContext {
            app_state,
            ctx,
            interaction_id,
            member,
            guild_id,
            user,
            channel_id,
            token,
        }
    }
}

impl<'a> Interactable for InteractionContext<'a> {
    fn http(&self) -> &serenity::http::Http {
        &self.ctx.http
    }

    fn state(&self) -> &DiscordAppState {
        self.app_state
    }

    fn interaction_id(&self) -> InteractionId {
        self.interaction_id
    }

    fn guild_id(&self) -> Option<GuildId> {
        self.guild_id
    }

    fn user(&self) -> &User {
        &self.user
    }

    fn channel_id(&self) -> ChannelId {
        self.channel_id
    }

    fn token(&self) -> &str {
        &self.token
    }

    async fn get_staff_handle(&self, user: UserId) -> crate::cache::users::User {
        self.app_state
            .shared_state
            .user_cache
            .get_or_insert(user, || {
                crate::cache::users::User::staff(self.app_state, user)
            })
            .await
    }

    async fn respond(
        &self,
        response: CreateInteractionResponse,
        attachments: Vec<CreateAttachment>,
    ) -> anyhow::Result<()> {
        Ok(self
            .ctx
            .http
            .create_interaction_response(self.interaction_id, &self.token, &response, attachments)
            .await?)
    }

    fn require_guild_id(&self) -> anyhow::Result<GuildId> {
        self.guild_id
            .ok_or_else(|| anyhow::anyhow!("Guild ID is required."))
    }

    fn require_member(&self) -> anyhow::Result<&serenity::model::guild::Member> {
        Ok(self
            .member
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Member is required."))?
            .as_ref())
    }
}
