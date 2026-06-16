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

use crate::core::{
    error::{DbError, HandlerError},
    model::data_flow::TransitionError,
};

pub type SdkResult<T> = Result<T, SdkError>;

#[derive(Debug, thiserror::Error)]
pub enum SdkError {
    #[error("Handler error: {0}")]
    Handler(#[from] HandlerError),
    #[error("Repository error: {0}")]
    Repo(#[from] DbError),
    #[error("Transition error: {0}")]
    Transition(#[from] TransitionError),
    #[error("Notification transport error: {0}")]
    Notification(#[from] reqwest::Error),
    #[error("Notification rejected with status {status}: {body}")]
    NotificationStatus { status: u16, body: String },
}
