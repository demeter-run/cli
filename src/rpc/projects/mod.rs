use dmtri::demeter::ops::v1alpha as proto;
use miette::IntoDiagnostic;
use tonic::transport::Channel;

use crate::{
    init::project::{parse_project_ref, ProjectRef},
    rpc::auth,
};

use super::get_base_url;

pub async fn find(access_token: &str) -> miette::Result<Vec<proto::Project>> {
    let credential = auth::Credential::Auth0(access_token.to_owned());
    let interceptor = auth::interceptor(credential).await;

    let rpc_url = get_base_url();
    let channel = Channel::builder(rpc_url.parse().into_diagnostic()?)
        .connect()
        .await
        .into_diagnostic()?;

    let mut client =
        proto::project_service_client::ProjectServiceClient::with_interceptor(channel, interceptor);

    let request = tonic::Request::new(proto::FetchProjectsRequest {
        page: Some(1),
        page_size: Some(100),
    });

    let response = client.fetch_projects(request).await.into_diagnostic()?;
    let records = response.into_inner().records;

    Ok(records)
}

pub async fn find_by_namespace(
    credential: auth::Credential,
    namespace: &str,
) -> miette::Result<proto::Project> {
    let interceptor = auth::interceptor(credential).await;

    let rpc_url = get_base_url();
    let channel = Channel::builder(rpc_url.parse().into_diagnostic()?)
        .connect()
        .await
        .into_diagnostic()?;

    let mut client =
        proto::project_service_client::ProjectServiceClient::with_interceptor(channel, interceptor);

    let request = tonic::Request::new(proto::FetchProjectByNamespaceRequest {
        namespace: namespace.into(),
    });

    let response = client
        .fetch_project_by_namespace(request)
        .await
        .into_diagnostic()?;

    let record = &response.into_inner().records[0];

    Ok(record.clone())
}

pub async fn create_project(access_token: &str, name: &str) -> miette::Result<ProjectRef> {
    let credential = auth::Credential::Auth0(access_token.to_owned());
    let interceptor = auth::interceptor(credential).await;

    let rpc_url = get_base_url();
    let channel = Channel::builder(rpc_url.parse().into_diagnostic()?)
        .connect()
        .await
        .into_diagnostic()?;

    let mut client =
        proto::project_service_client::ProjectServiceClient::with_interceptor(channel, interceptor);

    let request = tonic::Request::new(proto::CreateProjectRequest {
        name: name.to_owned(),
    });

    let response = client.create_project(request).await.into_diagnostic()?;
    let projec_inner = response.into_inner();
    let id = projec_inner.id;
    let name = projec_inner.name;
    let namespace = projec_inner.namespace;

    let project = parse_project_ref(id, namespace, name);

    Ok(project)
}

pub async fn create_secret(
    access_token: &str,
    project_id: &str,
    name: &str,
) -> miette::Result<String> {
    let credential = auth::Credential::Auth0(access_token.to_owned());
    let interceptor = auth::interceptor(credential).await;

    let rpc_url = get_base_url();
    let channel = Channel::builder(rpc_url.parse().into_diagnostic()?)
        .connect()
        .await
        .into_diagnostic()?;

    let mut client =
        proto::project_service_client::ProjectServiceClient::with_interceptor(channel, interceptor);

    let request = tonic::Request::new(proto::CreateProjectSecretRequest {
        project_id: project_id.to_owned(),
        name: name.to_owned(),
    });

    let response = client
        .create_project_secret(request)
        .await
        .into_diagnostic()?;

    let api_key = response.into_inner().key;
    println!("API key: {:?}", api_key);

    Ok(api_key)
}
