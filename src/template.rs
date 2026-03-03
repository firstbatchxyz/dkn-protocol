use serde::{Deserialize, Serialize};

/// Helper module to serialize/deserialize `Vec<u8>` as a base64 string.
mod base64_bytes {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(data: &Vec<u8>, ser: S) -> Result<S::Ok, S::Error> {
        use base64::Engine;
        let encoded = base64::engine::general_purpose::STANDARD.encode(data);
        encoded.serialize(ser)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(de: D) -> Result<Vec<u8>, D::Error> {
        use base64::Engine;
        let s = String::deserialize(de)?;
        base64::engine::general_purpose::STANDARD
            .decode(&s)
            .map_err(serde::de::Error::custom)
    }
}

/// A single part of a multimodal message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image {
        #[serde(with = "base64_bytes")]
        data: Vec<u8>,
    },
    #[serde(rename = "audio")]
    Audio {
        #[serde(with = "base64_bytes")]
        data: Vec<u8>,
    },
}

/// Multi-modal message content.
///
/// Serializes as an untagged enum: a plain string for text-only messages,
/// or an array of `ContentPart` for multimodal messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Parts(Vec<ContentPart>),
    Text(String),
}

impl MessageContent {
    /// Whether this content contains an image part.
    pub fn has_image(&self) -> bool {
        match self {
            MessageContent::Text(_) => false,
            MessageContent::Parts(parts) => {
                parts.iter().any(|p| matches!(p, ContentPart::Image { .. }))
            }
        }
    }

    /// Whether this content contains an audio part.
    pub fn has_audio(&self) -> bool {
        match self {
            MessageContent::Text(_) => false,
            MessageContent::Parts(parts) => {
                parts.iter().any(|p| matches!(p, ContentPart::Audio { .. }))
            }
        }
    }

    /// Render the text content, replacing each media part with the given marker string.
    ///
    /// For `Text(s)` returns `s` unchanged.
    /// For `Parts(…)` concatenates text parts and inserts `marker` for each image/audio part.
    pub fn text_with_markers(&self, marker: &str) -> String {
        match self {
            MessageContent::Text(s) => s.clone(),
            MessageContent::Parts(parts) => {
                let mut out = String::new();
                for part in parts {
                    match part {
                        ContentPart::Text { text } => out.push_str(text),
                        ContentPart::Image { .. } | ContentPart::Audio { .. } => {
                            out.push_str(marker);
                        }
                    }
                }
                out
            }
        }
    }

    /// Collect references to all media (image/audio) byte slices in order.
    pub fn media_data(&self) -> Vec<&[u8]> {
        match self {
            MessageContent::Text(_) => vec![],
            MessageContent::Parts(parts) => parts
                .iter()
                .filter_map(|p| match p {
                    ContentPart::Image { data } | ContentPart::Audio { data } => {
                        Some(data.as_slice())
                    }
                    ContentPart::Text { .. } => None,
                })
                .collect(),
        }
    }
}

impl std::fmt::Display for MessageContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageContent::Text(s) => f.write_str(s),
            MessageContent::Parts(parts) => {
                for part in parts {
                    match part {
                        ContentPart::Text { text } => f.write_str(text)?,
                        ContentPart::Image { .. } => f.write_str("<image>")?,
                        ContentPart::Audio { .. } => f.write_str("<audio>")?,
                    }
                }
                Ok(())
            }
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

    #[test]
    fn test_text_content_no_media() {
        let content = MessageContent::Text("hello".into());
        assert!(!content.has_image());
        assert!(!content.has_audio());
        assert!(content.media_data().is_empty());
        assert_eq!(content.text_with_markers("[M]"), "hello");
    }

    #[test]
    fn test_parts_with_image() {
        let content = MessageContent::Parts(vec![
            ContentPart::Text {
                text: "What is this?".into(),
            },
            ContentPart::Image {
                data: vec![1, 2, 3],
            },
        ]);
        assert!(content.has_image());
        assert!(!content.has_audio());
        assert_eq!(content.media_data().len(), 1);
        assert_eq!(content.media_data()[0], &[1, 2, 3]);
        assert_eq!(
            content.text_with_markers("<__media__>"),
            "What is this?<__media__>"
        );
        assert_eq!(content.to_string(), "What is this?<image>");
    }

    #[test]
    fn test_parts_with_audio() {
        let content = MessageContent::Parts(vec![
            ContentPart::Text {
                text: "Transcribe: ".into(),
            },
            ContentPart::Audio {
                data: vec![4, 5, 6],
            },
        ]);
        assert!(!content.has_image());
        assert!(content.has_audio());
        assert_eq!(content.media_data().len(), 1);
        assert_eq!(content.to_string(), "Transcribe: <audio>");
    }

    #[test]
    fn test_content_part_json_roundtrip() {
        let parts = vec![
            ContentPart::Text {
                text: "Look:".into(),
            },
            ContentPart::Image {
                data: vec![0xFF, 0x00, 0xAB],
            },
        ];
        let json = serde_json::to_string(&parts).unwrap();
        let roundtrip: Vec<ContentPart> = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip.len(), 2);
        match &roundtrip[1] {
            ContentPart::Image { data } => assert_eq!(data, &[0xFF, 0x00, 0xAB]),
            _ => panic!("expected Image"),
        }
    }

    #[test]
    fn test_message_content_parts_json_roundtrip() {
        let content = MessageContent::Parts(vec![
            ContentPart::Text {
                text: "Describe".into(),
            },
            ContentPart::Image {
                data: vec![1, 2, 3, 4],
            },
        ]);
        let json = serde_json::to_string(&content).unwrap();
        let roundtrip: MessageContent = serde_json::from_str(&json).unwrap();
        assert!(roundtrip.has_image());
        assert_eq!(roundtrip.media_data().len(), 1);
    }

    #[test]
    fn test_message_content_text_json_roundtrip() {
        // Plain string should still round-trip as Text variant
        let content = MessageContent::Text("just text".into());
        let json = serde_json::to_string(&content).unwrap();
        let roundtrip: MessageContent = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip.to_string(), "just text");
        assert!(!roundtrip.has_image());
    }

    #[test]
    fn test_message_content_parts_msgpack_roundtrip() {
        let content = MessageContent::Parts(vec![
            ContentPart::Text {
                text: "hi".into(),
            },
            ContentPart::Image {
                data: vec![10, 20],
            },
        ]);
        let packed = rmp_serde::to_vec(&content).unwrap();
        let roundtrip: MessageContent = rmp_serde::from_slice(&packed).unwrap();
        assert!(roundtrip.has_image());
    }

    #[test]
    fn test_chat_message_with_parts_roundtrip() {
        let msg = ChatMessage {
            role: "user".into(),
            content: MessageContent::Parts(vec![
                ContentPart::Text {
                    text: "What is this image?".into(),
                },
                ContentPart::Image {
                    data: vec![0xDE, 0xAD],
                },
            ]),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let roundtrip: ChatMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip.role, "user");
        assert!(roundtrip.content.has_image());
    }
}
