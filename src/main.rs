mod k8s_api;
mod config;
mod crd_manager;
mod helper;
mod resource_api;
mod models;
mod storage;
mod storage_sqlite;
mod watch;

use crate::config::OrchestratorConfig;
use crate::resource_api::{create_cluster_resource, list_cluster_resources, get_cluster_resource, get_namespaced_resource, get_crd, list_crds, create_crd};
use crate::watch::{EventBroker, WatchEvent};

use std::sync::mpsc::{Sender, Receiver};
use std::sync::{mpsc, Arc};
use std::{env, thread};

use actix_web::{get, middleware::Logger, App, HttpRequest, HttpResponse, HttpServer, Responder};
use stackable_config::get_matcher;
use storage_sqlite::SqliteStorage;
use crate::k8s_api::{list_api_groups, get_api_versions, list_resource_types};


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
    std::env::set_var("RUST_LOG", "debug");
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

    let storage = SqliteStorage::new();

    // This contains all registered CRDs so we only react to the correct ones
    // TODO: Currently this is just an in-memory HashSet but we need to keep it in sync with the database
    // TODO: This is wrapped in an Arc and `.data` wraps it again which is not needed but I can't figure out another way
    //let registered_crds: Arc<RwLock<HashSet<CrdCoordinates>>> = Arc::new(RwLock::new(HashSet::new()));

    // This creates and runs the actual Web Server
    // We pass in a closure that serves as a "factory" method to build new `App` instances.
    // Actix calls this closure once per thread it starts (one per CPU by default)
    let server = HttpServer::new(move || {
        // The variables have been moved into this closure but now we need to call another method
        // which would take ownership of the data (e.g. `data(...)`.
        // Because this closure is a `Fn` (and not a `FnOnce`) and could be called multiple times
        // we cannot move _out_ these variables multiple times.
        // That means each _thing_ needs to be either `Copy`, we need to clone it or we construct a
        // new instance within this closure because that _can_ then be moved out.
        // Actix wraps everything passed to `data` in an Arc automatically which means that the data
        // does not need to be `Clone` itself.
        App::new()

            .data(storage.clone())
            .data(reg_tx.clone())
            .data(evt_tx.clone())

            //.data(registered_crds.clone())

            .wrap(Logger::default())

            .service(health)

            .service(list_api_groups)
            .service(get_api_versions)
            .service(list_resource_types)

            .service(get_crd)
            .service(list_crds)
            .service(create_crd)

            // It is important that the following routes be added _after_ the more specific ones because this catches everything that begins with /apis
            .service(list_cluster_resources)
            .service(create_cluster_resource)
            .service(get_cluster_resource)
            .service(get_namespaced_resource)
    })
        .bind(format!("{}:{}", bind_address, bind_port))
        .expect(format!("Can't bind to {}:{}", bind_address, bind_port).as_str())
        .run();


    start_event_broker(reg_rx, evt_rx);

    println!("Server started successfully"); // TODO: Proper logging
    server.await?;

    Ok(())
}

fn start_event_broker(reg_rx: Receiver<Sender<WatchEvent>>, evt_rx: Receiver<WatchEvent>) {
    // Here we spawn two new threads
    // 1. Just waits for new registration events of Watchers
    // 2. Waits for new events coming in (e.g. ADDED, REMOVED, ...)
    // Both of these threads share an `EventBroker` which contains a list of all watches.
    // Because this event broker is shared across threads we wrap it in an Arc.
    // We could probably also use a single thread and use the `select!` macro instead but the
    // documentation for that is lacking and I couldn't find any good best practices.
    let event_broker = Arc::new(EventBroker::new());

    // event_broker being an Arc this only clones the reference and not the data
    let register_watch_provider = Arc::clone(&event_broker);

    let foo = event_broker.clone();
    thread::spawn(move || {
        // Waiting for new Watch requests
        for x in reg_rx.iter() {
            register_watch_provider.register(x);
        }
    });

    let event_watch_provider = Arc::clone(&event_broker);
    thread::spawn(move || {
        // Waiting for WatchEvents
        for x in evt_rx.iter() {
            event_watch_provider.new_event(x);
        }
    });
}

#[get("/health")]
async fn health(_: HttpRequest) -> impl Responder {
    HttpResponse::Ok().json("healthy")
}
