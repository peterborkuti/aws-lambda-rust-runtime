use aws_lambda_events::event::s3::S3Event;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};

/// This is the main body for the function.
/// You can use the event from S3 here.
pub(crate) async fn function_handler(event: LambdaEvent<S3Event>) -> Result<String, Error> {
    let data = event.payload.records;
    let len = data.len();
    let response = format!("{} records received from S3", len);

    Ok(response)
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

    use crate::function_handler;

    #[tokio::test]
    async fn test_my_lambda_handler() {
        let s3_event = S3Event::default();

        let context = lambda_runtime::Context::default();

        let event = lambda_runtime::LambdaEvent::new(s3_event, context);

        let result = function_handler(event).await.unwrap();

        assert_eq!(result, "0 records received from S3");
    }
}
