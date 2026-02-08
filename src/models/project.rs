use serde::{Deserialize, Serialize};
use super::protocol::Protocol;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BitLoomProject {
    pub project_version: u32,
    pub protocols: Vec<Protocol>,
}