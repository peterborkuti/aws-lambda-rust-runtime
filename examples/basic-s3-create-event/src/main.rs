use aws_lambda_events::event::s3::S3Event;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};

/// This is the main body for the function.
/// You can use the event from S3 here.
pub(crate) async fn function_handler(event: LambdaEvent<S3Event>) -> Result<String, Error> {
    let mut result = String::default();
    let records = event.payload.records;
    for record in records.iter() {
        if record.event_name.is_none() {
            continue;
        }
        if !record.event_name.as_ref().unwrap().starts_with("ObjectCreated") {
            continue;
        }

        if record.s3.bucket.name.is_none() || record.s3.object.key.is_none() {
            continue;
        }
        let bucket_name = record.s3.bucket.name.as_ref().unwrap();
        let object_key = record.s3.object.key.as_ref().unwrap();

        if !bucket_name.is_empty() && !object_key.is_empty() {
            result = format!("Object created on S3: {}, {}", bucket_name, object_key);
            tracing::info!(result);
        }
    }

    Ok(result)
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

#[cfg(test)]
mod tests {
    use aws_lambda_events::s3::S3Event;
    use basic_s3_test_helpers::get_s3_event_record;

    use crate::function_handler;

    #[tokio::test]
    async fn test_my_lambda_handler() {
        let mut s3_event = S3Event::default();
        s3_event
            .records
            .push(get_s3_event_record("ObjectCreated", "BUCKET", "KEY"));

        let context = lambda_runtime::Context::default();

        let event = lambda_runtime::LambdaEvent::new(s3_event, context);

        let result = function_handler(event).await.unwrap();

        assert_eq!(result, "Object created on S3: BUCKET, KEY");
    }
}
