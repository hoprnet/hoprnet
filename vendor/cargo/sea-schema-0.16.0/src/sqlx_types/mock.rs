#![allow(dead_code)]

pub struct MySqlPool;

pub mod mysql {
    pub struct MySqlRow;
}

pub struct PgPool;

pub mod postgres {
    pub struct PgRow;
}

pub struct SqlitePool;

pub mod sqlite {
    pub struct SqliteRow;
}

pub trait Row {}

#[derive(Debug)]
pub struct Error;

#[derive(Debug)]
pub enum SqlxError {
    RowNotFound,
}
