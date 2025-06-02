// Simple backend for Guestboard
// I could've done this in Python, but I'm working on a low end device
#![feature(future_join)]
mod benchmark;
mod database;
mod ip_ratelimit;
mod queries;
mod states;
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use database::AddMsgError;
use std::{future::join, sync::Arc, sync::Mutex};
use tokio_cron_scheduler::{Job, JobScheduler};
use uuid::Uuid;
type PublicAppState = web::Data<states::MainState>;
type PrivateApiKeyState = web::Data<states::ApiKeyState>;
#[get("/")]
async fn get_db(state: PublicAppState) -> impl Responder {
    let db_instance = state.db.lock().unwrap();
    HttpResponse::Ok().body(db_instance.to_json())
}

#[post("/add")]
async fn add(
    state: PublicAppState,
    request: HttpRequest,
    request_body: web::Json<queries::AddMessageQuery>,
) -> impl Responder {
    let request_body = request_body.into_inner();
    let mut db_instance = state.db.lock().unwrap();
    let conn_info = request.connection_info();
    let ip = conn_info.realip_remote_addr().unwrap();
    let db_out = db_instance.add_msg(
        ip.to_string(),
        request_body.username,
        request_body.msg,
        request_body.gif_id,
        request_body.owner_key,
    );
    match db_out {
        AddMsgError::Ratelimit => HttpResponse::Unauthorized().body(format!(
            "you need to wait ${:?} seconds before sending a new message",
            db_instance.ip_ratelimit.ratelimit_left(&ip.to_string())
        )),
        AddMsgError::UsernameOrMsgEmpty => {
            HttpResponse::BadRequest().body("username or message is empty")
        }
        AddMsgError::UsernameTooBig => {
            HttpResponse::BadRequest().body("username too big. max 20 characters.")
        }
        AddMsgError::GifIdTooBig => {
            HttpResponse::BadRequest().body("gif id too big. max 20 bytes.")
        }
        AddMsgError::MessageTooBig => {
            HttpResponse::BadRequest().body("message too big. max 64 characters.")
        }
        _ => HttpResponse::Ok().body("success"),
    }
}

#[post("/remove")]
async fn remove(
    state: PublicAppState,
    request_body: web::Json<queries::DeleteMessageQuery>,
) -> impl Responder {
    let mut db_instance = state.db.lock().unwrap();
    let request_body = request_body.into_inner();
    let db_out = db_instance.remove_msg(request_body.index, request_body.owner_key);
    if !db_out {
        return HttpResponse::Unauthorized().body("");
    }
    HttpResponse::Ok().body("body")
}

#[get("/")]
async fn get_key(state: PrivateApiKeyState) -> impl Responder {
    HttpResponse::Ok().body(state.api_key.clone())
}

#[actix_rt::main]
async fn main() -> () {
    let mut sched = JobScheduler::new().await.unwrap();
    let api_key = Uuid::new_v4().to_string();
    let database_mutex = Arc::new(Mutex::new(database::new(api_key.clone())));
    let container = web::Data::new(states::MainState {
        db: database_mutex.clone(),
    });
    let public_server = HttpServer::new(move || {
        App::new()
            .app_data(container.clone())
            .service(get_db)
            .service(add)
    })
    .bind(("0.0.0.0", 8080))
    .unwrap()
    .run();

    let api_key_container = web::Data::new(states::ApiKeyState {
        api_key: api_key.clone(),
    });
    let private_api_key_server = HttpServer::new(move || {
        App::new()
            .app_data(api_key_container.clone())
            .service(get_key)
    })
    .bind(("0.0.0.0", 8081))
    .unwrap()
    .run();

    let db_instance = database_mutex.clone();
    let housekeeping_job = sched.add(
        Job::new("every 60 seconds", move |uuid, l: JobScheduler| {
            println!("performing house keeping");
            let mut db = db_instance.lock().unwrap();
            db.save();
            db.ip_ratelimit.remove_stale();
        })
        .unwrap(),
    );

    let sched_corotiune = sched.start();
    join!(
        public_server,
        private_api_key_server,
        housekeeping_job,
        sched_corotiune
    )
    .await;
}
