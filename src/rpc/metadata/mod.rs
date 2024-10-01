use dmtri::demeter::ops::v1alpha as proto;
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use miette::IntoDiagnostic;
use serde::Deserialize;
use tonic::transport::Channel;

use super::get_base_url;

pub async fn find() -> miette::Result<Vec<ResourceMetadata>> {
    let rpc_url = get_base_url();
    let channel = Channel::builder(rpc_url.parse().into_diagnostic()?)
        .connect()
        .await
        .into_diagnostic()?;

    let mut client = proto::metadata_service_client::MetadataServiceClient::new(channel);

    let request = tonic::Request::new(proto::FetchMetadataRequest::default());
    let response = client.fetch_metadata(request).await.into_diagnostic()?;
    let records = response.into_inner().records;

    let metadata: Vec<ResourceMetadata> = records
        .iter()
        .map(|m| {
            Ok(ResourceMetadata {
                options: serde_json::from_str(&m.options).into_diagnostic()?,
                crd: serde_json::from_str(&m.crd).into_diagnostic()?,
            })
        })
        .collect::<miette::Result<Vec<_>>>()?;

    Ok(metadata)
}

#[derive(Debug, Deserialize)]
pub struct ResourceMetadataOption {
    pub description: String,
    pub spec: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct ResourceMetadata {
    pub options: Vec<ResourceMetadataOption>,
    pub crd: CustomResourceDefinition,
}
