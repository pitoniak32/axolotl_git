use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("could not find any sessions to choose from")]
    NoSessionsFound,

    #[error("could not create a session")]
    CouldNotCreateSession,

    #[error("no matching project was selected from the list, cannot proceed")]
    NoProjectSelected,

    #[error("provided config path does not exist")]
    ConfigPathDoesNotExist,

    #[error("project path {0} does not exist")]
    ProjectPathDoesNotExist(String),

    #[error("provided remote failed to be parsed")]
    ProjectRemoteNotParsable,
}
