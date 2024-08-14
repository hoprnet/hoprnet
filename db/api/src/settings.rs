use crate::db::HoprDb;
use async_trait::async_trait;

/// Defines DB API for accessing HOPR settings
#[async_trait]
pub trait HoprDbSettingsOperations {}

#[async_trait]
impl HoprDbSettingsOperations for HoprDb {}

#[cfg(test)]
mod tests {}
