use crustalline_ipc_types::IpcError;

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("{0}")]
    Chem(#[from] molrs::ChemError),
    #[error("{0}")]
    Embed(#[from] molrs::conformer::EmbedError),
    #[error("{0}")]
    Depict(#[from] molrs::depict::DepictError),
    #[error("failed to write {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
}

impl From<CoreError> for IpcError {
    fn from(err: CoreError) -> Self {
        IpcError {
            message: err.to_string(),
        }
    }
}
