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
use dataplane_sdk::core::model::{
    data_address::DataAddress,
    messages::{DataFlowStartMessage, DataFlowStartedNotificationMessage, DataFlowStatusMessage},
};
use uuid::Uuid;

#[derive(Builder)]
#[builder(on(String, into))]
pub struct ControlPlaneSimulator {
    consumer: String,
    provider: String,
    #[builder(default = reqwest::Client::new())]
    client: reqwest::Client,
}

#[derive(Builder)]
#[builder(on(String, into))]
pub struct DataPlaneRequest {
    pub dataset_id: String,
    pub process_id: String,
    pub agreement_id: String,
    pub data_address: Option<DataAddress>,
}

impl ControlPlaneSimulator {
    pub async fn provider_start(&self, req: DataPlaneRequest) -> anyhow::Result<DataAddress> {
        let url = format!("{}/dataflows/start", self.provider);

        let msg = DataFlowStartMessage::builder()
            .dataset_id(req.dataset_id)
            .process_id(req.process_id)
            .agreement_id(req.agreement_id)
            .maybe_data_address(req.data_address)
            .callback_address(format!("{}/callback", self.provider))
            .participant_id("control-plane-simulator")
            .counter_party_id("counter-party")
            .dataspace_context("example-dataspace")
            .transfer_type("example-transfer-type")
            .message_id(Uuid::new_v4().to_string())
            .build();

        let response = self.client.post(&url).json(&msg).send().await?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Failed to start data flow: {} - {}",
                response.status(),
                response.text().await?
            );
        }

        let body = response.json::<DataFlowStatusMessage>().await?;

        body.data_address
            .ok_or_else(|| anyhow::anyhow!("No data address returned from provider"))
    }
    pub async fn consumer_prepare(&self, req: DataPlaneRequest) -> anyhow::Result<()> {
        let url = format!("{}/dataflows/prepare", self.consumer);

        let msg = DataFlowStartMessage::builder()
            .dataset_id(req.dataset_id)
            .process_id(req.process_id)
            .agreement_id(req.agreement_id)
            .maybe_data_address(req.data_address)
            .callback_address(format!("{}/callback", self.consumer))
            .participant_id("control-plane-simulator")
            .counter_party_id("counter-party")
            .dataspace_context("example-dataspace")
            .transfer_type("example-transfer-type")
            .message_id(Uuid::new_v4().to_string())
            .build();

        let response = self.client.post(&url).json(&msg).send().await?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Failed to prepare data flow: {} - {}",
                response.status(),
                response.text().await?
            );
        }

        Ok(())
    }

    pub async fn consumer_started(
        &self,
        process_id: &str,
        data_address: DataAddress,
    ) -> anyhow::Result<()> {
        let url = format!("{}/dataflows/{}/started", self.consumer, process_id);

        let msg = DataFlowStartedNotificationMessage::builder()
            .data_address(data_address)
            .build();

        let response = self.client.post(&url).json(&msg).send().await?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Failed to notify started data flow: {} - {}",
                response.status(),
                response.text().await?
            );
        }

        Ok(())
    }

    pub async fn internal_start(
        &self,
        base_url: &str,
        req: DataPlaneRequest,
    ) -> anyhow::Result<Option<DataAddress>> {
        let url = format!("{}/dataflows/start", base_url);

        let msg = DataFlowStartMessage::builder()
            .dataset_id(req.dataset_id)
            .process_id(req.process_id)
            .agreement_id(req.agreement_id)
            .maybe_data_address(req.data_address)
            .callback_address(format!("{}/callback", base_url))
            .participant_id("control-plane-simulator")
            .counter_party_id("counter-party")
            .dataspace_context("example-dataspace")
            .transfer_type("example-transfer-type")
            .message_id(Uuid::new_v4().to_string())
            .build();

        let response = self.client.post(&url).json(&msg).send().await?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Failed to start data flow: {} - {}",
                response.status(),
                response.text().await?
            );
        }

        let body = response.json::<DataFlowStatusMessage>().await?;

        Ok(body.data_address)
    }
}
