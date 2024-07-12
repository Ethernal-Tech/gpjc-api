use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct Response {
    pub exit_code: i32,
    pub data: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ClientStartRequest {
    pub compliance_check_id: String,
    pub policy_id: String,
    pub participants: Vec<String>,
    pub to: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerStartRequest {
    pub compliance_check_id: String,
    pub policy_id: String,
}
