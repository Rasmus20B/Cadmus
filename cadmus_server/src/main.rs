
use serde::{Serialize, Deserialize};
use serde_json::json;
use axum::extract::State;
use axum::Json;
use axum::{routing::get, Router};
use sqlx::prelude::FromRow;
use sqlx::types::chrono::NaiveDateTime;
use sqlx::types::{JsonValue, time::PrimitiveDateTime};
use sqlx::{query, query_as, PgPool};
use tower_http::cors::CorsLayer;
use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::task;
use tonic::transport::Server;
use tonic::{Request, Response, Status};
use tonic::include_proto;

use sqlx::{pool::Pool, postgres::{Postgres, PgPoolOptions}};

mod project_store;
use project_store::project::get_projects;

// Import your generated gRPC service traits
mod monitor_proto {
    tonic::include_proto!("monitor");
}

//use monitor_proto::greeter_server::{Greeter, GreeterServer};
//use monitor_proto::{HelloRequest, HelloReply};

// Implement the gRPC service
//#[derive(Default)]
//struct MyServiceImpl;
//
//#[tonic::async_trait]
//impl Greeter for MyServiceImpl {
//    async fn say_hello(
//        &self,
//        request: Request<HelloRequest>,
//    ) -> Result<Response<HelloReply>, Status> {
//        let req = request.into_inner();
//        println!("Received update: {:?}", req);
//
//        Ok(Response::new(HelloReply {
//            message: "Update received".into(),
//        }))
//    }
//}



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let http_addr: SocketAddr = "0.0.0.0:3000".parse().unwrap();
    let grpc_addr: SocketAddr = "0.0.0.0:50051".parse().unwrap();
    // Axum server setup

    let database_url = env::var("DATABASE_URL").unwrap();
    let database_url = database_url.as_str();

    println!("Connecting to: {}", database_url);

    let db = PgPoolOptions::new()
        .connect(database_url)
        .await
        .expect("Unable to connect to database");

    sqlx::migrate!("../migrations/").run(&db).await
        .expect("Unable to perform database migration.");

    let app = Router::new()
        .route("/", get(|| async { "Hello from Axum!" }))
        .route("/projects/retrieve_projects", get(get_projects))
        .route("/projects/create_project", get(get_projects))
        .layer(CorsLayer::permissive())
        .with_state(db);

    // Spawn the Axum server in a separate task
    let http_server = async move {
        let listener = tokio::net::TcpListener::bind(&http_addr).await.unwrap();
        axum::serve(listener, app.into_make_service())
    };

    // Set up the Tonic gRPC server
    //let grpc_service = MyServiceImpl::default();
    //let grpc_server = Server::builder()
    //    .add_service(GreeterServer::new(grpc_service))
    //    .serve(grpc_addr);

    // Run both servers concurrently
    let _ = tokio::join!(
        http_server.await,
        //tokio::spawn(grpc_server) 
    );
    Ok(())
}
