use crate::{
    grux_configuration::get_configuration,
    grux_configuration_struct::{RequestHandler, Server},
    grux_external_request_handlers::grux_handler_php::PHPHandler,
};
use hyper::Request;
use log::debug;
use std::{collections::HashMap, sync::OnceLock};
pub mod grux_handler_php;

pub struct ExternalRequestHandlers {
    handlers: Vec<Box<dyn ExternalRequestHandler>>,
}

// A trait for external request handlers
pub trait ExternalRequestHandler: Send + Sync {
    fn start(&self);
    fn stop(&self);
    fn get_file_matches(&self) -> Vec<String>;
    fn handle_request(&self, request: &Request<hyper::body::Incoming>);
    fn get_handler_type(&self) -> String;
}

impl ExternalRequestHandlers {
    pub fn new() -> Self {
        let handlers: Vec<Box<dyn ExternalRequestHandler>> = Vec::new();
        ExternalRequestHandlers { handlers }
    }
}

// Handles external request handlers and their thread pools, such as PHP
fn start_external_request_handlers() -> Result<ExternalRequestHandlers, String> {
    // Get the config, to determine what we need
    let config = get_configuration();

    // Run through all the configured sites in configuration and determine which is actually referenced
    let servers: Vec<Server> = config.get("servers").unwrap();
    let mut handler_ids_used = HashMap::new();

    for server in servers {
        for binding in server.bindings {
            for site in binding.sites {
                for handler in &site.enabled_handlers {
                    if !handler_ids_used.contains_key(handler) {
                        handler_ids_used.insert(handler.clone(), true);
                    }
                }
            }
        }
    }

    debug!("Enabled external request handlers found in configuration: {:?}", handler_ids_used);

    // Load our implemented handlers, so they can be matched with what is configured
    let mut external_request_handlers = ExternalRequestHandlers::new();

    // Add PHP as a potential handler

    // Go through our configured handlers and load the ones we need
    let mut handler_type_to_load: HashMap<String, RequestHandler> = HashMap::new();

    let external_handlers: Vec<RequestHandler> = config.get("request_handlers").unwrap();
    for handler in external_handlers {
        if handler.is_enabled {
            // Check if the handler is in our enabled list
            if handler_ids_used.contains_key(&handler.id) {
                if !handler_type_to_load.contains_key(&handler.handler_type) {
                    handler_type_to_load.insert(handler.handler_type.clone(), handler);
                }
            }
        }
    }

    debug!("Enabled external request handler types found in configuration: {:?}", handler_type_to_load);

    // Start the handlers with the type we want
    for (handler_type, handler) in handler_type_to_load {
        match handler_type.as_str() {
            "php" => {
                let php_handler = PHPHandler::new(
                    handler.executable.clone(),
                    handler.ip_and_port.clone(),
                    handler.request_timeout,
                    handler.max_concurrent_requests,
                    handler.extra_handler_config,
                    handler.extra_environment,
                );
                php_handler.start();
                external_request_handlers.handlers.push(Box::new(php_handler));
                debug!("PHP handler started and added to external request handlers.");
            }
            _ => {
                debug!("Unknown handler type: {}", handler_type);
            }
        }
    }

    Ok(external_request_handlers)
}

// Get the request handlers
pub fn get_request_handlers() -> &'static ExternalRequestHandlers {
    static HANDLERS: OnceLock<ExternalRequestHandlers> = OnceLock::new();
    HANDLERS.get_or_init(|| start_external_request_handlers().unwrap_or_else(|e| panic!("Failed to start request handlers: {}", e)))
}
