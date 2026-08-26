use std::{io::ErrorKind, path};

use async_trait::async_trait;
///
/// Decoder for the AEDAT4 file format decribed [here](https://docs.inivation.com/software/software-advanced-usage/file-formats/aedat-4.0.html).
///
use tokio::io::{self, AsyncReadExt, BufReader};

#[path = "../generated/aedat4_header_generated.rs"]
#[allow(unsafe_code)]
mod aedat4_header_generated;
use aedat4_header_generated::*;

#[path = "../generated/aedat4_data_generated.rs"]
#[allow(unsafe_code)]
mod aedat4_data_generated;
use aedat4_data_generated::*;

use crate::{
    codec::{ByteSource, Codec, Decoder, DecoderFactory, VideoMetadata},
    error::{DecodeError, FileReadError},
    file::{AEDAT4_FORMAT, FileFormat},
    frame::Frame,
};

/// Struct representing the state of AEDAT4 codec.
pub struct Aedat4;

async fn read_exact_and_handle_err(
    buffered_reader: &mut BufReader<ByteSource>,
    buffer: &mut [u8],
) -> Result<(), FileReadError> {
    return match buffered_reader
        .read_exact(buffer)
        .await
        .map_err(|err| match err.kind() {
            io::ErrorKind::UnexpectedEof => FileReadError::UnexpectedEOF(
                "unexpected EOF: file ended before magic bytes ended".to_string(),
            ),
            _ => FileReadError::UnexpectedError(
                io::ErrorKind::Other,
                format!("unexpected I/O error: {}", err),
            ),
        }) {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    };
}

impl Codec for Aedat4 {
    fn format(&self) -> &FileFormat {
        &AEDAT4_FORMAT
    }

    fn validate(&self, data: &[u8]) -> Result<(), FileReadError> {
        if !data.starts_with(self.format().magic_bytes.unwrap()) {
            return Err(FileReadError::InvalidMagicBytes);
        }
        Ok(())
    }
}

impl DecoderFactory for Aedat4 {
    async fn open(&self, source: ByteSource) -> Result<Box<dyn Decoder>, FileReadError> {
        // check the magic bytes and parse the IOHeader to get metadata

        let mut buffered_reader = BufReader::new(source);
        let magic_bytes = self.format().magic_bytes.unwrap();

        let mut buffer = vec![0; magic_bytes.len()];

        read_exact_and_handle_err(&mut buffered_reader, &mut buffer).await?;

        self.validate(&buffer)?;

        // read IOHE

        let mut io_buffer = vec![0u8; 4];
        read_exact_and_handle_err(&mut buffered_reader, &mut io_buffer).await?;
        let header_size = u32::from_le_bytes(io_buffer.try_into().unwrap()) as usize;

        let mut iohe_content = vec![0u8; 4 + header_size]; // parser expects a 4 byte size prefix when using `size_prefixed_root_...`
        read_exact_and_handle_err(&mut buffered_reader, &mut iohe_content[4..]).await?;
        let iohe_root = aedat4_header_generated::size_prefixed_root_as_ioheader(&iohe_content)
            .map_err(|_err| {
                FileReadError::UnexpectedError(
                    ErrorKind::InvalidData,
                    String::from(
                        "IOHeader is malformed, expected a valid, size-prefixed flatbuffer.",
                    ),
                )
            })?;

        println!("iohe_root: {:?}", iohe_root);

        Ok(Box::new(Aedat4Decoder {
            offset: iohe_root.data_table_offset(),
            current_timestamp: 0,
            current_frame: 1,
            buffered_reader,
            metadata: None, // TODO: extract metadata from xml in iohe_root
        }))
    }
}

/// State for the AEDAT4 decoder.
struct Aedat4Decoder {
    offset: i64,
    /// The current timestamp in microseconds.
    current_timestamp: i64,
    /// The current frame number.
    current_frame: i64,
    /// The buffered reader for reading the AEDAT4 file.
    buffered_reader: BufReader<ByteSource>,
    /// The metadata for the video.
    metadata: Option<VideoMetadata>,
}

#[async_trait]
impl Decoder for Aedat4Decoder {
    fn metadata(&self) -> &VideoMetadata {
        self.metadata.as_ref().unwrap()
    }

    async fn next_frame(&mut self) -> Result<Option<Frame>, DecodeError> {
        // parse packet header, this is exactly 8 bytes
        let mut header_buffer = [0u8; 8];
        self.buffered_reader
            .read_exact(&mut header_buffer)
            .await
            .map_err(|e| DecodeError::UnexpectedError(ErrorKind::Other, e.to_string()))?;

        println!("saw header: {:?}", header_buffer.to_ascii_lowercase());

        let header =
            aedat4_data_generated::root_as_packet(&mut header_buffer).expect("header not found");

        println!("{:?}", header);
        Ok(None)
    }

    async fn all_frames(&mut self) -> Result<Vec<Frame>, DecodeError> {
        let mut frames = Vec::new();
        while let Some(frame) = self.next_frame().await? {
            frames.push(frame);
        }
        Ok(frames)
    }
}
