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

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use dataplane_sdk::{core::error::DbError, error::SdkError};
use serde_json::json;
use tracing::error;

pub type SignalingResult<T> = Result<T, SignalingError>;

pub enum SignalingError {
    Generic(anyhow::Error),
    Sdk(SdkError),
}

impl IntoResponse for SignalingError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            SignalingError::Generic(e) => {
                error!("Internal server error: {:#}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_owned(),
                )
            }
            SignalingError::Sdk(SdkError::Repo(DbError::NotFound(err))) => {
                (StatusCode::NOT_FOUND, err.to_string())
            }
            SignalingError::Sdk(SdkError::Repo(DbError::AlreadyExists(err))) => {
                (StatusCode::CONFLICT, err.to_string())
            }
            SignalingError::Sdk(e) => {
                error!("SDK error: {:#}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "SDK error".to_owned())
            }
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

impl From<SdkError> for SignalingError {
    fn from(value: SdkError) -> Self {
        SignalingError::Sdk(value)
    }
}

impl From<anyhow::Error> for SignalingError {
    fn from(value: anyhow::Error) -> Self {
        SignalingError::Generic(value)
    }
}
