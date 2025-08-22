use crate::grux_external_request_handlers::ExternalRequestHandler;
use crate::grux_port_manager::PortManager;
use hyper::Request;
use log::{debug, error, info, warn};
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use tokio::process::{Child, Command};
use std::time::Duration;

/// Structure to manage a persistent PHP-CGI process.
///
/// This handles:
/// - Starting php-cgi.exe with appropriate parameters for Windows
/// - Monitoring process health
/// - Automatic restart when the process dies
/// - Process lifecycle management
/// - Port management through the PortManager
pub struct PhpCgiProcess {
    process: Option<Child>,
    executable_path: String,
    restart_count: u32,
    service_id: String,
    assigned_port: Option<u16>,
    port_manager: PortManager,
}impl PhpCgiProcess {
    pub fn new(executable_path: String, service_id: String, port_manager: PortManager) -> Self {
        PhpCgiProcess {
            process: None,
            executable_path,
            restart_count: 0,
            service_id,
            assigned_port: None,
            port_manager,
        }
    }

    pub async fn start(&mut self) -> Result<(), String> {
        info!("Starting PHP-CGI process: {} for service {}", self.executable_path, self.service_id);

        // Allocate a port if we don't have one
        if self.assigned_port.is_none() {
            self.assigned_port = self.port_manager.allocate_port(self.service_id.clone()).await;
            if self.assigned_port.is_none() {
                return Err("Failed to allocate port for PHP-CGI process".to_string());
            }
        }

        let port = self.assigned_port.unwrap();
        let mut cmd = Command::new(&self.executable_path);

        if cfg!(target_os = "windows") {
            // For Windows, use php-cgi.exe in CGI mode with assigned port
            cmd.arg("-b").arg(format!("127.0.0.1:{}", port));
        }

        match cmd.spawn() {
            Ok(child) => {
                self.process = Some(child);
                self.restart_count += 1;
                info!("PHP-CGI process started successfully on port {} for service {} (restart count: {})",
                      port, self.service_id, self.restart_count);
                Ok(())
            }
            Err(e) => {
                error!("Failed to start PHP-CGI process for service {}: {}", self.service_id, e);
                // Release the port if process failed to start
                if let Some(port) = self.assigned_port {
                    self.port_manager.release_port(port).await;
                    self.assigned_port = None;
                }
                Err(format!("Failed to start PHP-CGI: {}", e))
            }
        }
    }

    pub async fn is_alive(&mut self) -> bool {
        if let Some(ref mut process) = self.process {
            match process.try_wait() {
                Ok(Some(_)) => {
                    warn!("PHP-CGI process for service {} has exited", self.service_id);
                    self.process = None;
                    false
                }
                Ok(None) => true, // Process is still running
                Err(e) => {
                    error!("Error checking PHP-CGI process status for service {}: {}", self.service_id, e);
                    self.process = None;
                    false
                }
            }
        } else {
            false
        }
    }

    async fn ensure_running(&mut self) -> Result<(), String> {
        if !self.is_alive().await {
            warn!("PHP-CGI process for service {} is not running, restarting...", self.service_id);
            // Wait a bit before restarting to avoid rapid restart loops
            tokio::time::sleep(Duration::from_millis(1000)).await;
            self.start().await?;
        }
        Ok(())
    }

    pub async fn stop(&mut self) {
        if let Some(mut process) = self.process.take() {
            info!("Stopping PHP-CGI process for service {}", self.service_id);
            if let Err(e) = process.kill().await {
                error!("Failed to kill PHP-CGI process for service {}: {}", self.service_id, e);
            }
        }

        // Release the assigned port
        if let Some(port) = self.assigned_port.take() {
            self.port_manager.release_port(port).await;
        }
    }

    pub fn get_port(&self) -> Option<u16> {
        self.assigned_port
    }
}

/// PHP handler that manages persistent PHP-CGI processes for handling PHP requests.
///
/// This implementation:
/// - Starts and maintains persistent php-cgi.exe processes on Windows
/// - Monitors process health and automatically restarts dead processes
/// - Provides worker threads that handle requests through the CGI processes
/// - Ensures thread-safe access to the PHP-CGI processes
/// - Uses the singleton port manager to assign unique ports to each process
pub struct PHPHandler {
    request_queue_tx: mpsc::Sender<String>, // Changed to String for simplicity in this example
    request_queue_rx: Arc<Mutex<mpsc::Receiver<String>>>,
    tokio_runtime: tokio::runtime::Runtime,
    request_timeout: usize,
    max_concurrent_requests: usize,
    executable: String,
    ip_and_port: String,
    extra_handler_config: Vec<(String, String)>,
    extra_environment: Vec<(String, String)>,
    php_processes: Arc<Mutex<Vec<Arc<Mutex<PhpCgiProcess>>>>>,
}

