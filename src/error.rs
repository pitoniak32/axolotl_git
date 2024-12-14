use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("could not find any sessions to choose from")]
    NoSessionsFound,

    #[error("could not create a session")]
    CouldNotCreateSession,

    #[error("provided config path does not exist")]
    ConfigPathDoesNotExist,

    #[error("path {0} does not exist")]
    PathDoesNotExist(String),
}
