use serde::{Deserialize, Serialize};

/// Multi-modal message content.
///
/// Currently only text is supported; image/audio variants will be added later.
/// Serializes as an untagged enum so plain strings round-trip transparently.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
}

impl MessageContent {
    /// Whether this content contains an image part.
    pub fn has_image(&self) -> bool {
        false
    }

    /// Whether this content contains an audio part.
    pub fn has_audio(&self) -> bool {
        false
    }
}

impl std::fmt::Display for MessageContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageContent::Text(s) => f.write_str(s),
        }
    }
}

impl From<String> for MessageContent {
    fn from(s: String) -> Self {
        MessageContent::Text(s)
    }
}

impl From<&str> for MessageContent {
    fn from(s: &str) -> Self {
        MessageContent::Text(s.to_string())
    }
}

/// A single message in a chat conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: MessageContent,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_message_serde() {
        let msg = ChatMessage {
            role: "user".into(),
            content: "hello".into(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let roundtrip: ChatMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip.role, "user");
        assert_eq!(roundtrip.content.to_string(), "hello");
    }
}
