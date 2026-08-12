use crustalline_ipc_types::IpcError;

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("{0}")]
    Chem(#[from] molrs::ChemError),
    #[error("{0}")]
    Embed(#[from] molrs::conformer::EmbedError),
    #[error("{0}")]
    Depict(#[from] molrs::depict::DepictError),
    // Not #[from]: molrs::edit::EditError doesn't implement std::error::Error
    // (only Debug/Clone/PartialEq — see docs/molrs-api-contract.md Deviation 5,
    // f64's lack of Eq ruled out deriving more), so thiserror can't wire a
    // `source()` for it automatically. Converted manually below instead.
    #[error("edit failed: {0:?}")]
    Edit(molrs::edit::EditError),
    #[error("nothing to undo")]
    NothingToUndo,
    #[error("nothing to redo")]
    NothingToRedo,
    #[error("failed to write {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
}

impl From<molrs::edit::EditError> for CoreError {
    fn from(err: molrs::edit::EditError) -> Self {
        CoreError::Edit(err)
    }
}

impl From<CoreError> for IpcError {
    fn from(err: CoreError) -> Self {
        IpcError {
            message: err.to_string(),
        }
    }
}
