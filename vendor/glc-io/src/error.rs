use thiserror::Error;

#[derive(Debug, Error)]
pub enum IoError {
    #[error("unsupported file format: {0}")]
    UnsupportedFormat(String),

    #[error("file not found: {0}")]
    FileNotFound(String),

    #[error("parse error: {0}")]
    ParseError(String),

    #[error("OBJ load error: {0}")]
    ObjError(String),

    #[error("STL load error: {0}")]
    StlError(String),

    #[error("OFF load error: {0}")]
    OffError(String),

    #[error("3DXML load error: {0}")]
    ThreeDxmlError(String),

    #[error("album XML error: {0}")]
    AlbumError(String),

    #[error("export error: {0}")]
    ExportError(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("XML error: {0}")]
    Xml(#[from] quick_xml::Error),
}

pub type Result<T> = std::result::Result<T, IoError>;
