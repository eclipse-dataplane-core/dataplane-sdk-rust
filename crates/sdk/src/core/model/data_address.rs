use bon::Builder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Builder, PartialEq)]
#[serde(rename_all = "camelCase")]
#[builder(on(String, into))]
pub struct DataAddress {
    #[builder(default = "DataAddress".to_string())]
    #[serde(rename = "@type")]
    pub kind: String,
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
