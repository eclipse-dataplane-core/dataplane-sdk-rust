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

use bon::Builder;

/// A control plane that a data flow reports back to. The `url` is the base
/// address used to build control-plane notification callbacks. Data flows
/// reference a control plane by its `id` (see `DataFlow::control_plane_id`).
#[derive(Debug, Clone, PartialEq, Builder)]
#[builder(on(String, into))]
pub struct ControlPlane {
    pub id: String,
    pub url: String,
}
