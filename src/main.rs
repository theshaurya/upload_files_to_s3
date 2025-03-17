use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use dotenv::dotenv;
use glob::glob;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

pub async fn new_client() -> Client {
    let region_provider = RegionProviderChain::default_provider();
    let config = aws_config::from_env().region(region_provider).load().await;
    Client::new(&config)
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let region = env::var("AWS_REGION").expect("Failed to load AWS_REGION");
    let bucket_name = env::var("AWS_BUCKET_NAME").expect("Failed to load BUCKET_NAME");
    let file_name = env::var("FILE_JSON").expect("Failed get path FILE_JSON");

    let client = new_client().await;
    let base_path = PathBuf::from("./upload_folder")
        .canonicalize()
        .expect("Invalid base path");

    let pattern = format!("{}/**/*", base_path.display());
    let paths = glob(&pattern).expect("Failed to read glob pattern");
    let mut data = Vec::new();
    let file = File::create(&file_name).unwrap();
    let mut writer = BufWriter::new(file);
    for entry in paths.flatten() {
        if entry.is_file() {
            use serde_json::json;

            // Ensure entry is canonicalized before stripping prefix
            let entry_abs = entry.canonicalize().expect("Failed to get absolute path");

            // Strip prefix safely
            let relative_path = entry_abs
                .strip_prefix(&base_path)
                .expect("Failed to strip prefix")
                .to_string_lossy();

            // update path
            let s3_key = format!("{}", relative_path)
                .to_lowercase()
                .replace(' ', "-")
                .replace('&', "_");
            // some condition to upload specific file
            // for now it is true
            if true {
                let obj = ByteStream::from_path(&entry)
                    .await
                    .expect("Failed to load object");

                client
                    .put_object()
                    .bucket(&bucket_name)
                    .key(&s3_key)
                    .body(obj)
                    .send()
                    .await
                    .expect("Failed to upload file");

                let url = format!(
                    "https://{}.s3.{}.amazonaws.com/{}",
                    bucket_name, region, s3_key
                );

                let d = json!({
                    "uploading": format!("{}", entry.to_string_lossy()),
                    "url": url.replace(" ", "%20")
                });
                data.push(d);
            }
        }
    }

    serde_json::to_writer(&mut writer, &data).unwrap();
    writer.flush().unwrap();
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn delete() {
        dotenv::dotenv().ok();
        let bucket_name = env::var("BUCKET_NAME").expect("Failed to load BUCKET_NAME");
        let key = vec![
            ".ds_store",
            "general/.ds_store",
            "general/2025/.ds_store",
            "general/2025/03/.ds_store",
        ];
        for key in key {
            let client = new_client().await;
            let d = client
                .delete_object()
                .bucket(&bucket_name)
                .key(key)
                .send()
                .await;
            dbg!(&d);
        }
    }
}
