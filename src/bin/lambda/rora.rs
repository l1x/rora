use async_once::AsyncOnce;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::model::Bucket;
use aws_sdk_s3::Client as S3Client;
// use aws_sdk_s3::Error as S3Error;
use chrono::{DateTime, Datelike, Timelike, Utc};
use lambda_http::Error as LambdaHttpError;
use lambda_http::{run, service_fn, Body, Error, Request, Response};
use lazy_static::lazy_static;

lazy_static! {
    static ref S3_CLIENT: AsyncOnce<S3Client> = AsyncOnce::new(async {
        let region_provider = RegionProviderChain::default_provider().or_else("eu-west-1");
        let config = aws_config::from_env().region(region_provider).load().await;

        S3Client::new(&config)
    });
}

#[allow(dead_code)]
fn utc_now() -> String {
    let now: DateTime<Utc> = Utc::now();
    format!(
        "{}-{}-{}T{}:{}:{}Z",
        now.year(),
        now.month(),
        now.day(),
        now.hour(),
        now.minute(),
        now.second(),
    )
}

fn get_bucket_names(buckets: Option<&[Bucket]>) -> String {
    let buckets_unwrap = buckets.unwrap_or_default();
    let bucket_names: String = buckets_unwrap
        .iter()
        .map(|bucket| format!("<li>{}</li>", bucket.name().unwrap_or_default()))
        .collect();

    format!(
        "Number of buckets: {} <br> <ul>{}</ul>",
        buckets_unwrap.len(),
        bucket_names
    )
}

#[allow(dead_code)]
async fn list_buckets(s3_client: &S3Client) -> String {
    let aws_response = s3_client.list_buckets().send().await;

    match aws_response {
        Ok(list_buckets_output) => get_bucket_names(list_buckets_output.buckets()),
        Err(err) => format!("{}", err),
    }
}

async fn function_handler(_event: Request) -> Result<Response<Body>, LambdaHttpError> {
    let s3_client = S3_CLIENT.get().await;

    let bucket_list_html = list_buckets(s3_client).await;

    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body(bucket_list_html.into())
        .map_err(Box::new)?;
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();
    run(service_fn(function_handler)).await
}
