use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::message::Capacity;

/// Router sends this challenge after QUIC connection is established.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeMessage {
    pub challenge: [u8; 32],
}

/// Node responds with signed challenge + metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthRequest {
    /// Ethereum address (hex, no 0x prefix).
    pub address: String,
    /// Signature over SHA-256(challenge).
    pub signature: Vec<u8>,
    /// Recovery ID for the signature.
    pub recovery_id: u8,
    /// Models this node can serve.
    pub models: Vec<String>,
    /// Benchmark tokens-per-second per model.
    pub tps: HashMap<String, f64>,
    /// Node software version.
    pub version: String,
    /// Current capacity.
    pub capacity: Capacity,
}

/// Router responds with auth result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub authenticated: bool,
    /// Assigned node ID on success.
    pub node_id: Option<String>,
    /// Error message on failure.
    pub error: Option<String>,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_challenge_message_roundtrip() {
        let msg = ChallengeMessage {
            challenge: [0x42; 32],
        };
        let packed = rmp_serde::to_vec(&msg).unwrap();
        let roundtrip: ChallengeMessage = rmp_serde::from_slice(&packed).unwrap();
        assert_eq!(roundtrip.challenge, [0x42; 32]);
    }

    #[test]
    fn test_auth_request_roundtrip() {
        let mut tps = HashMap::new();
        tps.insert("gemma3:4b".to_string(), 42.5);
        let req = AuthRequest {
            address: "deadbeef".into(),
            signature: vec![1, 2, 3],
            recovery_id: 0,
            models: vec!["gemma3:4b".into()],
            tps,
            version: "2.0.0".into(),
            capacity: Capacity { free: 1, max: 2 },
        };
        let packed = rmp_serde::to_vec(&req).unwrap();
        let roundtrip: AuthRequest = rmp_serde::from_slice(&packed).unwrap();
        assert_eq!(roundtrip.address, "deadbeef");
        assert_eq!(roundtrip.models, vec!["gemma3:4b"]);
        assert!((*roundtrip.tps.get("gemma3:4b").unwrap() - 42.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_auth_response_success_roundtrip() {
        let resp = AuthResponse {
            authenticated: true,
            node_id: Some("node-123".into()),
            error: None,
        };
        let packed = rmp_serde::to_vec(&resp).unwrap();
        let roundtrip: AuthResponse = rmp_serde::from_slice(&packed).unwrap();
        assert!(roundtrip.authenticated);
        assert_eq!(roundtrip.node_id.unwrap(), "node-123");
    }

    #[test]
    fn test_auth_response_failure_roundtrip() {
        let resp = AuthResponse {
            authenticated: false,
            node_id: None,
            error: Some("bad signature".into()),
        };
        let packed = rmp_serde::to_vec(&resp).unwrap();
        let roundtrip: AuthResponse = rmp_serde::from_slice(&packed).unwrap();
        assert!(!roundtrip.authenticated);
        assert_eq!(roundtrip.error.unwrap(), "bad signature");
    }
}
