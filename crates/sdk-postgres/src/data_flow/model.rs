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
use serde_json::Value;
use sqlx::{FromRow, types::Json};

use dataplane_sdk::core::model::data_address::DataAddress;

#[derive(Builder, Clone, Debug, FromRow, PartialEq)]
pub struct DataFlow {
    pub id: String,
    pub participant_context_id: String,
    pub counter_party_id: String,
    pub state: DataFlowState,
    pub transfer_type: String,
    #[sqlx(rename = "type")]
    pub kind: DataFlowType,
    pub agreement_id: String,
    pub dataset_id: String,
    pub dataspace_context: String,
    pub participant_id: String,
    pub callback_address: String,
    pub suspension_reason: Option<String>,
    pub termination_reason: Option<String>,
    #[builder(default)]
    pub metadata: Json<HashMap<String, Value>>,
    #[builder(into)]
    pub data_address: Option<Json<DataAddress>>,
    #[builder(into)]
    pub labels: Json<Vec<String>>,
    #[builder(default)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[builder(default)]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, sqlx::Type, PartialEq)]
#[sqlx(type_name = "data_flow_state", rename_all = "snake_case")]
pub enum DataFlowState {
    Initiating,
    Preparing,
    Prepared,
    Starting,
    Started,
    Suspended,
    Completed,
    Terminated,
}

#[derive(Clone, Debug, sqlx::Type, PartialEq)]
#[sqlx(type_name = "data_flow_type", rename_all = "snake_case")]
pub enum DataFlowType {
    Consumer,
    Provider,
}

impl From<DataFlow> for dataplane_sdk::core::model::data_flow::DataFlow {
    fn from(flow: DataFlow) -> Self {
        Self::builder()
            .id(flow.id)
            .counter_party_id(flow.counter_party_id)
            .maybe_data_address(flow.data_address.map(|json| json.0))
            .participant_context_id(flow.participant_context_id)
            .state(flow.state.into())
            .labels(flow.labels.0)
            .agreement_id(flow.agreement_id)
            .metadata(flow.metadata.0)
            .dataset_id(flow.dataset_id)
            .dataspace_context(flow.dataspace_context)
            .participant_id(flow.participant_id)
            .callback_address(flow.callback_address)
            .transfer_type(flow.transfer_type)
            .kind(flow.kind.into())
            .build()
    }
}

impl From<DataFlowState> for dataplane_sdk::core::model::data_flow::DataFlowState {
    fn from(status: DataFlowState) -> Self {
        match status {
            DataFlowState::Started => Self::Started,
            DataFlowState::Suspended => Self::Suspended,
            DataFlowState::Terminated => Self::Terminated,
            DataFlowState::Initiating => Self::Initiating,
            DataFlowState::Preparing => Self::Preparing,
            DataFlowState::Prepared => Self::Prepared,
            DataFlowState::Starting => Self::Starting,
            DataFlowState::Completed => Self::Completed,
        }
    }
}

impl From<dataplane_sdk::core::model::data_flow::DataFlowState> for DataFlowState {
    fn from(status: dataplane_sdk::core::model::data_flow::DataFlowState) -> Self {
        match status {
            dataplane_sdk::core::model::data_flow::DataFlowState::Started => Self::Started,
            dataplane_sdk::core::model::data_flow::DataFlowState::Suspended => Self::Suspended,
            dataplane_sdk::core::model::data_flow::DataFlowState::Terminated => Self::Terminated,
            dataplane_sdk::core::model::data_flow::DataFlowState::Initiating => Self::Initiating,
            dataplane_sdk::core::model::data_flow::DataFlowState::Preparing => Self::Preparing,
            dataplane_sdk::core::model::data_flow::DataFlowState::Prepared => Self::Prepared,
            dataplane_sdk::core::model::data_flow::DataFlowState::Starting => Self::Starting,
            dataplane_sdk::core::model::data_flow::DataFlowState::Completed => Self::Completed,
        }
    }
}

impl From<DataFlowType> for dataplane_sdk::core::model::data_flow::DataFlowType {
    fn from(kind: DataFlowType) -> Self {
        match kind {
            DataFlowType::Consumer => Self::Consumer,
            DataFlowType::Provider => Self::Provider,
        }
    }
}

impl From<dataplane_sdk::core::model::data_flow::DataFlowType> for DataFlowType {
    fn from(kind: dataplane_sdk::core::model::data_flow::DataFlowType) -> Self {
        match kind {
            dataplane_sdk::core::model::data_flow::DataFlowType::Consumer => Self::Consumer,
            dataplane_sdk::core::model::data_flow::DataFlowType::Provider => Self::Provider,
        }
    }
}
