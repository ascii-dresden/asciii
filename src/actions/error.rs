use thiserror::Error;

#[derive(Error, Debug)]
pub enum ActionError {

    #[error("unexpected response from service")]
    ActionError,

    #[error("Adding Failed")]
    AddingFailed,

    #[error("Nothing found for {:?}", _0)]
    NothingFound(Vec<String>),
}
