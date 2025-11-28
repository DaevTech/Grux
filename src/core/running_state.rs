use std::sync::Arc;
use log::info;

use crate::{
    external_request_handlers::external_request_handlers::ExternalRequestHandlers, file::file_cache::FileCache, logging::access_logging::AccessLogBuffer
};

pub struct RunningState {
    pub access_log_buffer: AccessLogBuffer,
    pub external_request_handlers: Arc<ExternalRequestHandlers>,
    pub http_servers: Vec<tokio::task::JoinHandle<()>>,
    pub file_cache: FileCache
}

impl RunningState {
    pub fn new() -> Self {
        let access_log_buffer = AccessLogBuffer::new();
        info!("Access log buffers initialized");

        // Start external request handlers
        let external_request_handlers = ExternalRequestHandlers::new();
        info!("External request handlers initialized");

        // Start http servers using spawn_blocking
        let http_servers = tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(async { crate::http::http_server::initialize_server().await }));

        // Start file cache
        let file_cache = FileCache::new();

        RunningState {
            access_log_buffer,
            external_request_handlers: Arc::new(external_request_handlers),
            http_servers,
            file_cache,
        }
    }

    pub fn get_external_request_handlers(&self) -> Arc<ExternalRequestHandlers> {
        self.external_request_handlers.clone()
    }

    pub fn get_access_log_buffer(&self) -> &AccessLogBuffer {
        &self.access_log_buffer
    }

    pub fn get_file_cache(&self) -> &FileCache {
        &self.file_cache
    }
}
