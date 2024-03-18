use serenity::all::{ActionRowComponent, Context, ModalInteraction};
use std::collections::HashMap;

use crate::impl_interactable;
use crate::interactions::InteractionContext;
use crate::shared_state::SharedAppState;

pub struct ModalContext {
    pub interaction: InteractionContext,
    text_inputs: HashMap<String, String>,
}

impl ModalContext {
    pub fn new(
        app_state: SharedAppState,
        ctx: Context,
        interaction: ModalInteraction,
    ) -> ModalContext {
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

    pub fn pop_text_input(&mut self, input_name: &str) -> Option<String> {
        self.text_inputs.remove(input_name)
    }
}

impl_interactable!(for ModalContext.interaction);
