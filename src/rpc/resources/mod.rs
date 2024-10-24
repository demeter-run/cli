use dmtri::demeter::ops::v1alpha as proto;
use miette::IntoDiagnostic;
use tonic::transport::Channel;

use crate::rpc::{auth, get_base_url};

pub async fn find(api_key: &str, project_id: &str) -> miette::Result<Vec<proto::Resource>> {
    let credential = auth::Credential::Secret((project_id.to_owned(), api_key.to_owned()));
    let interceptor = auth::interceptor(credential).await;

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
        project_id: project_id.to_owned(),
        page: Some(1),
        page_size: Some(100),
    });

    let response = client.fetch_resources(request).await.into_diagnostic()?;
    let records = response.into_inner().records;

    Ok(records)
}

pub async fn find_by_id(
    access_token: &str,
    project_id: &str,
    resource_id: &str,
) -> miette::Result<Vec<proto::Resource>> {
    let credential = auth::Credential::Secret((project_id.to_owned(), access_token.to_owned()));
    let interceptor = auth::interceptor(credential).await;

    let rpc_url = get_base_url();
    let channel = Channel::builder(rpc_url.parse().into_diagnostic()?)
        .connect()
        .await
        .into_diagnostic()?;

    let mut client = proto::resource_service_client::ResourceServiceClient::with_interceptor(
        channel,
        interceptor,
    );

    let request = tonic::Request::new(proto::FetchResourcesByIdRequest {
        id: resource_id.into(),
    });

    let response = client
        .fetch_resources_by_id(request)
        .await
        .into_diagnostic()?;

    let resource = response.into_inner().records;

    Ok(resource)
}

pub async fn create(
    access_token: &str,
    project_id: &str,
    kind: &str,
    spec: &str,
) -> miette::Result<proto::Resource> {
    let credential = auth::Credential::Secret((project_id.to_owned(), access_token.to_owned()));
    let interceptor = auth::interceptor(credential).await;

    let rpc_url = get_base_url();
    let channel = Channel::builder(rpc_url.parse().into_diagnostic()?)
        .connect()
        .await
        .into_diagnostic()?;

    let mut client = proto::resource_service_client::ResourceServiceClient::with_interceptor(
        channel,
        interceptor,
    );

    let request = tonic::Request::new(proto::CreateResourceRequest {
        project_id: project_id.to_owned(),
        kind: kind.to_owned(),
        spec: spec.to_owned(),
    });

    let response = client.create_resource(request).await.into_diagnostic()?;

    let resource = response.into_inner();
    let id = resource.id;
    let kind = resource.kind;

    Ok(proto::Resource {
        id,
        kind,
        ..Default::default()
    })
}

pub async fn delete(access_token: &str, project_id: &str, id: &str) -> miette::Result<()> {
    let credential = auth::Credential::Secret((project_id.to_owned(), access_token.to_owned()));
    let interceptor = auth::interceptor(credential).await;

    let rpc_url = get_base_url();
    let channel = Channel::builder(rpc_url.parse().into_diagnostic()?)
        .connect()
        .await
        .into_diagnostic()?;

    let mut client = proto::resource_service_client::ResourceServiceClient::with_interceptor(
        channel,
        interceptor,
    );

    let request = tonic::Request::new(proto::DeleteResourceRequest { id: id.into() });

    client.delete_resource(request).await.into_diagnostic()?;

    Ok(())
}
