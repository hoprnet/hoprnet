use thiserror::Error;

#[derive(Error, Debug)]
pub enum EventError {

}

pub type Result<T> = std::result::Result<T, EventError>;