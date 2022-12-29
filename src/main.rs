
use std::path::Path;
use aws_sdk_s3::{
    config,
    types::ByteStream,
    Client, Credentials, Region,
};
use dotenv::dotenv;
use glob::glob;
use std::env;
#[tokio::main]
async fn main() {
    dotenv().ok();
    let access_key_id=env::var("ACCESS_KEY_ID").expect("Failed to load ACCESS_KEY_ID");
    let secrest_key_id=env::var("SECRET_KEY_ID").expect("Failed to load SECRET_KEY_ID");
    let region=env::var("AWS_REGION").expect("Failed to load AWS_REGION");
    let bucket_name=env::var("BUCKET_NAME").expect("Failed to load BUCKET_NAME");
    let cred = Credentials::new(
        access_key_id.clone(),
        secrest_key_id.clone(),
        None,
        None,
        "loaded from .env file",
    );
    let region = Region::new(region.clone());
    let conf_builder = config::Builder::new()
        .region(region.clone())
        .credentials_provider(cred);
    let conf = conf_builder.build();
    let client = Client::from_conf(conf);
    let path = glob(&"./upload_folder\\/**/*".to_owned()).unwrap();
    
    for paths in path {
        let a= format!("{}",paths.unwrap().display());
        

         let path=format!("{}",a.clone());
         
        let name = Path::new(&path)
                        .file_name()
                        .unwrap()
                        .to_str();
        let rename = name.unwrap().to_string();


        let obj = ByteStream::from_path(path)
        .await
        .expect("failed to load object");
        let s3_obj = client
            .put_object()
            .bucket(bucket_name.clone())
            .key(rename.clone())
            .body(obj)
            .content_type(".zip");
        s3_obj.send().await.expect("Failed to upload files");
        let b=format!(
            "https://{}.s3.{}.amazonaws.com/{}",bucket_name.clone(), region.clone(), rename.clone()
            
        );
        println!("{}",b.replace(" ", "%20"));
    }
}
