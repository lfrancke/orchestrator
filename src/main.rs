mod config;
mod crd;
mod storage_sqlite;
mod watch;

use crate::config::OrchestratorConfig;
use crate::crd::{add_custom_resource_definition, list_custom_resource_definitions};
use crate::watch::{EventBroker, WatchEvent};

use std::sync::mpsc::Sender;
use std::sync::{mpsc, Arc};
use std::{env, thread};

use actix_web::{get, middleware::Logger, App, HttpRequest, HttpResponse, HttpServer, Responder};
use stackable_config::get_matcher;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let matcher = get_matcher(
        &OrchestratorConfig,
        "ORCHESTRATOR_CONFIG_FILE",
        env::args_os().collect(),
    )
    .expect("unexpected error occurred when parsing parameters");

    let bind_address = matcher
        .value_of(OrchestratorConfig::BIND_ADDRESS.name)
        .unwrap();
    let bind_port = matcher
        .value_of(OrchestratorConfig::BIND_PORT.name)
        .unwrap();

    // Temporarily use the embedded Actix logging
    // TODO: This should be replaced with our own structured logging
    std::env::set_var("RUST_LOG", "actix_web=debug");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    // This is used for watches.
    // The problem is that within a request we register a watch, in completely separate requests
    // e.g. a request adding a new resource this watch is triggered and this needs to "bubble down"
    // to all watchers.
    //
    // For this to work we use channels and we have two pairs of those
    // * reg_{tx, rx} are used for registering new watches
    // * evt_{tx, rx} are used to exchange the actual events
    let (reg_tx, reg_rx) = mpsc::channel::<Sender<WatchEvent>>();
    let (evt_tx, evt_rx) = mpsc::channel::<WatchEvent>();

    // This creates and runs the actual Web Server
    let server = HttpServer::new(move || {
        App::new()
            .data(storage_sqlite::get_pool().clone())
            .data(reg_tx.clone())
            .data(evt_tx.clone())
            .wrap(Logger::default())
            .service(health)
            .service(add_custom_resource_definition)
            .service(list_custom_resource_definitions)
    })
    .bind(format!("{}:{}", bind_address, bind_port))
    .expect(format!("Can't bind to {}:{}", bind_address, bind_port).as_str())
    .run();

    // Here we spawn two new threads
    // 1. Just waits for new registration events of Watchers
    // 2. Waits for new events coming in (e.g. ADDED, REMOVED, ...)
    // Both of these threads share an `EventBroker` which contains a list of all watches.
    // Because this event broker is shared across threads we wrap it in an Arc.
    // We could also use a single thread and use the `select!` macro instead.
    let event_broker = Arc::new(EventBroker::new());

    // event_broker being an Arc this only clones the reference and not the data
    let register_watch_provider = event_broker.clone();
    thread::spawn(move || {
        // Waiting for new Watch requests
        for x in reg_rx.iter() {
            register_watch_provider.register(x);
        }
    });

    let event_watch_provider = event_broker.clone();
    thread::spawn(move || {
        // Waiting for WatchEvents
        for x in evt_rx.iter() {
            event_watch_provider.new_event(x);
        }
    });

    println!("Server started successfully");
    server.await?;

    Ok(())
}

#[get("/health")]
async fn health(_: HttpRequest) -> impl Responder {
    HttpResponse::Ok().json("healthy")
}
