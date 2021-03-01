use serde::{Serialize, Deserialize};

use rand::Rng;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct FailureDetails {
    pub session_id: i32
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Failure {
    HandshakeFailedServerFull(FailureDetails),
    HandshakeFailedTooManyConnections(FailureDetails)
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct HandshakeRequest {
    pub key: i32
}

impl HandshakeRequest {
    pub fn new() -> HandshakeRequest {
        let mut rng = rand::thread_rng();
        HandshakeRequest {
            key: rng.gen()
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct HandshakeResponse {
    pub port: usize,
    pub control: usize,
    pub verification: i32
}