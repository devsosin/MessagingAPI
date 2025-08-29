use reqwest::{Client, multipart};
use tokio::fs;
use uuid::Uuid;

use crate::{errors::ClientError, types::ClientResult};

pub async fn get_image() {}

pub fn get_mime_type(image_url: &str) -> String {
    let ext = image_url.split(".").last().unwrap();

    match ext.to_lowercase().as_str() {
        "jpg" => "image/jpeg",
        "jpeg" => "image/jpeg",
        "png" => "image/png",
        "avif" => "image/avif",
        "webp" => "image/webp",
        _ => "image/jpeg",
    }
    .to_lowercase()
}

pub async fn get_file_part(path_or_url: &str) -> ClientResult<multipart::Part> {
    let mime_type = get_mime_type(path_or_url);

    let file: ClientResult<Vec<u8>> = if path_or_url.contains("http") {
        let res = Client::new()
            .get(path_or_url)
            .send()
            .await
            .map_err(|_| ClientError::InternalError("Get Image Error"))?;
        Ok(res.bytes().await.unwrap().to_vec())
    } else {
        let result = fs::read(path_or_url)
            .await
            .map_err(|_| ClientError::InternalError("File Open Error"))?;
        Ok(result)
    };

    let file = file?;

    let part = multipart::Part::bytes(file)
        .file_name(format!("image.{mime_type}"))
        .mime_str(&mime_type)
        .unwrap();
    Ok(part)
}

pub fn get_uuid() -> String {
    Uuid::new_v4().simple().to_string()
}
