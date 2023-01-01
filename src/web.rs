use std::net::SocketAddr;

use axum::{self, Router, routing::get};
use crate::prome;

pub async fn run(port:u16){
    let app = Router::new()
    .route("/ymd/nginx/metrics",get(prome::prometheus_metrics))
    .route("/",get(home));
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("listening on {}",addr);
    axum::Server::bind(&addr)
    .serve(app.into_make_service())
    .await
    .unwrap();
}

async fn home()->String{
    "数据监控".to_string()
}