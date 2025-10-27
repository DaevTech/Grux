use log::info;
use std::sync::{
    OnceLock,
    atomic::{AtomicUsize, Ordering},
};
use crate::{configuration::load_configuration::get_configuration, grux_core::async_runtime_handlers::get_async_runtime_handlers, grux_file_cache::get_file_cache};

pub struct MonitoringState {
    requests_served: AtomicUsize,
    requests_served_last: AtomicUsize,
    requests_served_per_sec: AtomicUsize,
    waiting_requests: AtomicUsize,
    server_start_time: std::time::Instant,
    file_cache_enabled: bool,
    file_cache_current_items: AtomicUsize,
    file_cache_max_items: usize,
}

impl MonitoringState {
    pub fn new() -> Self {
        let configuration = get_configuration();

        MonitoringState {
            requests_served: AtomicUsize::new(0),       // Updated from http server
            requests_served_last: AtomicUsize::new(0),       // Updated from monitoring thread
            requests_served_per_sec: AtomicUsize::new(0),
            waiting_requests: AtomicUsize::new(0),
            server_start_time: std::time::Instant::now(),
            file_cache_enabled: configuration.core.file_cache.is_enabled,
            file_cache_current_items: AtomicUsize::new(0),  // Updated from monitoring thread
            file_cache_max_items: configuration.core.file_cache.cache_item_size,
        }
    }

    // Background monitoring task.
    pub fn initialize_monitoring(&self) {
        info!("Monitoring initialized");
        tokio::spawn(Self::monitoring_task());
    }

    async fn monitoring_task() {

        let handlers = get_async_runtime_handlers();
        let http_server_handle = &handlers.http_server_handle;
        let update_interval_seconds: usize = 10;
        let update_interval = tokio::time::Duration::from_secs(update_interval_seconds as u64);

        loop {
            // Set how many active threads we have in tokio
            let metrics = http_server_handle.metrics();
            get_monitoring_state().waiting_requests.store(metrics.num_alive_tasks(), Ordering::SeqCst);

            // Calculate requests per second
            let current_requests = get_monitoring_state().get_requests_served();
            let last_requests = get_monitoring_state().requests_served_last.load(Ordering::SeqCst);
            let requests_diff = current_requests.saturating_sub(last_requests);
            let requests_per_sec: f64 = requests_diff as f64 / update_interval_seconds as f64;
            get_monitoring_state().requests_served_per_sec.store(requests_per_sec.to_bits() as usize, Ordering::SeqCst);
            get_monitoring_state().requests_served_last.store(current_requests, Ordering::SeqCst);

            // Fetch some data from file cache
            let file_cache = get_file_cache();
            get_monitoring_state().file_cache_current_items.store(file_cache.get_current_item_count(), Ordering::SeqCst);

            tokio::time::sleep(update_interval).await;
        }
    }

    pub fn increment_requests_served(&self) {
        self.requests_served.fetch_add(1, Ordering::SeqCst);
    }

    pub fn get_requests_served(&self) -> usize {
        self.requests_served.load(Ordering::SeqCst)
    }

    pub fn get_json(&self) -> serde_json::Value {
        serde_json::json!({
            "requests_served": self.get_requests_served(),
            "requests_per_sec": f64::from_bits(self.requests_served_per_sec.load(Ordering::Relaxed) as u64),
            "waiting_requests": self.waiting_requests.load(Ordering::SeqCst),
            "uptime_seconds": self.server_start_time.elapsed().as_secs(),
            "file_cache": {
                "enabled": self.file_cache_enabled,
                "current_items": self.file_cache_current_items.load(Ordering::SeqCst),
                "max_items": self.file_cache_max_items,
            }
        })
    }
}

static CURRENT_STATE_SINGLETON: OnceLock<MonitoringState> = OnceLock::new();

pub fn get_monitoring_state() -> &'static MonitoringState {
    CURRENT_STATE_SINGLETON.get_or_init(|| MonitoringState::new())
}
