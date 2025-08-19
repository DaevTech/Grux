use crate::grux_external_request_handlers::ExternalRequestHandler;
use hyper::Request;
use log::trace;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use tokio::sync::mpsc;

pub struct PHPHandler {
    request_queue_tx: mpsc::Sender<Request<hyper::body::Incoming>>,
    request_queue_rx: Arc<Mutex<mpsc::Receiver<Request<hyper::body::Incoming>>>>,
    tokio_runtime: tokio::runtime::Runtime,
    request_timeout: usize,
    max_concurrent_requests: usize,
    extra_handler_config: Vec<(String, String)>,
    extra_environment: Vec<(String, String)>,
}

impl PHPHandler {
    pub fn new(request_timeout: usize, max_concurrent_requests: usize, extra_handler_config: Vec<(String, String)>, extra_environment: Vec<(String, String)>) -> Self {
        // Initialize PHP threads
        let (request_queue_tx, rx) = mpsc::channel::<Request<hyper::body::Incoming>>(1000);
        // Shared receiver
        let request_queue_rx = Arc::new(Mutex::new(rx));
        let tokio_runtime = Runtime::new().expect("Failed to create Tokio runtime for PHP handler");
        PHPHandler {
            request_queue_tx,
            request_queue_rx,
            tokio_runtime,
            request_timeout,
            max_concurrent_requests,
            extra_handler_config,
            extra_environment,
        }
    }
}

impl ExternalRequestHandler for PHPHandler {
    fn start(&self) {
        // Start PHP worker threads
        for worker_id in 0..self.max_concurrent_requests {
            let rx = self.request_queue_rx.clone();
            let enter_guard = self.tokio_runtime.enter();

            tokio::spawn(async move {
                println!("PHP worker thread started");
                loop {
                    // Lock the receiver and await one job
                    let mut rx_data = rx.lock().await;
                    match rx_data.recv().await {
                        Some(job) => {
                            drop(rx_data); // release lock early
                            println!("PHP Worker {worker_id} got job");
                            // process job here
                        }
                        None => {
                            drop(rx_data); // release lock early
                            continue;
                        }
                    }
                }
            });

            drop(enter_guard);

        }
    }

    fn stop(&self) {
        // Stop the PHP handler
    }

    fn get_file_matches(&self) -> Vec<String> {
        vec!["*.php".to_string()]
    }

    fn handle_request(&self, request: &Request<hyper::body::Incoming>) {
        //  let mut queue = self.request_queue.lock().unwrap();
        // queue.push_back(request.clone());
    }

    fn get_handler_type(&self) -> String {
        "php".to_string()
    }
}