impl PHPHandler {
    pub fn new(executable: String, ip_and_port: String,  request_timeout: usize, max_concurrent_requests: usize, extra_handler_config: Vec<(String, String)>, extra_environment: Vec<(String, String)>) -> Self {
        // Initialize PHP threads
        let (request_queue_tx, rx) = mpsc::channel::<String>(1000);
        // Shared receiver
        let request_queue_rx = Arc::new(Mutex::new(rx));
        let tokio_runtime = Runtime::new().expect("Failed to create Tokio runtime for PHP handler");

        // Get the singleton port manager instance
        let port_manager = PortManager::instance();

        // Initialize PHP-CGI processes
        let mut php_processes = Vec::new();
        for i in 0..max_concurrent_requests {
            let service_id = format!("php-worker-{}", i);
            let process = Arc::new(Mutex::new(PhpCgiProcess::new(
                executable.clone(),
                service_id,
                port_manager.clone(),
            )));
            php_processes.push(process);
        }

        PHPHandler {
            request_queue_tx,
            request_queue_rx,
            tokio_runtime,
            request_timeout,
            max_concurrent_requests,
            executable,
            ip_and_port,
            extra_handler_config,
            extra_environment,
            php_processes: Arc::new(Mutex::new(php_processes)),
        }
    }

    /// Get the maximum number of concurrent requests this handler supports
    pub fn get_max_concurrent_requests(&self) -> usize {
        self.max_concurrent_requests
    }
}

impl ExternalRequestHandler for PHPHandler {
    fn start(&self) {
        // Start PHP worker threads
        let processes = self.php_processes.clone();

        for worker_id in 0..self.max_concurrent_requests {
            let rx = self.request_queue_rx.clone();
            let processes_clone = processes.clone();
            let enter_guard = self.tokio_runtime.enter();

            tokio::spawn(async move {
                info!("PHP worker thread {} started", worker_id);

                // Get the PHP process for this worker
                let process = {
                    let processes_guard = processes_clone.lock().await;
                    processes_guard[worker_id].clone()
                };

                // Start the PHP-CGI process
                {
                    let mut process_guard = process.lock().await;
                    if let Err(e) = process_guard.start().await {
                        error!("Failed to start PHP-CGI for worker {}: {}", worker_id, e);
                        return;
                    }
                }

                // Process health monitoring task
                let process_monitor = process.clone();
                tokio::spawn(async move {
                    loop {
                        {
                            let mut process_guard = process_monitor.lock().await;
                            if let Err(e) = process_guard.ensure_running().await {
                                error!("Failed to ensure PHP-CGI process is running: {}", e);
                            }
                        }
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                });

                // Main request processing loop
                loop {
                    // Lock the receiver and await one job
                    let mut rx_data = rx.lock().await;
                    match rx_data.recv().await {
                        Some(_job) => {
                            drop(rx_data); // release lock early
                            info!("PHP Worker {} got job", worker_id);

                            // Ensure process is running before handling request
                            {
                                let mut process_guard = process.lock().await;
                                if let Err(e) = process_guard.ensure_running().await {
                                    error!("Failed to ensure PHP-CGI process is running before handling request: {}", e);
                                    continue;
                                }
                            }

                            // TODO: Process the request through PHP-CGI
                            // This would involve creating CGI environment variables,
                            // sending the request to php-cgi, and handling the response
                            debug!("Processing PHP request for worker {}", worker_id);

                            // Simulate processing time
                            tokio::time::sleep(Duration::from_millis(100)).await;
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
        info!("Stopping PHP handler");
        let processes = self.php_processes.clone();
        self.tokio_runtime.spawn(async move {
            let processes_guard = processes.lock().await;
            for process in processes_guard.iter() {
                let mut process_guard = process.lock().await;
                process_guard.stop().await;
            }
        });
    }

    fn get_file_matches(&self) -> Vec<String> {
        vec!["*.php".to_string()]
    }

    fn handle_request(&self, _request: &Request<hyper::body::Incoming>) {
        // TODO: Convert request to a format that can be sent through the channel
        // For now, we'll log that a request was received
        info!("PHP request received");

        // In a complete implementation, you would:
        // 1. Extract request data (headers, body, URI, etc.)
        // 2. Create a serializable request structure
        // 3. Send it through the channel to workers
        // 4. Workers would then communicate with PHP-CGI process
    }

    fn get_handler_type(&self) -> String {
        "php".to_string()
    }
}
