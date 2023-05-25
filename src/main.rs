use std::{sync::Arc, time::Duration};

use dating_service::DatingService;
use tokio::time;
use web::Web;

mod dating_service;
mod web;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();
    let dating = Arc::new(DatingService::new());

    let web = Web::new(dating.clone());

    let inner_dating = dating.clone();
    tokio::spawn(async move {
        let mut iv = time::interval(Duration::new(5, 0));
        loop {
            iv.tick().await;
            inner_dating.clean_old_dates().await;
        }
    });

    web.start().await;
}
