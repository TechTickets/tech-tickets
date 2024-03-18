use crate::shared_state::SharedAppState;
use errors::{MiscError, TicketsResult};
use serenity::all::{
    ChannelId, CommandInteraction, Context, GuildId, InteractionId, ModalInteraction, User,
};
use std::sync::Arc;

#[macro_export]
macro_rules! __response_part {
    ($builder:ident => $addition:ident($($value:expr),*)) => {
        $builder = $builder.$addition($($value),*);
    };
}

#[macro_export]
macro_rules! response {
    (
        message {
            $($addition:ident($($value:expr),*))*
        }
    ) => {
        {
            let mut builder = serenity::builder::CreateInteractionResponseMessage::new();

            $($crate::__response_part!(builder => $addition($($value),*));)*

            serenity::builder::CreateInteractionResponse::Message(builder)
        }
    }
}

#[macro_export]
macro_rules! respond {
    (
        $http:expr,
        $interaction_id:expr,
        $token:expr,
        message {
            $($addition:ident($($value:expr),*))*
        }
    ) => {
        $http.create_interaction_response(
            $interaction_id,
            $token,
            &$crate::response! {
                message {
                    $($addition($($value),*))*
                }
            },
            vec![],
        ).await
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
                    member: member.clone().map(|$member_ident| $member_mapper),
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

pub struct InteractionContext {
    pub state: SharedAppState,
    pub context: Context,
    pub interaction_id: InteractionId,
    member: Option<Box<serenity::model::guild::Member>>,
    guild_id: Option<GuildId>,
    pub user: User,
    pub channel_id: ChannelId,
    pub token: String,
}

pub trait Interactable {
    fn http(&self) -> Arc<serenity::http::Http>;

    fn state(&self) -> SharedAppState;

    fn interaction_id(&self) -> InteractionId;

    fn guild_id(&self) -> Option<GuildId>;

    fn member(&self) -> Option<&serenity::model::guild::Member>;

    fn user(&self) -> &User;

    fn channel_id(&self) -> ChannelId;

    fn token(&self) -> &str;

    fn require_guild_id(&self) -> TicketsResult<GuildId>;

    fn require_member(&self) -> TicketsResult<&serenity::model::guild::Member>;
}

#[macro_export]
macro_rules! impl_interactable {
    (for $type:ident$(::<$($generics:tt),*>)?.$field:ident) => {
        impl$(<$($generics)*>)? $crate::interactions::Interactable for $type$(<$($generics)*>)? {
            fn http(&self) -> std::sync::Arc<serenity::http::Http> {
                self.$field.http()
            }

            fn state(&self) -> crate::shared_state::SharedAppState {
                self.$field.state()
            }

            fn interaction_id(&self) -> serenity::model::id::InteractionId {
                self.$field.interaction_id()
            }

            fn guild_id(&self) -> Option<serenity::model::id::GuildId> {
                self.$field.guild_id()
            }

            fn member(&self) -> Option<&serenity::model::guild::Member> {
                self.$field.member()
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

            fn require_guild_id(&self) -> errors::TicketsResult<serenity::model::id::GuildId> {
                self.$field.require_guild_id()
            }

            fn require_member(&self) -> errors::TicketsResult<&serenity::model::guild::Member> {
                self.$field.require_member()
            }
        }
    };
}

impl InteractionContext {
    pub fn new(
        state: SharedAppState,
        context: Context,
        parts: impl Into<InteractionContextParts>,
    ) -> InteractionContext {
        let InteractionContextParts {
            interaction_id,
            member,
            guild_id,
            user,
            channel_id,
            token,
        } = parts.into();
        InteractionContext {
            state,
            context,
            interaction_id,
            member,
            guild_id,
            user,
            channel_id,
            token,
        }
    }
}

impl Interactable for InteractionContext {
    fn http(&self) -> Arc<serenity::http::Http> {
        self.context.http.clone()
    }

    fn state(&self) -> SharedAppState {
        self.state.clone()
    }

    fn interaction_id(&self) -> InteractionId {
        self.interaction_id
    }

    fn guild_id(&self) -> Option<GuildId> {
        self.guild_id
    }

    fn member(&self) -> Option<&serenity::model::guild::Member> {
        self.member.as_ref().map(|member| member.as_ref())
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

    fn require_guild_id(&self) -> TicketsResult<GuildId> {
        self.guild_id.ok_or(MiscError::GuildContextRequired.into())
    }

    fn require_member(&self) -> TicketsResult<&serenity::model::guild::Member> {
        Ok(self
            .member
            .as_ref()
            .ok_or(MiscError::GuildContextRequired)?
            .as_ref())
    }
}
