use thiserror::Error;

#[derive(Error, Debug)]
pub enum AxlError {
    #[error("could not find any sessions to choose from")]
    NoSessionsFound,

    #[error("no project was selected from the list, cannot proceed")]
    NoProjectSelected,

    #[error("provided config path does not exist")]
    ConfigPathDoesNotExist,

    #[error("project path {0} does not exist")]
    ProjectPathDoesNotExist(String),
}
