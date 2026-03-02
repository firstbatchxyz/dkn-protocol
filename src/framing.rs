use serde::Serialize;

use crate::error::ProtocolError;

/// Maximum allowed message size (16 MB).
pub const MAX_MESSAGE_SIZE: u32 = 16 * 1024 * 1024;

/// Write a length-prefixed MessagePack message to a QUIC send stream.
///
/// Wire format: `[4-byte BE length][msgpack payload]`
pub async fn write_framed<T: Serialize>(
    send: &mut quinn::SendStream,
    msg: &T,
) -> Result<(), ProtocolError> {
    let payload =
        rmp_serde::to_vec(msg).map_err(|e| ProtocolError::Serialize(format!("{e}")))?;
    let len = payload.len() as u32;
    if len > MAX_MESSAGE_SIZE {
        return Err(ProtocolError::Network(format!(
            "message too large: {len} bytes (max {MAX_MESSAGE_SIZE})"
        )));
    }
    send.write_all(&len.to_be_bytes())
        .await
        .map_err(|e| ProtocolError::Network(format!("write length: {e}")))?;
    send.write_all(&payload)
        .await
        .map_err(|e| ProtocolError::Network(format!("write payload: {e}")))?;
    Ok(())
}

/// Read a length-prefixed MessagePack message from a QUIC receive stream.
///
/// Returns `Ok(None)` on clean EOF (stream closed), `Err` on protocol violations.
pub async fn read_framed<T: serde::de::DeserializeOwned>(
    recv: &mut quinn::RecvStream,
) -> Result<Option<T>, ProtocolError> {
    let mut len_buf = [0u8; 4];
    match recv.read_exact(&mut len_buf).await {
        Ok(()) => {}
        Err(quinn::ReadExactError::FinishedEarly(_)) => return Ok(None),
        Err(e) => return Err(ProtocolError::Network(format!("read length: {e}"))),
    }
    let len = u32::from_be_bytes(len_buf);
    if len > MAX_MESSAGE_SIZE {
        return Err(ProtocolError::Network(format!(
            "message too large: {len} bytes (max {MAX_MESSAGE_SIZE})"
        )));
    }
    let mut payload = vec![0u8; len as usize];
    recv.read_exact(&mut payload)
        .await
        .map_err(|e| ProtocolError::Network(format!("read payload: {e}")))?;
    let msg = rmp_serde::from_slice(&payload)
        .map_err(|e| ProtocolError::Deserialize(format!("{e}")))?;
    Ok(Some(msg))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::*;
    use uuid::Uuid;

    #[test]
    fn test_message_size_within_limit() {
        let msg = NodeMessage::TaskResult {
            task_id: Uuid::nil(),
            text: "x".repeat(1000),
            stats: TaskStats {
                tokens_generated: 100,
                prompt_tokens: 50,
                generation_time_ms: 500,
                tokens_per_second: 200.0,
            },
            proof: None,
        };
        let packed = rmp_serde::to_vec(&msg).unwrap();
        assert!((packed.len() as u32) < MAX_MESSAGE_SIZE);
    }
}
