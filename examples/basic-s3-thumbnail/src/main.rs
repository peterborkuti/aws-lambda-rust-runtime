use std::io::Cursor;

use aws_config::meta::region::RegionProviderChain;
use aws_lambda_events::{event::s3::S3Event, s3::S3EventRecord};
use aws_sdk_s3::{types::ByteStream, Client};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use thumbnailer::{create_thumbnails, ThumbnailSize};

struct FileProps {
    bucket_name: String,
    object_key: String,
}

/// This lambda handler listen to file creation events and it creates a thumbnail
/// and uploads it to s3 into a bucket "[original bucket name]-thumbs".
///
/// Make sure that this lambda only gets event from png file creation
/// Make sure that there is another bucket with "-thumbs" prefix in the name
/// Make sure that this lambda has permission to put file into the "-thumbs" bucket
/// Make sure that the created png file has no strange characters in the name
pub(crate) async fn function_handler(event: LambdaEvent<S3Event>) -> Result<(), Error> {
    let client = get_client().await;

    let records = event.payload.records;
    for record in records.iter() {
        let optional_file_props = get_file_props(record);
        if optional_file_props.is_none() {
            // The event is not a create event or bucket/object key is missing
            println!("record skipped");
            continue;
        }

        // The event is a CreateObject and it contains the bucket name and
        // object_key
        // If the object_key has strange characters, the upload may not work
        // https://docs.aws.amazon.com/AmazonS3/latest/userguide/object-keys.html
        // Try it with something simple like this: abc_123.png
        let file_props = optional_file_props.unwrap();
        let name = file_props.bucket_name.as_str();
        let key = file_props.object_key.as_str();

        let reader = get_file(&client, name, key).await;

        if reader.is_none() {
            continue;
        }

        let thumbnail = get_thumbnail(reader.unwrap());

        let mut thumbs_bucket_name = name.to_owned();
        thumbs_bucket_name.push_str("-thumbs");

        // It uplaods the thumbnail into a bucket name suffixed with "-thumbs"
        // So it needs file creation permission into that bucket
        let _ = put_file(&client, &thumbs_bucket_name, key, thumbnail).await;
    }

    Ok(())
}

async fn get_file(client: &Client, bucket: &str, key: &str) -> Option<Cursor<Vec<u8>>> {
    println!("get file bucket {}, key {}", bucket, key);

    let output = client.get_object().bucket(bucket).key(key).send().await;

    let mut reader = None;

    if output.as_ref().ok().is_some() {
        let bytes = output.ok().unwrap().body.collect().await.unwrap().to_vec();
        println!("Object is downloaded, size is {}", bytes.len());
        reader = Some(Cursor::new(bytes));
    } else if output.as_ref().err().is_some() {
        let err = output.err().unwrap();
        let service_err = err.into_service_error();
        let meta = service_err.meta();
        println!("Error from aws when downloding: {}", meta.to_string());
    } else {
        println!("Unknown error when downloading");
    }

    return reader;
}

async fn put_file(client: &Client, bucket: &str, key: &str, bytes: ByteStream) {
    println!("put file bucket {}, key {}", bucket, key);
    let _ = client.put_object().bucket(bucket).key(key).body(bytes).send().await;

    return;
}

fn get_thumbnail(reader: Cursor<Vec<u8>>) -> ByteStream {
    let mut thumbnails = create_thumbnails(reader, mime::IMAGE_PNG, [ThumbnailSize::Small]).unwrap();

    let thumbnail = thumbnails.pop().unwrap();
    let mut buf = Cursor::new(Vec::new());
    thumbnail.write_png(&mut buf).unwrap();

    return ByteStream::from(buf.into_inner());
}

fn get_file_props(record: &S3EventRecord) -> Option<FileProps> {
    if record.event_name.is_none() {
        return None;
    }
    if !record.event_name.as_ref().unwrap().starts_with("ObjectCreated") {
        return None;
    }

    if record.s3.bucket.name.is_none() || record.s3.object.key.is_none() {
        return None;
    }
    let bucket_name = record.s3.bucket.name.to_owned().unwrap();
    let object_key = record.s3.object.key.to_owned().unwrap();

    if bucket_name.is_empty() || object_key.is_empty() {
        println!("Bucket name ro object_key is empty");
        return None;
    }

    println!("Bucket: {}, Object key: {}", bucket_name, object_key);

    return Some(FileProps {
        bucket_name: (bucket_name),
        object_key: (object_key),
    });
}

async fn get_client() -> Client {
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-2");
    let config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&config);

    println!("client region {}", client.conf().region().unwrap().to_string());

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
