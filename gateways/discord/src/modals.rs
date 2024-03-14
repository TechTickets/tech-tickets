use serenity::all::{ActionRowComponent, Context, ModalInteraction};
use std::collections::HashMap;

use crate::impl_interactable;
use crate::interactions::InteractionContext;
use crate::state::DiscordAppState;

pub struct ModalContext<'a> {
    pub interaction: InteractionContext<'a>,
    text_inputs: HashMap<String, String>,
}

impl<'a> ModalContext<'a> {
    pub fn new<'b>(
        app_state: &'b DiscordAppState,
        ctx: &'b Context,
        interaction: ModalInteraction,
    ) -> ModalContext<'b> {
        let mut text_inputs = HashMap::new();

        for component in &interaction.data.components {
            for component in &component.components {
                match component {
                    ActionRowComponent::Button(_) => {}
                    ActionRowComponent::SelectMenu(_) => {}
                    ActionRowComponent::InputText(text) => {
                        if let Some(value) = text.value.as_ref() {
                            text_inputs.insert(text.custom_id.to_string(), value.to_string());
                        }
                    }
                    _ => {}
                }
            }
        }

        ModalContext {
            interaction: InteractionContext::new(app_state, ctx, interaction),
            text_inputs,
        }
    }

    pub fn pop_text_input(&mut self, input_name: &str) -> anyhow::Result<String> {
        self.text_inputs
            .remove(input_name)
            .ok_or_else(|| anyhow::format_err!("Missing required input: {}", input_name))
    }
}

impl_interactable!(for ModalContext::<'a>.interaction);
