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
use thiserror::Error;

use super::data_address::DataAddress;

#[derive(Builder, Clone, Debug, PartialEq)]
#[builder(on(String, into))]
pub struct DataFlow {
    pub id: String,
    #[builder(default = DataFlowState::Initiating)]
    pub state: DataFlowState,
    pub transfer_type: String,
    pub kind: DataFlowType,
    pub agreement_id: String,
    pub dataset_id: String,
    pub dataspace_context: String,
    pub participant_id: String,
    pub counter_party_id: String,
    pub callback_address: String,
    pub participant_context_id: String,
    pub suspension_reason: Option<String>,
    pub termination_reason: Option<String>,
    #[builder(default)]
    pub labels: Vec<String>,
    #[builder(default)]
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
pub enum DataFlowType {
    Consumer,
    Provider,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
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

#[derive(Debug, Error)]
pub enum TransitionError {
    #[error("Invalid state transition: {0}")]
    InvalidTransition(String),
}

impl DataFlow {
    pub fn transition_to_preparing(&mut self) -> Result<(), TransitionError> {
        match self.state {
            DataFlowState::Initiating => {
                self.state = DataFlowState::Preparing;
                self.updated_at = chrono::Utc::now();
                Ok(())
            }
            DataFlowState::Preparing => Ok(()), // already in preparing, idempotent
            _ => Err(TransitionError::InvalidTransition(format!(
                "Invalid state for dataflow {} transition from {:?} to PREPARING",
                self.id, self.state
            ))),
        }
    }

    pub fn transition_to_prepared(&mut self) -> Result<(), TransitionError> {
        match self.state {
            DataFlowState::Initiating | DataFlowState::Preparing => {
                self.state = DataFlowState::Prepared;
                self.updated_at = chrono::Utc::now();
                Ok(())
            }
            DataFlowState::Prepared => Ok(()), // already in prepared, idempotent
            _ => Err(TransitionError::InvalidTransition(format!(
                "Invalid state for dataflow {} transition from {:?} to PREPARED",
                self.id, self.state
            ))),
        }
    }

    pub fn transition_to_starting(&mut self) -> Result<(), TransitionError> {
        match self.state {
            DataFlowState::Prepared => {
                self.state = DataFlowState::Starting;
                self.updated_at = chrono::Utc::now();
                Ok(())
            }
            DataFlowState::Starting => Ok(()), // already in starting, idempotent
            _ => Err(TransitionError::InvalidTransition(format!(
                "Invalid state for dataflow {} transition from {:?} to STARTING",
                self.id, self.state
            ))),
        }
    }

    pub fn transition_to_started(&mut self) -> Result<(), TransitionError> {
        match self.state {
            DataFlowState::Starting | DataFlowState::Prepared | DataFlowState::Suspended => {
                self.state = DataFlowState::Started;
                self.updated_at = chrono::Utc::now();
                Ok(())
            }
            DataFlowState::Started => Ok(()), // already in started, idempotent
            _ => Err(TransitionError::InvalidTransition(format!(
                "Invalid state for dataflow {} transition from {:?} to STARTED",
                self.id, self.state
            ))),
        }
    }

    pub fn transition_to_suspended(
        &mut self,
        reason: Option<String>,
    ) -> Result<(), TransitionError> {
        match self.state {
            DataFlowState::Started => {
                self.state = DataFlowState::Suspended;
                self.suspension_reason = reason;
                self.updated_at = chrono::Utc::now();
                Ok(())
            }
            DataFlowState::Suspended => Ok(()), // already in suspended, idempotent
            _ => Err(TransitionError::InvalidTransition(format!(
                "Invalid state for dataflow {} transition from {:?} to SUSPENDED",
                self.id, self.state
            ))),
        }
    }

    pub fn transition_to_completed(&mut self) -> Result<(), TransitionError> {
        match self.state {
            DataFlowState::Started => {
                self.state = DataFlowState::Completed;
                self.updated_at = chrono::Utc::now();
                Ok(())
            }
            DataFlowState::Completed => Ok(()), // already in completed, idempotent
            _ => Err(TransitionError::InvalidTransition(format!(
                "Invalid state for dataflow {} transition from {:?} to COMPLETED",
                self.id, self.state
            ))),
        }
    }

    pub fn transition_to_terminated(
        &mut self,
        reason: Option<String>,
    ) -> Result<(), TransitionError> {
        match self.state {
            DataFlowState::Completed => Err(TransitionError::InvalidTransition(format!(
                "Invalid state for dataflow {} transition from {:?} to TERMINATED",
                self.id, self.state
            ))),
            DataFlowState::Terminated => Ok(()), // already in terminated, idempotent
            _ => {
                self.state = DataFlowState::Terminated;
                self.termination_reason = reason;
                self.updated_at = chrono::Utc::now();
                Ok(())
            }
        }
    }
}
