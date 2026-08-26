use io::{aedat4_decoder::Aedat4, codec::DecoderFactory};
use std::io::Error;
use tokio::fs::File;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    // TODO: REMOVE
    let file = File::open("src/davis346.aedat4").await?;

    let mut decoder = Aedat4::open(&Aedat4, Box::pin(file)).await.map_err(|err| {
        Error::new(
            std::io::ErrorKind::Other,
            format!("error opening file: {:?}", err),
        )
    })?;

    let frames = decoder
        .all_frames()
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, "something went wrong"))?;
    println!("frames: {}", frames.len());

    Ok(())
}
