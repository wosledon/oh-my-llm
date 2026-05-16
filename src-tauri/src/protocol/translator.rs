use crate::protocol::anthropic_types::{
    AnthropicContent, AnthropicContentBlock, AnthropicMessage, AnthropicMessageContent,
    AnthropicRequest, AnthropicResponse, AnthropicSystem,
};
use crate::protocol::openai_types::{
    ChatCompletionRequest, ChatCompletionResponse, Choice, ToolCall, Usage,
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
        } else if msg.role == "tool" {
            // OpenAI tool result → Anthropic tool_result block
            let tool_use_id = msg.tool_call_id.clone().unwrap_or_default();
            let content = extract_text(&msg.content).unwrap_or_default();
            let block = AnthropicContentBlock::ToolResult {
                tool_use_id,
                content: serde_json::Value::String(content),
            };
            // Anthropic tool_result blocks must be inside a user message
            // Check if previous message is user; if so merge, else push new user msg
            if let Some(last) = anthropic_messages.last_mut() {
                if last.role == "user" {
                    if let AnthropicMessageContent::Blocks(ref mut blocks) = last.content {
                        blocks.push(block);
                    } else if let AnthropicMessageContent::Text(ref t) = last.content {
                        let mut blocks = vec![AnthropicContentBlock::Text { text: t.clone() }];
                        blocks.push(block);
                        last.content = AnthropicMessageContent::Blocks(blocks);
                    }
                    continue;
                }
            }
            anthropic_messages.push(AnthropicMessage {
                role: "user".to_string(),
                content: AnthropicMessageContent::Blocks(vec![block]),
            });
        } else {
            let role = match msg.role.as_str() {
                "assistant" => "assistant",
                _ => "user",
            };
            let mut blocks: Vec<AnthropicContentBlock> = Vec::new();
            // thinking / reasoning_content → thinking block (must come first)
            if let Some(reasoning) = &msg.reasoning_content {
                if !reasoning.is_empty() {
                    blocks.push(AnthropicContentBlock::Thinking {
                        thinking: reasoning.clone(),
                        signature: String::new(),
                    });
                }
            }
            // tool_calls → tool_use blocks
            if let Some(ref calls) = msg.tool_calls {
                for call in calls {
                    let input = serde_json::from_str(&call.function.arguments)
                        .unwrap_or(serde_json::Value::Null);
                    blocks.push(AnthropicContentBlock::ToolUse {
                        id: call.id.clone(),
                        name: call.function.name.clone(),
                        input,
                    });
                }
            }
            if let Some(content) = extract_text(&msg.content) {
                blocks.push(AnthropicContentBlock::Text { text: content });
            }
            if !blocks.is_empty() {
                anthropic_messages.push(AnthropicMessage {
                    role: role.to_string(),
                    content: AnthropicMessageContent::Blocks(blocks),
                });
            }
        }
    }

    let max_tokens = req.max_completion_tokens.or(req.max_tokens).unwrap_or(4096);

    let tools = req.tools.map(|openai_tools| {
        openai_tools
            .into_iter()
            .map(
                |t| crate::protocol::anthropic_types::AnthropicToolDefinition {
                    name: t.function.name,
                    description: t.function.description,
                    input_schema: t.function.parameters,
                },
            )
            .collect()
    });

    let tool_choice = req.tool_choice.and_then(|tc| {
        if let Some(s) = tc.as_str() {
            match s {
                "auto" => Some(serde_json::json!({"type": "auto"})),
                "none" => Some(serde_json::json!({"type": "none"})),
                "required" => Some(serde_json::json!({"type": "any"})),
                _ => Some(serde_json::json!({"type": "auto"})),
            }
        } else if let Ok(obj) =
            serde_json::from_value::<serde_json::Map<String, serde_json::Value>>(tc.clone())
        {
            if obj.get("type").and_then(|v| v.as_str()) == Some("function") {
                let name = obj
                    .get("function")
                    .and_then(|f| f.as_object())
                    .and_then(|f| f.get("name"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("");
                Some(serde_json::json!({"type": "tool", "name": name}))
            } else {
                Some(tc)
            }
        } else {
            Some(tc)
        }
    });

    AnthropicRequest {
        model: req.model,
        max_tokens,
        messages: anthropic_messages,
        system: system_msg.map(AnthropicSystem::Text),
        temperature: req.temperature,
        top_p: req.top_p,
        top_k: None,
        stop_sequences: req.stop,
        stream: req.stream,
        metadata: None,
        tools,
        tool_choice,
    }
}

/// Convert Anthropic response to OpenAI chat completion response
pub fn anthropic_to_openai(resp: AnthropicResponse) -> ChatCompletionResponse {
    let mut content_parts: Vec<String> = Vec::new();
    let mut tool_calls: Vec<ToolCall> = Vec::new();
    let mut reasoning_parts: Vec<String> = Vec::new();

    for block in &resp.content {
        match block {
            AnthropicContent::Text { text } => content_parts.push(text.clone()),
            AnthropicContent::Thinking { thinking, .. } => reasoning_parts.push(thinking.clone()),
            AnthropicContent::ToolUse { id, name, input } => {
                tool_calls.push(ToolCall {
                    id: id.clone(),
                    call_type: "function".to_string(),
                    function: crate::protocol::openai_types::FunctionCall {
                        name: name.clone(),
                        arguments: input.to_string(),
                    },
                });
            }
            _ => {}
        }
    }

    let content = if content_parts.is_empty() {
        None
    } else {
        Some(crate::protocol::openai_types::ChatContent::Text(
            content_parts.join(""),
        ))
    };

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
                content,
                reasoning_content: if reasoning_parts.is_empty() {
                    None
                } else {
                    Some(reasoning_parts.join(""))
                },
                tool_calls: if tool_calls.is_empty() {
                    None
                } else {
                    Some(tool_calls)
                },
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
            content: Some(crate::protocol::openai_types::ChatContent::Text(
                system.text(),
            )),
            reasoning_content: None,
            tool_calls: None,
            tool_call_id: None,
        });
    }

    for msg in req.messages {
        let mut text_parts: Vec<String> = Vec::new();
        let mut thinking_parts: Vec<String> = Vec::new();
        let mut tool_calls: Vec<ToolCall> = Vec::new();
        let mut tool_results: Vec<crate::protocol::openai_types::ChatMessage> = Vec::new();

        if let AnthropicMessageContent::Blocks(ref blocks) = msg.content {
            for block in blocks {
                match block {
                    AnthropicContentBlock::Text { text } => text_parts.push(text.clone()),
                    AnthropicContentBlock::Thinking { thinking, .. } => {
                        thinking_parts.push(thinking.clone())
                    }
                    AnthropicContentBlock::ToolUse { id, name, input } => {
                        tool_calls.push(ToolCall {
                            id: id.clone(),
                            call_type: "function".to_string(),
                            function: crate::protocol::openai_types::FunctionCall {
                                name: name.clone(),
                                arguments: input.to_string(),
                            },
                        });
                    }
                    AnthropicContentBlock::ToolResult {
                        tool_use_id,
                        content,
                    } => {
                        let content_str = match content {
                            serde_json::Value::String(s) => s.clone(),
                            other => other.to_string(),
                        };
                        tool_results.push(crate::protocol::openai_types::ChatMessage {
                            role: "tool".to_string(),
                            name: None,
                            content: Some(crate::protocol::openai_types::ChatContent::Text(
                                content_str,
                            )),
                            reasoning_content: None,
                            tool_calls: None,
                            tool_call_id: Some(tool_use_id.clone()),
                        });
                    }
                    _ => {}
                }
            }
        } else {
            let text = msg.content.text();
            let thinking = msg.content.thinking();
            if !text.is_empty() {
                text_parts.push(text);
            }
            if !thinking.is_empty() {
                thinking_parts.push(thinking);
            }
        }

        // Assistant message with tool_calls
        if msg.role == "assistant"
            && (!text_parts.is_empty() || !thinking_parts.is_empty() || !tool_calls.is_empty())
        {
            messages.push(crate::protocol::openai_types::ChatMessage {
                role: "assistant".to_string(),
                name: None,
                content: if text_parts.is_empty() {
                    None
                } else {
                    Some(crate::protocol::openai_types::ChatContent::Text(
                        text_parts.join(""),
                    ))
                },
                reasoning_content: if thinking_parts.is_empty() {
                    None
                } else {
                    Some(thinking_parts.join(""))
                },
                tool_calls: if tool_calls.is_empty() {
                    None
                } else {
                    Some(tool_calls)
                },
                tool_call_id: None,
            });
        }
        // User message with text and/or tool_results
        else if msg.role == "user" {
            if !text_parts.is_empty() {
                messages.push(crate::protocol::openai_types::ChatMessage {
                    role: "user".to_string(),
                    name: None,
                    content: Some(crate::protocol::openai_types::ChatContent::Text(
                        text_parts.join(""),
                    )),
                    reasoning_content: None,
                    tool_calls: None,
                    tool_call_id: None,
                });
            }
            messages.extend(tool_results);
        }
    }

    let tools = req.tools.map(|anthropic_tools| {
        anthropic_tools
            .into_iter()
            .map(|t| crate::protocol::openai_types::ToolDefinition {
                tool_type: "function".to_string(),
                function: crate::protocol::openai_types::ToolFunction {
                    name: t.name,
                    description: t.description,
                    parameters: t.input_schema,
                },
            })
            .collect()
    });

    let tool_choice = req.tool_choice.and_then(|tc| {
        tc.get("type").and_then(|v| v.as_str()).map(|t| match t {
            "auto" => serde_json::Value::String("auto".to_string()),
            "any" => serde_json::Value::String("required".to_string()),
            "none" => serde_json::Value::String("none".to_string()),
            "tool" => {
                let name = tc.get("name").and_then(|n| n.as_str()).unwrap_or("");
                serde_json::json!({
                    "type": "function",
                    "function": { "name": name }
                })
            }
            _ => serde_json::Value::String("auto".to_string()),
        })
    });

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
        tools,
        tool_choice,
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
            content: Some(crate::protocol::openai_types::ChatContent::Text(
                "".to_string(),
            )),
            reasoning_content: None,
            tool_calls: None,
            tool_call_id: None,
        },
        logprobs: None,
        finish_reason: None,
    });

    let text_content = extract_text(&choice.message.content).unwrap_or_default();
    let reasoning_content = choice.message.reasoning_content.unwrap_or_default();

    let mut content: Vec<crate::protocol::anthropic_types::AnthropicContent> = Vec::new();
    if !reasoning_content.is_empty() {
        content.push(
            crate::protocol::anthropic_types::AnthropicContent::Thinking {
                thinking: reasoning_content,
                signature: String::new(),
            },
        );
    }
    // tool_calls → Anthropic tool_use content blocks
    if let Some(ref calls) = choice.message.tool_calls {
        for call in calls {
            let input =
                serde_json::from_str(&call.function.arguments).unwrap_or(serde_json::Value::Null);
            content.push(
                crate::protocol::anthropic_types::AnthropicContent::ToolUse {
                    id: call.id.clone(),
                    name: call.function.name.clone(),
                    input,
                },
            );
        }
    }
    if !text_content.is_empty() {
        content
            .push(crate::protocol::anthropic_types::AnthropicContent::Text { text: text_content });
    }

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
