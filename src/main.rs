use dating_service::DatingService;
use web::Web;

mod dating_service;
mod web;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();
    let dating = DatingService::new();
    let web = Web::new(dating);
    web.start().await;
}
