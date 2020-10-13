use crate::storage::Storage;
use crate::storage_sqlite::SqliteStorage;
use crate::watch::{WatchEvent, WatchStream};

use std::sync::mpsc;
use std::sync::mpsc::Sender;

use actix_web::{get, post, Error};
use actix_web::{web, HttpResponse, Responder};
use bytes::Buf;
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use serde::{Deserialize, Serialize};
use serde_json::Value;


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
    storage: web::Data<SqliteStorage>,
) -> impl Responder {
    // TODO: A CRD is also a watchable resource so it should also notify the event system
    let crd = crd.into_inner();
    let text_data = serde_json::to_string(&crd).unwrap();

    storage.create(&crd.metadata.name.unwrap(), &text_data.into_bytes());
    HttpResponse::Ok().body(format!("Got CRD"))
}

#[derive(Deserialize, Debug)]
pub struct CrdCoordinates {
    group: String,
    version: String,
    name: String
}


#[get("/apis/{group}/{version}/{name}")]
pub async fn handle_crd_get(
    coordinates: web::Path<CrdCoordinates>
    // TODO: We need an object here that provides access to Storage saved in Actix Data
) -> impl Responder {
    println!("{:?}", coordinates);

    HttpResponse::Ok()
        .content_type("application/json")
        .body(format!("bla"))
}


#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CustomResource {
    metadata: ObjectMeta,

    #[serde(flatten)]
    remainder: Value
}

#[post("/apis/{group}/{version}/{name}")]
pub async fn handle_crd_create(
    coordinates: web::Path<CrdCoordinates>,
    storage: web::Data<SqliteStorage>,
    bytes: web::Bytes,
    event_sender: web::Data<Sender<WatchEvent>>,
) -> Result<HttpResponse, Error> {
    let crd: CustomResource = serde_json::from_slice(bytes.bytes())?;

    let name = format!("{}/{}/{}/{}", coordinates.group, coordinates.version, coordinates.name, crd.metadata.name.clone().unwrap());

    storage.into_inner().create(&name, &bytes.to_vec());
    event_sender.send(WatchEvent::ADDED(crd));
    Ok(HttpResponse::Ok().finish())
}
