use serde::{Deserialize, Serialize};

use rand::Rng;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct FailureDetails {
    pub session_id: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Failure {
    HandshakeFailedServerFull(FailureDetails),
    HandshakeFailedTooManyConnections(FailureDetails),
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct HandshakeRequest {
    pub key: i32,
}

impl HandshakeRequest {
    pub fn new() -> HandshakeRequest {
        let mut rng = rand::thread_rng();
        HandshakeRequest { key: rng.gen() }
    }
}

impl Default for HandshakeRequest {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandshakeResponse {
    pub port: usize,
    pub control: usize,
    pub verification: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handshake_request_serialization() {
        let req = HandshakeRequest::new();
        let bytes = bincode::serialize(&req).expect("Failed to serialize");
        let deserialized: HandshakeRequest =
            bincode::deserialize(&bytes).expect("Failed to deserialize");
        assert_eq!(req.key, deserialized.key);
    }

    #[test]
    fn test_handshake_response_serialization() {
        let resp = HandshakeResponse {
            port: 40124,
            control: 32105,
            verification: 123456,
        };
        let bytes = bincode::serialize(&resp).expect("Failed to serialize");
        let deserialized: HandshakeResponse =
            bincode::deserialize(&bytes).expect("Failed to deserialize");
        assert_eq!(resp, deserialized);
    }

    #[test]
    fn test_failure_serialization() {
        let failure = Failure::HandshakeFailedServerFull(FailureDetails { session_id: 42 });
        let bytes = bincode::serialize(&failure).expect("Failed to serialize");
        let deserialized: Failure = bincode::deserialize(&bytes).expect("Failed to deserialize");
        match deserialized {
            Failure::HandshakeFailedServerFull(details) => assert_eq!(details.session_id, 42),
            _ => panic!("Expected HandshakeFailedServerFull"),
        }
    }
}
