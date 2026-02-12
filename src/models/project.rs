use super::protocol::Protocol;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BitLoomProject {
    pub project_version: u32,
    pub protocols: Vec<Protocol>,
}
