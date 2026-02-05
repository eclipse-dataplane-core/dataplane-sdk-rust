use std::collections::HashMap;

use bon::Builder;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::data_address::DataAddress;

#[derive(Builder, Clone, Debug, PartialEq)]
#[builder(on(String, into))]
pub struct DataFlow {
    pub id: String,
    #[builder(default = DataFlowState::Initiating)]
    pub state: DataFlowState,
    pub transfer_type: String,
    pub process_id: String,
    pub agreement_id: String,
    pub dataset_id: String,
    pub dataspace_context: String,
    pub participant_id: String,
    pub counter_party_id: String,
    pub participant_context_id: String,
    pub suspension_reason: Option<String>,
    pub termination_reason: Option<String>,
    pub labels: Vec<String>,
    pub metadata: HashMap<String, Value>,
    #[builder(into)]
    pub data_address: Option<DataAddress>,
    #[builder(default)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[builder(default)]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DataFlowState {
    Initiating,
    Peparing,
    Prepared,
    Starting,
    Started,
    Suspended,
    Completed,
    Terminated,
}
