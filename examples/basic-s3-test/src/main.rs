
use aws_config::meta::region::RegionProviderChain;
use aws_lambda_events::{event::s3::S3Event};
use aws_sdk_s3::{types::ByteStream, Client};
use aws_smithy_http::body::SdkBody;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde::Serialize;

#[derive(Serialize)]
struct Response {
    req_id: String,
    msg: String,
}

struct SharedClient {
    client: &'static Client,
}

impl SharedClient {
    fn new(client: &'static Client) -> Self {
        Self { client }
    }

    pub async fn put_file_x(&self) -> Response {
        let stream = ByteStream::new(SdkBody::from("hello"));
        let _ = self.put_file("bucket", "key", stream).await;

        Response { req_id: "1".to_string(), msg: "msg".to_string()}
    }

    pub async fn put_file(&self, bucket: &str, key: &str, bytes: ByteStream) {
        let _ = self
            .client
            .put_object()
            .bucket(bucket)
            .key(key)
            .body(bytes)
            .send()
            .await;
    }
}

async fn get_client() -> Client {
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-2");
    let config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&config);

    return client;
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let s3_client = get_client().await;
    let client = move || &SharedClient::new(&s3_client);

    let client_ref = client();

    run(service_fn(
        move |_event: LambdaEvent<S3Event>| async move {
            Ok::<Response, Error>(client_ref.put_file_x().await)
        }
    ))
    .await?;
    Ok(())
}

