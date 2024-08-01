use dmtri::demeter::ops::v1alpha as proto;
use miette::IntoDiagnostic;
use tonic::transport::Channel;

use crate::rpc::auth;

pub async fn find_projects(access_token: &str) -> miette::Result<Vec<proto::Project>> {
    let interceptor = auth::interceptor(access_token.to_owned()).await;

    let channel = Channel::from_static("http://0.0.0.0:5001")
        .connect()
        .await
        .into_diagnostic()?;

    let mut client =
        proto::project_service_client::ProjectServiceClient::with_interceptor(channel, interceptor);

    println!("searching for projects...");

    let request = tonic::Request::new(proto::FetchProjectsRequest::default());

    println!("request: {:?}", request);

    let response = client.fetch_projects(request).await.into_diagnostic()?;

    let records = response.into_inner().records;

    Ok(records)
}
