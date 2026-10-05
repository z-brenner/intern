use std::{error::Error, fmt};

use serde::{Deserialize, Serialize};

use crate::OperationReceipt;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    FileChanged,
    SourceLocked,
    DestinationUnavailable,
    MoveVerificationFailed,
    SourceDeleteFailed,
    InvalidTransition,
    StateConflict,
    DatabaseUnavailable,
    IoError,
    InvalidData,
    ModelOutputInvalid,
    Duplicate,
    /// The model declined to answer about this document.
    ModelDeclined,
    UploaderUnverified,
    /// Reconciliation could not prove what a half-applied operation left on
    /// disk, so the item was handed to a person instead of held in `applying`.
    ReconciliationRequired,
    /// The document is encrypted and cannot be read without its password.
    PasswordProtected,
    /// The file's content is not the format its extension claims.
    UnsupportedContent,
    /// The document is beyond what Intern will read or send to the model.
    DocumentTooLarge,
    /// Text recognition is needed but its runtime files are missing.
    OcrUnavailable,
    /// The parser could not read the document, and retrying will not help.
    ExtractionFailed,
    /// Analysis failed internally on this one document.
    AnalysisFailed,
    /// The model could not finish reading this document.
    ModelFailed,
    /// The hosted model could not be used for this document.
    HostedModelUnavailable,
    /// On a canceled row: the intake watcher withdrew the document, not a
    /// person - another computer took it over, or its uploader could no
    /// longer be confirmed - so it may be handed over and run again.
    IntakeWithdrawn,
}

impl ErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FileChanged => "FILE_CHANGED",
            Self::SourceLocked => "SOURCE_LOCKED",
            Self::DestinationUnavailable => "DESTINATION_UNAVAILABLE",
            Self::MoveVerificationFailed => "MOVE_VERIFICATION_FAILED",
            Self::SourceDeleteFailed => "SOURCE_DELETE_FAILED",
            Self::InvalidTransition => "INVALID_TRANSITION",
            Self::StateConflict => "STATE_CONFLICT",
            Self::DatabaseUnavailable => "DATABASE_UNAVAILABLE",
            Self::IoError => "IO_ERROR",
            Self::InvalidData => "INVALID_DATA",
            Self::ModelOutputInvalid => "MODEL_OUTPUT_INVALID",
            Self::Duplicate => "DUPLICATE",
            Self::ModelDeclined => "MODEL_DECLINED",
            Self::UploaderUnverified => "UPLOADER_UNVERIFIED",
            Self::ReconciliationRequired => "RECONCILIATION_REQUIRED",
            Self::PasswordProtected => "PASSWORD_PROTECTED",
            Self::UnsupportedContent => "UNSUPPORTED_CONTENT",
            Self::DocumentTooLarge => "DOCUMENT_TOO_LARGE",
            Self::OcrUnavailable => "OCR_UNAVAILABLE",
            Self::ExtractionFailed => "EXTRACTION_FAILED",
            Self::AnalysisFailed => "ANALYSIS_FAILED",
            Self::ModelFailed => "MODEL_FAILED",
            Self::HostedModelUnavailable => "HOSTED_MODEL_UNAVAILABLE",
            Self::IntakeWithdrawn => "INTAKE_WITHDRAWN",
        }
    }

    pub(crate) fn from_str(value: &str) -> Option<Self> {
        Some(match value {
            "FILE_CHANGED" => Self::FileChanged,
            "SOURCE_LOCKED" => Self::SourceLocked,
            "DESTINATION_UNAVAILABLE" => Self::DestinationUnavailable,
            "MOVE_VERIFICATION_FAILED" => Self::MoveVerificationFailed,
            "SOURCE_DELETE_FAILED" => Self::SourceDeleteFailed,
            "INVALID_TRANSITION" => Self::InvalidTransition,
            "STATE_CONFLICT" => Self::StateConflict,
            "DATABASE_UNAVAILABLE" => Self::DatabaseUnavailable,
            "IO_ERROR" => Self::IoError,
            "INVALID_DATA" => Self::InvalidData,
            "MODEL_OUTPUT_INVALID" => Self::ModelOutputInvalid,
            "DUPLICATE" => Self::Duplicate,
            "MODEL_DECLINED" => Self::ModelDeclined,
            "UPLOADER_UNVERIFIED" => Self::UploaderUnverified,
            "RECONCILIATION_REQUIRED" => Self::ReconciliationRequired,
            "PASSWORD_PROTECTED" => Self::PasswordProtected,
            "UNSUPPORTED_CONTENT" => Self::UnsupportedContent,
            "DOCUMENT_TOO_LARGE" => Self::DocumentTooLarge,
            "OCR_UNAVAILABLE" => Self::OcrUnavailable,
            "EXTRACTION_FAILED" => Self::ExtractionFailed,
            "ANALYSIS_FAILED" => Self::AnalysisFailed,
            "MODEL_FAILED" => Self::ModelFailed,
            "HOSTED_MODEL_UNAVAILABLE" => Self::HostedModelUnavailable,
            "INTAKE_WITHDRAWN" => Self::IntakeWithdrawn,
            _ => return None,
        })
    }
}

#[derive(Debug)]
pub struct InternError {
    code: ErrorCode,
    message: String,
    receipt: Option<Box<OperationReceipt>>,
}

impl InternError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            receipt: None,
        }
    }

    pub const fn code(&self) -> ErrorCode {
        self.code
    }

    pub fn receipt(&self) -> Option<&OperationReceipt> {
        self.receipt.as_deref()
    }

    pub(crate) fn with_receipt(mut self, receipt: OperationReceipt) -> Self {
        self.receipt = Some(Box::new(receipt));
        self
    }
}

impl fmt::Display for InternError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code.as_str(), self.message)
    }
}

impl Error for InternError {}

impl From<rusqlite::Error> for InternError {
    fn from(error: rusqlite::Error) -> Self {
        Self::new(ErrorCode::DatabaseUnavailable, error.to_string())
    }
}

pub type InternResult<T> = Result<T, InternError>;

#[cfg(test)]
mod tests {
    use super::ErrorCode;

    /// Every code a queue row can carry must read back as itself; a code that
    /// writes but cannot be read would hide the whole row from the queue.
    #[test]
    fn every_error_code_round_trips_through_its_stored_string() {
        let all = [
            ErrorCode::FileChanged,
            ErrorCode::SourceLocked,
            ErrorCode::DestinationUnavailable,
            ErrorCode::MoveVerificationFailed,
            ErrorCode::SourceDeleteFailed,
            ErrorCode::InvalidTransition,
            ErrorCode::StateConflict,
            ErrorCode::DatabaseUnavailable,
            ErrorCode::IoError,
            ErrorCode::InvalidData,
            ErrorCode::ModelOutputInvalid,
            ErrorCode::Duplicate,
            ErrorCode::ModelDeclined,
            ErrorCode::UploaderUnverified,
            ErrorCode::ReconciliationRequired,
            ErrorCode::PasswordProtected,
            ErrorCode::UnsupportedContent,
            ErrorCode::DocumentTooLarge,
            ErrorCode::OcrUnavailable,
            ErrorCode::ExtractionFailed,
            ErrorCode::AnalysisFailed,
            ErrorCode::ModelFailed,
            ErrorCode::HostedModelUnavailable,
            ErrorCode::IntakeWithdrawn,
        ];
        for code in all {
            assert_eq!(ErrorCode::from_str(code.as_str()), Some(code));
        }
        assert_eq!(ErrorCode::from_str("NOT_A_CODE"), None);
    }
}
