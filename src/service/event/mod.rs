pub mod event_service;
pub use event_service::EventService;

#[cfg(test)]
pub mod tests {
    pub mod event_test;
}