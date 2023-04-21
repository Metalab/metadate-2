use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Html,
    routing::{get, get_service, post},
    Json, Router,
};
use minijinja::{context, Environment};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};

use tower_http::services::ServeDir;

#[derive(Serialize)]
struct Date {
    id: u64,
    username: String,
    who: String,
    what: String,
    shortdesc: String,
    longdesc: String,
    contact: String,
}
#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let mut env = Environment::new();
    env.add_template("list", include_str!("templates/list.html"))
        .unwrap();
    env.add_template("date", include_str!("templates/date.html"))
        .unwrap();
    let env = Arc::new(env);

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(list))
        // `POST /users` goes to `create_user`
        .route("/users", post(create_user))
        .route("/date/:date_id", get(show_date))
        .nest_service("/public", get_service(ServeDir::new("public")))
        .with_state(env);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// basic handler that responds with a static string
async fn list(State(env): State<Arc<Environment<'static>>>) -> Html<String> {
    let tmpl = env.get_template("list").unwrap();
    Html(tmpl.render(context!(name => "John")).unwrap())
}

async fn show_date(
    State(env): State<Arc<Environment<'static>>>,
    Path(user_id): Path<u32>,
) -> Html<String> {
    let current_date = Date {
        id: 1,
        username: user_id.to_string(),
        who: user_id.to_string(),
        what: String::from("aaa"),
        shortdesc: String::from("aaa"),
        longdesc: String::from("aaa"),
        contact: String::from("aaa"),
    };
    let tmpl = env.get_template("date").unwrap();
    Html(tmpl.render(context!(date => current_date)).unwrap())
}
async fn create_user(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<User>) {
    // insert your application logic here
    let user = User {
        id: 1337,
        username: payload.username,
    };

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Json(user))
}

// the input to our `create_user` handler
#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

// the output to our `create_user` handler
#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}
