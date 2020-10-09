use crate::watch::{WatchEvent, WatchStream};
use actix_web::get;
use actix_web::{web, HttpResponse, Responder};
use std::sync::mpsc;
use std::sync::mpsc::Sender;

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
