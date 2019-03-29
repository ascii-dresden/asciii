use failure::Fail;

#[derive(Fail, Debug)]
pub enum ActionError {

    #[fail(display = "unexpected response from service")]
    ActionError,

    #[fail(display = "Adding Failed")]
    AddingFailed,

    #[fail(display = "Nothing found for {:?}", _0)]
    NothingFound(Vec<String>),
}
