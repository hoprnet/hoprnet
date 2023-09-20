use thiserror::Error;

#[derive(Debug, Error)]
pub enum StrategyError {}

pub type Result<T> = std::result::Result<T, StrategyError>;

#[cfg(feature = "wasm")]
impl From<StrategyError> for wasm_bindgen::JsValue {
    fn from(value: StrategyError) -> Self {
        value.to_string().into()
    }
}
