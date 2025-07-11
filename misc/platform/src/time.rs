pub mod native {
    pub fn current_time() -> std::time::SystemTime {
        std::time::SystemTime::now()
    }
}

pub use native::current_time;
