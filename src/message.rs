use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::proof::InferenceProof;
use crate::template::ChatMessage;

// ---------------------------------------------------------------------------
// Node -> Router messages
// ---------------------------------------------------------------------------

/// Messages sent from a compute node to the router.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeMessage {
    /// Completed inference result for a task.
    TaskResult {
        task_id: Uuid,
        text: String,
        stats: TaskStats,
        proof: Option<InferenceProof>,
    },
    /// We cannot accept the assigned task.
    TaskRejected {
        task_id: Uuid,
        reason: RejectReason,
    },
    /// Periodic or on-demand status snapshot.
    StatusUpdate {
        models: Vec<String>,
        capacity: Capacity,
        version: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        stats: Option<NodeStatsSnapshot>,
    },
    /// Response to a router challenge (placeholder).
    ChallengeResponse {
        challenge: [u8; 32],
        signature: Vec<u8>,
        recovery_id: u8,
    },
}

// ---------------------------------------------------------------------------
// Router -> Node messages
// ---------------------------------------------------------------------------

/// Messages sent from the router to a compute node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RouterMessage {
    /// A new inference task to execute.
    TaskAssignment {
        task_id: Uuid,
        model: String,
        messages: Vec<ChatMessage>,
        max_tokens: u32,
        temperature: f32,
        validation: Option<ValidationRequest>,
    },
    /// Challenge for proof-of-liveness.
    Challenge { challenge: [u8; 32] },
    /// Heartbeat / keep-alive ping.
    Ping,
    /// Updated model registry from the router.
    ModelRegistryUpdate {
        entries: Vec<ModelRegistryEntry>,
    },
}

// ---------------------------------------------------------------------------
// Supporting types
// ---------------------------------------------------------------------------

/// Statistics about a completed inference task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStats {
    pub tokens_generated: u32,
    pub prompt_tokens: u32,
    pub generation_time_ms: u64,
    pub tokens_per_second: f64,
}

/// Reason a task was rejected.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RejectReason {
    /// Model not loaded on this node.
    ModelNotLoaded,
    /// All inference slots are busy.
    AtCapacity,
    /// Task parameters are invalid.
    InvalidRequest(String),
}

/// Current capacity snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capacity {
    /// Number of free inference slots.
    pub free: usize,
    /// Maximum concurrent inference slots.
    pub max: usize,
}

/// Optional validation parameters included with a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRequest {
    /// Token positions at which to extract logprobs.
    pub logprob_positions: Vec<usize>,
    /// Top-k alternatives to collect at each logprob position.
    pub logprob_top_k: usize,
}

/// Snapshot of node-level stats sent with status updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatsSnapshot {
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub tasks_rejected: u64,
    pub total_tokens_generated: u64,
    pub uptime_secs: u64,
}

