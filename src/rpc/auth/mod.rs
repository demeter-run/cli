use tonic::service::Interceptor;
use tonic::{Request, Status};

pub async fn interceptor(access_token: String) -> impl Interceptor {
    move |mut req: Request<()>| {
        req.metadata_mut().insert(
            "authorization",
            format!("Bearer {}", access_token).parse().unwrap(),
        );
        Ok(req)
    }
}
