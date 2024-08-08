use dmtri::demeter::ops::v1alpha as proto;
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use miette::IntoDiagnostic;
use tonic::transport::Channel;

use super::get_base_url;

pub async fn find() -> miette::Result<Vec<CustomResourceDefinition>> {
    let rpc_url = get_base_url();
    let channel = Channel::builder(rpc_url.parse().into_diagnostic()?)
        .connect()
        .await
        .into_diagnostic()?;

    let mut client = proto::metadata_service_client::MetadataServiceClient::new(channel);

    let request = tonic::Request::new(proto::FetchMetadataRequest::default());
    let response = client.fetch_metadata(request).await.into_diagnostic()?;
    let records = response.into_inner().records;

    let crds: Vec<CustomResourceDefinition> = records
        .iter()
        .map(|json| serde_json::from_str(json))
        .collect::<Result<_, _>>()
        .into_diagnostic()?;

    Ok(crds)
}
