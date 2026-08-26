use std::pin::Pin;

use async_trait::async_trait;
use tokio::io::AsyncRead;

use crate::{
    error::{DecodeError, FileReadError},
    file::FileFormat,
    frame::Frame,
};

/// (asyncronous) source of bytes, e.g. a file, network stream etc.
pub type ByteSource = Pin<Box<dyn AsyncRead + Send>>;

/// Represents a codec which decodes / encodes a video file.
pub trait Codec {
    /// Returns the file format of the input file.
    fn format(&self) -> &FileFormat;
    /// Validates the input data against the codec's file format.
    /// `data` should not be the entire file in bytes, but preferably just the header.
    fn validate(&self, data: &[u8]) -> Result<(), FileReadError>;
}

/// Creates a decoder for a codec with a decoder implementation.
pub trait DecoderFactory: Codec {
    /// Opens and validates a stream of bytes, e.g. file, network stream, etc.
    /// It does not decode the entire file, responsibility is delegated to the decoder.
    async fn open(&self, source: ByteSource) -> Result<Box<dyn Decoder>, FileReadError>;
}

/// Represents a decoder which decodes a video file. Basically just a fancy iterator over the decoded frames.

#[async_trait]
pub trait Decoder {
    /// Get metadata about the decoded video.
    fn metadata(&self) -> &VideoMetadata;

    /// Decode the next frame.
    async fn next_frame(&mut self) -> Result<Option<Frame>, DecodeError>;

    /// Decode all frames and return them as a vector. Very slow!!!
    async fn all_frames(&mut self) -> Result<Vec<Frame>, DecodeError>;
}

//pub trait Encoder {}

#[derive(Debug)]
pub struct VideoMetadata {} // TODO: figure out what we need
