use std::io::Write;
use std::path::{Path, PathBuf};
use actix_web::{web, http::{HeaderMap, header::CONTENT_LENGTH}};
use actix_multipart::{Field};
use bytes::{Bytes, BytesMut};
use futures_util::stream::StreamExt;

pub fn read_content_length(headers: &HeaderMap) -> Option<u64> {
    let value = headers.get(CONTENT_LENGTH)?;
    value.to_str().ok()
        .and_then(|s| s.parse::<u64>().ok())
}

#[allow(dead_code)]
pub async fn read_body(field: &mut Field) -> Bytes {
    let mut b = BytesMut::new();
    loop {
        match field.next().await {
            Some(Ok(chunk)) => b.extend_from_slice(&chunk),
            None => return b.freeze(),
            _ => error!("unreachable")
        }
    }
}

type ByteStream = Field;

pub struct SaveUploadedFileOptions<'a> {
    pub directory: &'a str,
    pub path: &'a str,
    pub bytes_min: Option<usize>,
    pub bytes_max: Option<usize>,
}

async fn try_save_uploaded_file<'a>(stream: &mut ByteStream, path: &Path,
                                    options: SaveUploadedFileOptions<'a>) ->
    Result<(), String>
{
    let mut f = std::fs::File::create(path.clone())
        .map_err(|e| format!("{}", &e))?;

    let mut bytes_total = 0;

    // Field in turn is stream of *Bytes* object
    while let Some(chunk) = stream.next().await {
        let data = chunk.unwrap();
        bytes_total += data.len();
        match options.bytes_max {
            Some(max) if bytes_total > max => Err("File too large"),
            _ => Ok(())
        }?;
        // Run asynchronously with threadpool
        f = web::block(move || f.write_all(&data).map(|_| f)).await
            .map_err(|e| format!("{}", &e))?;
    };

    match options.bytes_min {
        Some(min) if bytes_total < min => Err("File too smarll"),
        _ => Ok(())
    }?;

    Ok(())
}

pub async fn save_uploaded_file<'a>(stream: &mut ByteStream,
                                    options: SaveUploadedFileOptions<'a>) ->
    Result<PathBuf, String>
{
    // Ensure directory to save
    std::fs::create_dir_all(Path::new(options.directory))
        .map_err(|e| format!("{}", &e))?;

    let path = format!("{}/{}", &options.directory, &options.path);
    let path = Path::new(path.as_str());
    match try_save_uploaded_file(stream, &path, options).await {
        Ok(()) => Ok(path.to_owned()),
        Err(e) => {
            error!("Canceled to save uploaded file, e: {}", &e);

            // Delete file on error
            if path.is_file() {
                debug!("Delete file, path: {:?}", &path);
                std::fs::remove_file(path).unwrap();
            }

            Err(format!("{}", &e).to_string())
        }
    }
}
