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

use sqlx::FromRow;

#[derive(Clone, Debug, FromRow, PartialEq)]
pub struct ControlPlane {
    pub id: String,
    pub url: String,
}

impl From<ControlPlane> for dataplane_sdk::core::model::control_plane::ControlPlane {
    fn from(control_plane: ControlPlane) -> Self {
        Self::builder()
            .id(control_plane.id)
            .url(control_plane.url)
            .build()
    }
}
