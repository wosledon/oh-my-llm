use crate::protocol::anthropic_types::{AnthropicMessage, AnthropicRequest, AnthropicResponse};
use crate::protocol::openai_types::{
    ChatCompletionRequest, ChatCompletionResponse, Choice, Usage,
};

/// Convert OpenAI chat completion request to Anthropic messages request
pub fn openai_to_anthropic(req: ChatCompletionRequest) -> AnthropicRequest {
    let mut system_msg: Option<String> = None;
    let mut anthropic_messages: Vec<AnthropicMessage> = Vec::new();

    for msg in req.messages {
        if msg.role == "system" {
            if let Some(content) = extract_text(&msg.content) {
                system_msg = Some(content);
            }
        } else {
            let role = match msg.role.as_str() {
                "assistant" => "assistant",
                _ => "user",
            };
            if let Some(content) = extract_text(&msg.content) {
                anthropic_messages.push(AnthropicMessage {
                    role: role.to_string(),
                    content,
                });
            }
        }
    }

    let max_tokens = req.max_completion_tokens.or(req.max_tokens).unwrap_or(4096);

    AnthropicRequest {
        model: req.model,
        max_tokens,
        messages: anthropic_messages,
        system: system_msg,
        temperature: req.temperature,
        top_p: req.top_p,
        top_k: None,
        stop_sequences: req.stop,
        stream: req.stream,
        metadata: None,
    }
}

/// Convert Anthropic response to OpenAI chat completion response
pub fn anthropic_to_openai(resp: AnthropicResponse) -> ChatCompletionResponse {
    let content = resp
        .content
        .iter()
        .filter_map(|c| c.text())
        .collect::<Vec<_>>()
        .join("");

    ChatCompletionResponse {
        id: resp.id,
        object: "chat.completion".to_string(),
        created: chrono::Utc::now().timestamp(),
        model: resp.model,
        system_fingerprint: None,
        choices: vec![Choice {
            index: 0,
            message: crate::protocol::openai_types::ChatMessage {
                role: "assistant".to_string(),
                name: None,
                content: Some(crate::protocol::openai_types::ChatContent::Text(content)),
                tool_calls: None,
                tool_call_id: None,
            },
            logprobs: None,
            finish_reason: resp.stop_reason,
        }],
        usage: Some(Usage {
            prompt_tokens: resp.usage.input_tokens,
            completion_tokens: resp.usage.output_tokens,
            total_tokens: resp.usage.input_tokens + resp.usage.output_tokens,
        }),
    }
}

/// Convert Anthropic messages request to OpenAI chat completion request
pub fn anthropic_to_openai_request(req: AnthropicRequest) -> ChatCompletionRequest {
    let mut messages: Vec<crate::protocol::openai_types::ChatMessage> = Vec::new();

    if let Some(system) = req.system {
        messages.push(crate::protocol::openai_types::ChatMessage {
            role: "system".to_string(),
            name: None,
            content: Some(crate::protocol::openai_types::ChatContent::Text(system)),
            tool_calls: None,
            tool_call_id: None,
        });
    }

    for msg in req.messages {
        messages.push(crate::protocol::openai_types::ChatMessage {
            role: msg.role,
            name: None,
            content: Some(crate::protocol::openai_types::ChatContent::Text(msg.content)),
            tool_calls: None,
            tool_call_id: None,
        });
    }

    ChatCompletionRequest {
        model: req.model,
        messages,
        temperature: req.temperature,
        top_p: req.top_p,
        n: None,
        stream: req.stream,
        stop: req.stop_sequences,
        max_tokens: Some(req.max_tokens),
        max_completion_tokens: None,
        presence_penalty: None,
        frequency_penalty: None,
        logit_bias: None,
        user: None,
        tools: None,
        tool_choice: None,
        response_format: None,
    }
}

/// Convert OpenAI chat completion response to Anthropic response
pub fn openai_to_anthropic_response(resp: ChatCompletionResponse) -> AnthropicResponse {
    let choice = resp.choices.into_iter().next().unwrap_or(Choice {
        index: 0,
        message: crate::protocol::openai_types::ChatMessage {
            role: "assistant".to_string(),
            name: None,
            content: Some(crate::protocol::openai_types::ChatContent::Text("".to_string())),
            tool_calls: None,
            tool_call_id: None,
        },
        logprobs: None,
        finish_reason: None,
    });

    let text_content = extract_text(&choice.message.content).unwrap_or_default();

    let content = if text_content.is_empty() {
        vec![]
    } else {
        vec![crate::protocol::anthropic_types::AnthropicContent::Text {
            text: text_content,
        }]
    };

    let usage = resp.usage.unwrap_or(Usage {
        prompt_tokens: 0,
        completion_tokens: 0,
        total_tokens: 0,
    });

    AnthropicResponse {
        id: resp.id,
        response_type: "message".to_string(),
        role: "assistant".to_string(),
        content,
        model: resp.model,
        stop_reason: choice.finish_reason,
        stop_sequence: None,
        usage: crate::protocol::anthropic_types::AnthropicUsage {
            input_tokens: usage.prompt_tokens,
            output_tokens: usage.completion_tokens,
        },
    }
}

fn extract_text(content: &Option<crate::protocol::openai_types::ChatContent>) -> Option<String> {
    match content {
        Some(crate::protocol::openai_types::ChatContent::Text(t)) => Some(t.clone()),
        Some(crate::protocol::openai_types::ChatContent::Parts(parts)) => {
            let texts: Vec<String> = parts
                .iter()
                .filter_map(|p| match p {
                    crate::protocol::openai_types::ContentPart::Text { text } => Some(text.clone()),
                    _ => None,
                })
                .collect();
            if texts.is_empty() {
                None
            } else {
                Some(texts.join(""))
            }
        }
        None => None,
    }
}