/// A model entry from the router's registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRegistryEntry {
    pub name: String,
    pub hf_repo: String,
    pub hf_file: String,
    pub chat_template: Option<String>,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_message_roundtrip() {
        let msg = NodeMessage::TaskResult {
            task_id: Uuid::nil(),
            text: "Hello world".into(),
            stats: TaskStats {
                tokens_generated: 10,
                prompt_tokens: 5,
                generation_time_ms: 100,
                tokens_per_second: 100.0,
            },
            proof: None,
        };
        let packed = rmp_serde::to_vec(&msg).unwrap();
        let roundtrip: NodeMessage = rmp_serde::from_slice(&packed).unwrap();
        match roundtrip {
            NodeMessage::TaskResult { task_id, text, .. } => {
                assert_eq!(task_id, Uuid::nil());
                assert_eq!(text, "Hello world");
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_router_message_roundtrip() {
        let msg = RouterMessage::TaskAssignment {
            task_id: Uuid::nil(),
            model: "gemma3:4b".into(),
            messages: vec![ChatMessage {
                role: "user".into(),
                content: "hello".into(),
            }],
            max_tokens: 512,
            temperature: 0.7,
            validation: None,
        };
        let packed = rmp_serde::to_vec(&msg).unwrap();
        let roundtrip: RouterMessage = rmp_serde::from_slice(&packed).unwrap();
        match roundtrip {
            RouterMessage::TaskAssignment { model, .. } => {
                assert_eq!(model, "gemma3:4b");
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_reject_reason_roundtrip() {
        let msg = NodeMessage::TaskRejected {
            task_id: Uuid::nil(),
            reason: RejectReason::AtCapacity,
        };
        let packed = rmp_serde::to_vec(&msg).unwrap();
        let roundtrip: NodeMessage = rmp_serde::from_slice(&packed).unwrap();
        match roundtrip {
            NodeMessage::TaskRejected { reason, .. } => {
                matches!(reason, RejectReason::AtCapacity);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_status_update_roundtrip() {
        let msg = NodeMessage::StatusUpdate {
            models: vec!["gemma3:4b".into()],
            capacity: Capacity { free: 2, max: 4 },
            version: "2.0.0".into(),
            stats: None,
        };
        let packed = rmp_serde::to_vec(&msg).unwrap();
        let roundtrip: NodeMessage = rmp_serde::from_slice(&packed).unwrap();
        match roundtrip {
            NodeMessage::StatusUpdate {
                capacity, version, ..
            } => {
                assert_eq!(capacity.free, 2);
                assert_eq!(capacity.max, 4);
                assert_eq!(version, "2.0.0");
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_node_stats_snapshot_roundtrip() {
        let snap = NodeStatsSnapshot {
            tasks_completed: 10,
            tasks_failed: 2,
            tasks_rejected: 1,
            total_tokens_generated: 5000,
            uptime_secs: 3600,
        };
        let packed = rmp_serde::to_vec(&snap).unwrap();
        let roundtrip: NodeStatsSnapshot = rmp_serde::from_slice(&packed).unwrap();
        assert_eq!(roundtrip.tasks_completed, 10);
        assert_eq!(roundtrip.tasks_failed, 2);
        assert_eq!(roundtrip.tasks_rejected, 1);
        assert_eq!(roundtrip.total_tokens_generated, 5000);
        assert_eq!(roundtrip.uptime_secs, 3600);
    }

    #[test]
    fn test_status_update_with_stats_roundtrip() {
        let msg = NodeMessage::StatusUpdate {
            models: vec!["gemma3:4b".into()],
            capacity: Capacity { free: 1, max: 2 },
            version: "2.0.0".into(),
            stats: Some(NodeStatsSnapshot {
                tasks_completed: 5,
                tasks_failed: 1,
                tasks_rejected: 0,
                total_tokens_generated: 2500,
                uptime_secs: 120,
            }),
        };
        let packed = rmp_serde::to_vec(&msg).unwrap();
        let roundtrip: NodeMessage = rmp_serde::from_slice(&packed).unwrap();
        match roundtrip {
            NodeMessage::StatusUpdate { stats, .. } => {
                let s = stats.unwrap();
                assert_eq!(s.tasks_completed, 5);
                assert_eq!(s.total_tokens_generated, 2500);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_challenge_roundtrip() {
        let msg = RouterMessage::Challenge {
            challenge: [0xAB; 32],
        };
        let packed = rmp_serde::to_vec(&msg).unwrap();
        let roundtrip: RouterMessage = rmp_serde::from_slice(&packed).unwrap();
        match roundtrip {
            RouterMessage::Challenge { challenge } => {
                assert_eq!(challenge, [0xAB; 32]);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_ping_roundtrip() {
        let packed = rmp_serde::to_vec(&RouterMessage::Ping).unwrap();
        let roundtrip: RouterMessage = rmp_serde::from_slice(&packed).unwrap();
        assert!(matches!(roundtrip, RouterMessage::Ping));
    }

    #[test]
    fn test_model_registry_update_roundtrip() {
        let msg = RouterMessage::ModelRegistryUpdate {
            entries: vec![ModelRegistryEntry {
                name: "test:1b".into(),
                hf_repo: "repo/model".into(),
                hf_file: "model.gguf".into(),
                chat_template: Some("chatml".into()),
            }],
        };
        let packed = rmp_serde::to_vec(&msg).unwrap();
        let roundtrip: RouterMessage = rmp_serde::from_slice(&packed).unwrap();
        match roundtrip {
            RouterMessage::ModelRegistryUpdate { entries } => {
                assert_eq!(entries.len(), 1);
                assert_eq!(entries[0].name, "test:1b");
            }
            _ => panic!("wrong variant"),
        }
    }
}
