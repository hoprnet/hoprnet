pub mod native {
    pub fn current_time() -> std::time::SystemTime {
        std::time::SystemTime::now()
    }
}
