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
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Builder, PartialEq)]
#[serde(rename_all = "camelCase")]
#[builder(on(String, into))]
pub struct DataAddress {
    #[builder(default = "DataAddress".to_string())]
    #[serde(rename = "@type")]
    pub kind: String,
    pub endpoint: String,
    pub endpoint_type: String,
    #[builder(default)]
    pub endpoint_properties: Vec<EndpointProperty>,
}

impl DataAddress {
    pub fn get_property(&self, name: &str) -> Option<&str> {
        self.endpoint_properties
            .iter()
            .find(|p| p.name == name)
            .map(|p| p.value.as_str())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Builder, PartialEq)]
#[serde(rename_all = "camelCase")]
#[builder(on(String, into))]
pub struct EndpointProperty {
    #[builder(default = "EndpointProperty".to_string())]
    #[serde(rename = "@type")]
    pub kind: String,
    pub name: String,
    pub value: String,
}
