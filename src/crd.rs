use crate::watch::{WatchEvent, WatchStream};

use std::sync::mpsc;
use std::sync::mpsc::Sender;

use actix_web::{get, post};
use actix_web::{web, HttpResponse, Responder};
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;

/// This lists CustomResourceDefinitions registered in the server
/// The request will currently always be a "watch" on new resources
/// That means the result will be a never-ending HTTP response with newline-separated JSON objects
#[get("/apis/apiextensions.k8s.io/v1/customresourcedefinitions")]
pub async fn list_custom_resource_definitions(
    watch_register: web::Data<Sender<Sender<WatchEvent>>>,
) -> impl Responder {
    // We're creating a new channel pair, the _sending_ end of which we send to the EventBroker
    // The receiving end will be given to the WatchStream which
    let (tx, rx) = mpsc::channel();
    let res = watch_register.send(tx);

    let body = WatchStream::new(rx);

    HttpResponse::Ok()
        .content_type("application/json")
        .streaming(body)
}

#[post("/apis/apiextensions.k8s.io/v1/customresourcedefinitions")]
pub async fn add_custom_resource_definition(
    crd: web::Json<CustomResourceDefinition>,
    db: web::Data<Pool<SqliteConnectionManager>>,
    event_sender: web::Data<Sender<WatchEvent>>,
) -> impl Responder {
    //println!("CRD: {:?}", crd);

    let res = event_sender.send(WatchEvent::ADDED);

    let crd = &crd.into_inner();

    let text_data = serde_json::to_string(&crd).unwrap();

    let conn = &db.get().unwrap();
    let res = conn.execute("INSERT INTO data(id, json) VALUES (?1, ?2) ON CONFLICT(id) DO UPDATE SET json=excluded.json", params![crd.metadata.name.clone().unwrap(), text_data]);
    //println!("{:?}", result);
    HttpResponse::Ok().body(format!("Got CRD"))
}
