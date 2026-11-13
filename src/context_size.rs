// Lesson 13
//! Informational context sizing, separate from the conservative compaction budget.
use goose_providers::{
    conversation::{
        fix_conversation, merge_consecutive_messages_for_request, message::Message, Conversation,
    },
    declarative::DeclarativeProviderConfig,
    formats::openai::{create_request_with_options, OpenAiFormatOptions},
    images::ImageFormat,
    model::ModelConfig,
};
use rmcp::model::Tool;
use serde_json::{json, Value};

pub(super) struct ContextSize {
    model: ModelConfig,
    options: OpenAiFormatOptions,
}

impl ContextSize {
    pub(super) fn new(model: &ModelConfig, config: &DeclarativeProviderConfig) -> Self {
        // Match the provider's exact selected-model lookup, not the first model.
        let declared_model = config
            .models
            .iter()
            .find(|candidate| candidate.name == model.model_name);
        let thinking_preservation_format = match declared_model {
            Some(declared_model) => declared_model.thinking_preservation_format,
            None => None,
        };
        let supports_vision = model.supports_vision.unwrap_or_default();
        let options = OpenAiFormatOptions {
            preserve_thinking_context: config.preserves_thinking
                || thinking_preservation_format.is_some(),
            supports_vision,
            thinking_preservation_format,
        };
        Self {
            model: model.clone(),
            options,
        }
    }

    // At completed safe boundaries, mirror alpha.11's visible/fixed/merged projection.
    // Work only on clones; never repair the application's stored conversation here.
    fn projection(
        &self,
        prompt: &str,
        tools: &[Tool],
        conversation: &Conversation,
    ) -> anyhow::Result<Value> {
        let mut visible_messages = conversation.agent_visible_messages();
        let completed = visible_messages
            .last()
            .is_some_and(|message| message.role == rmcp::model::Role::Assistant);
        // Request repair strips a trailing assistant. Protect the completed answer
        // with a temporary next-user boundary, then remove it before measuring.
        // No unknown future learner text or placeholder contributes to the estimate.
        if completed {
            visible_messages.push(Message::user().with_text("context sizing boundary"));
        }
        let visible_conversation = Conversation::new_unvalidated(visible_messages);
        let (fixed_conversation, _issues) = fix_conversation(visible_conversation);
        let mut fixed_messages = fixed_conversation.messages().clone();
        if completed {
            fixed_messages.pop();
        }
        let messages = merge_consecutive_messages_for_request(fixed_messages);
        let request = create_request_with_options(
            &self.model,
            prompt,
            &messages,
            tools,
            &ImageFormat::OpenAi,
            true,
            self.options,
        )?;
        let formatted_tools = match request.get("tools") {
            Some(formatted_tools) => formatted_tools.clone(),
            None => json!([]),
        };
        // Only content: omit model/options/max_tokens, application-only metadata,
        // and cumulative usage. Do not print or save this transient projection.
        Ok(json!({
            "messages": request["messages"],
            "tools": formatted_tools,
        }))
    }

    pub(super) fn estimate(
        &self,
        prompt: &str,
        tools: &[Tool],
        conversation: &Conversation,
    ) -> anyhow::Result<usize> {
        let projection = self.projection(prompt, tools, conversation)?;
        let bytes = serde_json::to_vec(&projection)?;
        // UTF-8 JSON bytes / 4, rounded up: not a tokenizer or exact wire size.
        // This heuristic is NOT a guaranteed upper bound or a safety budget.
        let estimate = bytes.len().div_ceil(4);
        Ok(estimate)
    }

    pub(super) fn status(
        &self,
        prompt: &str,
        tools: &[Tool],
        conversation: &Conversation,
    ) -> String {
        let estimate_result = self.estimate(prompt, tools, conversation);
        let estimate = match estimate_result {
            Ok(estimate) => estimate,
            Err(_error) => return "Context: token estimate unavailable".to_string(),
        };
        format!("Context: ~{} tokens (estimate)", estimate)
    }
}

#[cfg(test)]
mod tests;
