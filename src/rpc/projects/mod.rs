use dmtri::demeter::ops::v1alpha as proto;
use miette::IntoDiagnostic;
use tonic::transport::Channel;

use crate::{
    init::project::{parse_project_ref, ProjectRef},
    rpc::auth,
};

use super::get_base_url;

pub async fn find_projects(access_token: &str) -> miette::Result<Vec<proto::Project>> {
    let interceptor = auth::interceptor(access_token.to_owned()).await;

    let rpc_url = get_base_url();
    let channel = Channel::builder(rpc_url.parse().into_diagnostic()?)
        .connect()
        .await
        .into_diagnostic()?;

    let mut client =
        proto::project_service_client::ProjectServiceClient::with_interceptor(channel, interceptor);

    let request = tonic::Request::new(proto::FetchProjectsRequest::default());
    let response = client.fetch_projects(request).await.into_diagnostic()?;
    let records = response.into_inner().records;

    Ok(records)
}

pub async fn create_project(access_token: &str, name: &str) -> miette::Result<ProjectRef> {
    let interceptor = auth::interceptor(access_token.to_owned()).await;

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
    let name = projec_inner.name;
    let namespace = projec_inner.namespace;

    let project = parse_project_ref(namespace.to_owned(), name.to_owned());

    Ok(project)
}
