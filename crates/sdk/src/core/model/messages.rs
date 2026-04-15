//  Copyright (c) 2026 Metaform Systems, Inc
//
//  This program and the accompanying materials are made available under the
//  terms of the Apache License, Version 2.0 which is available at
//  https://www.apache.org/licenses/LICENSE-2.0
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Contributors:
//         Metaform Systems, Inc. - initial API and implementation
//

use std::collections::HashMap;

use bon::Builder;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{data_address::DataAddress, data_flow::DataFlowState};

#[derive(Debug, Serialize, Deserialize, Clone, Builder)]
#[serde(rename_all = "camelCase")]
#[builder(on(String, into))]
pub struct DataFlowStartMessage {
    pub message_id: String,
    pub participant_id: String,
    pub counter_party_id: String,
    pub dataspace_context: String,
    pub process_id: String,
    pub agreement_id: String,
    pub dataset_id: String,
    pub callback_address: String,
    pub transfer_type: String,
    pub data_address: Option<DataAddress>,
    #[builder(default)]
    pub labels: Vec<String>,
    #[builder(default)]
    pub metadata: HashMap<String, Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Builder)]
#[serde(rename_all = "camelCase")]
#[builder(on(String, into))]
pub struct DataFlowPrepareMessage {
    pub message_id: String,
    pub participant_id: String,
    pub counter_party_id: String,
    pub dataspace_context: String,
    pub process_id: String,
    pub agreement_id: String,
    pub dataset_id: String,
    pub callback_address: String,
    pub transfer_type: String,
    #[builder(default)]
    pub labels: Vec<String>,
    #[builder(default)]
    pub metadata: HashMap<String, Value>,
}

#[derive(Debug, Builder, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[builder(on(String, into))]
pub struct DataFlowStatusMessage {
    pub data_address: Option<DataAddress>,
    pub state: DataFlowState,
    pub error: Option<String>,
}

#[derive(Debug, Builder, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DataFlowStartedNotificationMessage {
    pub data_address: Option<DataAddress>,
}

#[derive(Builder, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[builder(on(String, into))]
pub struct DataFlowStatusResponseMessage {
    pub data_flow_id: String,
    pub state: DataFlowState,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DataFlowSuspendMessage {
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Builder)]
#[serde(rename_all = "camelCase")]
#[builder(on(String, into))]
pub struct DataFlowTerminateMessage {
    pub reason: Option<String>,
}
