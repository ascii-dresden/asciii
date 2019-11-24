use thiserror::Error;

#[derive(Debug, Error)]
pub enum ActionError {

    #[error("unexpected response from service")]
    ActionError,

    #[error("Adding Failed")]
    AddingFailed,

    #[error("Nothing found for {:?}", _0)]
    NothingFound(Vec<String>),
}
