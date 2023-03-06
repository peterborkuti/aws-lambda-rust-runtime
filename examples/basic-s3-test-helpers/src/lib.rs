use std::collections::HashMap;

use aws_lambda_events::{
    chrono::DateTime,
    s3::{S3Bucket, S3Entity, S3EventRecord, S3Object, S3RequestParameters, S3UserIdentity},
};

pub fn get_s3_event_record(event_name: &str, bucket_name: &str, object_key: &str) -> S3EventRecord {
    let s3_entity = S3Entity {
        schema_version: (Some(String::default())),
        configuration_id: (Some(String::default())),
        bucket: (S3Bucket {
            name: (Some(bucket_name.to_string())),
            owner_identity: (S3UserIdentity {
                principal_id: (Some(String::default())),
            }),
            arn: (Some(String::default())),
        }),
        object: (S3Object {
            key: (Some(object_key.to_string())),
            size: (Some(1)),
            url_decoded_key: (Some(String::default())),
            version_id: (Some(String::default())),
            e_tag: (Some(String::default())),
            sequencer: (Some(String::default())),
        }),
    };

    return S3EventRecord {
        event_version: (Some(String::default())),
        event_source: (Some(String::default())),
        aws_region: (Some(String::default())),
        event_time: (DateTime::default()),
        event_name: (Some(event_name.to_string())),
        principal_id: (S3UserIdentity {
            principal_id: (Some("X".to_string())),
        }),
        request_parameters: (S3RequestParameters {
            source_ip_address: (Some(String::default())),
        }),
        response_elements: (HashMap::new()),
        s3: (s3_entity),
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = get_s3_event_record("EVENT_NAME", "BUCKET_NAME", "OBJECT_KEY");
        assert_eq!("EVENT_NAME", result.event_name.unwrap());
        assert_eq!("BUCKET_NAME", result.s3.bucket.name.unwrap());
        assert_eq!("OBJECT_KEY", result.s3.object.key.unwrap());
    }
}
