use tokio::fs::File;
use tokio::io::AsyncReadExt;

pub async fn read_file_to_string(path: &str) -> Result<String, std::io::Error> {
    let mut file = File::open(path).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    Ok(contents)
}
