use std::io::Cursor;

use aws_config::meta::region::RegionProviderChain;
use aws_lambda_events::{event::s3::S3Event, s3::S3EventRecord};
use aws_sdk_s3::{Client, types::{ByteStream}};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use thumbnailer::{create_thumbnails, ThumbnailSize};

struct FileProps {
    bucket_name: String,
    object_key: String
}

/// This is the main body for the function.
/// You can use the event from S3 here.
pub(crate) async fn function_handler(event: LambdaEvent<S3Event>) -> Result<(), Error> {
    let client = get_client().await;

    let records = event.payload.records;
    for record in records.iter() {
        let optional_file_props = get_file_props(record);
        if optional_file_props.is_none() {
            continue;
        }

        let file_props = optional_file_props.unwrap();
        let name = file_props.bucket_name.as_str();
        let key = file_props.object_key.as_str();

        let reader = get_file(&client, name, key).await;
        let thumbnail = get_thumbnail(reader);

        let mut thumbs_bucket_name = name.to_owned();
        thumbs_bucket_name.push_str("-thumbs");

        let _ = put_file(&client, &thumbs_bucket_name, key, thumbnail).await;
    }

    Ok(())
}

async fn get_file(client: &Client, bucket: &str, key: &str) -> Cursor<Vec<u8>> {
    let output = client
    .get_object()
    .bucket(bucket)
    .key(key)
    .send()
    .await.ok().unwrap();

    let bytes = output.body.collect().await.unwrap().to_vec();

    let reader = Cursor::new(bytes);

    return reader;
}

async fn put_file(client: &Client, bucket: &str, key: &str, bytes: ByteStream) {
    let _ = client
    .put_object()
    .bucket(bucket)
    .key(key)
    .body(bytes)
    .send()
    .await;

    return
}

fn get_thumbnail(reader: Cursor<Vec<u8>>) -> ByteStream {

    let mut  thumbnails = create_thumbnails(
        reader,
        mime::IMAGE_PNG,
        [ThumbnailSize::Small])
        .unwrap();

    let thumbnail = thumbnails.pop().unwrap();
    let mut buf = Cursor::new(Vec::new());
    thumbnail.write_png(&mut buf).unwrap();

    return ByteStream::from(buf.into_inner());
}

fn get_file_props(record: &S3EventRecord) -> Option<FileProps> {
        if record.event_name.is_none() { return None; }
        if !record.event_name.as_ref().unwrap().starts_with("ObjectCreated") {
            return None;
        }

        if record.s3.bucket.name.is_none() || record.s3.object.key.is_none() {
            return None;
        }
        let bucket_name = record.s3.bucket.name.to_owned().unwrap();
        let object_key = record.s3.object.key.to_owned().unwrap();

        if bucket_name.is_empty() || object_key.is_empty() {
            return None;
        }

        return Some(FileProps { bucket_name: (bucket_name), object_key: (object_key) });
}

async fn get_client() -> Client {
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&config);

    return client;
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // this needs to be set to false, otherwise ANSI color codes will
        // show up in a confusing manner in CloudWatch logs.
        .with_ansi(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}
