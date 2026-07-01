//! Schema-derived Signal contract for Listener capture and transcription.
//!
//! This crate carries only the ordinary peer-callable wire vocabulary. The
//! `listener` daemon owns audio capture, durable writes, transcription, and
//! clipboard delivery.

#[rustfmt::skip]
pub mod schema;

pub use schema::lib::*;

impl CaptureSession {
    pub fn value(&self) -> u64 {
        *self.payload()
    }
}

impl WirePath {
    pub fn as_str(&self) -> &str {
        self.payload().as_str()
    }
}

impl SocketMode {
    pub fn as_u32(&self) -> u32 {
        *self.payload() as u32
    }
}

impl TranscriptText {
    pub fn as_str(&self) -> &str {
        self.payload().as_str()
    }
}

impl Input {
    pub fn operation_kind(&self) -> OperationKind {
        match self {
            Self::Start(_) => OperationKind::Start,
            Self::Stop(_) => OperationKind::Stop,
        }
    }
}

pub type Operation = Input;
pub type ListenerOperation = Input;
pub type ListenerReply = Output;
