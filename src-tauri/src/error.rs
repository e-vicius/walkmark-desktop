use serde::Serialize;

/// Every failure that can cross the IPC boundary.
///
/// `kind` is a stable machine-readable discriminant so the UI can react
/// (e.g. show the "grant screen recording access" flow) without string matching
/// on human-facing copy.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Screen recording permission has not been granted.")]
    PermissionDenied,

    #[error("No capture source matched the requested id. It may have been closed.")]
    SourceUnavailable,

    #[error("A recording is already in progress.")]
    AlreadyRecording,

    #[error("No recording is in progress.")]
    NotRecording,

    #[error("No API key is configured for this provider.")]
    MissingApiKey,

    #[error("The request was rejected: {0}")]
    ApiRejected(String),

    #[error("{0}")]
    LocalRuntimeUnavailable(String),

    #[error("The local model `{0}` has not been downloaded yet.")]
    LocalModelMissing(String),

    #[error("The operation was cancelled.")]
    Cancelled,

    #[error("{0}")]
    NotFound(String),

    #[error("{0}")]
    Invalid(String),

    #[error("Screen capture failed: {0}")]
    Capture(String),

    #[error("Could not read or write files: {0}")]
    Io(#[from] std::io::Error),

    #[error("Network request failed: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Could not process an image: {0}")]
    Image(#[from] image::ImageError),

    #[error("Data could not be read or written: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("{0}")]
    Other(String),
}

impl AppError {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::PermissionDenied => "permission_denied",
            Self::SourceUnavailable => "source_unavailable",
            Self::AlreadyRecording => "already_recording",
            Self::NotRecording => "not_recording",
            Self::MissingApiKey => "missing_api_key",
            Self::ApiRejected(_) => "api_rejected",
            Self::LocalRuntimeUnavailable(_) => "local_runtime_unavailable",
            Self::LocalModelMissing(_) => "local_model_missing",
            Self::Cancelled => "cancelled",
            Self::NotFound(_) => "not_found",
            Self::Invalid(_) => "invalid",
            Self::Capture(_) => "capture",
            Self::Io(_) => "io",
            Self::Network(_) => "network",
            Self::Image(_) => "image",
            Self::Serde(_) => "serde",
            Self::Other(_) => "other",
        }
    }
}

impl From<xcap::XCapError> for AppError {
    fn from(e: xcap::XCapError) -> Self {
        Self::Capture(e.to_string())
    }
}

impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut st = s.serialize_struct("AppError", 2)?;
        st.serialize_field("kind", self.kind())?;
        st.serialize_field("message", &self.to_string())?;
        st.end()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
