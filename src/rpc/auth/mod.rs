use tonic::metadata::MetadataValue;
use tonic::service::Interceptor;
use tonic::Request;

pub type Secret = String;
pub type ProjectId = String;

pub enum Credential {
    Auth0(String),
    Secret((ProjectId, Secret)),
}

pub async fn interceptor(credential: Credential) -> impl Interceptor {
    move |mut req: Request<()>| {
        match &credential {
            Credential::Auth0(token) => {
                req.metadata_mut().insert(
                    "authorization",
                    MetadataValue::try_from(&format!("Bearer {}", token)).unwrap(),
                );
            }
            Credential::Secret((project_id, secret)) => {
                req.metadata_mut()
                    .insert("project-id", MetadataValue::try_from(project_id).unwrap());
                req.metadata_mut()
                    .insert("dmtr-api-key", MetadataValue::try_from(secret).unwrap());
            }
        };

        Ok(req)
    }
}
