use dmtri::demeter::ops::v1alpha as proto;
use miette::IntoDiagnostic;
use tonic::transport::Channel;

use crate::rpc::{auth, get_base_url};

pub async fn find(access_token: &str, id: &str) -> miette::Result<Vec<proto::Resource>> {
    let interceptor = auth::interceptor(access_token.to_owned()).await;

    let rpc_url = get_base_url();
    let channel = Channel::builder(rpc_url.parse().into_diagnostic()?)
        .connect()
        .await
        .into_diagnostic()?;

    let mut client = proto::resource_service_client::ResourceServiceClient::with_interceptor(
        channel,
        interceptor,
    );

    let request = tonic::Request::new(proto::FetchResourcesRequest {
        project_id: id.to_owned(),
        ..Default::default()
    });

    let response = client.fetch_resources(request).await.into_diagnostic()?;
    let records = response.into_inner().records;

    Ok(records)
}
