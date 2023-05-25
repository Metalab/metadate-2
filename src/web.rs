use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Html,
    routing::{get, get_service, post},
    Form, Router,
};
use minijinja::{context, Environment};
use std::{net::SocketAddr, sync::Arc};
use uuid::Uuid;

use tower_http::services::ServeDir;

use crate::dating_service::{DateContent, DatingService, DeleteRequest};

pub struct Web {
    dating: Arc<DatingService>,
    env: Environment<'static>,
}

impl Web {
    pub fn new(dating: Arc<DatingService>) -> Arc<Self> {
        let mut env = Environment::new();
        env.add_template("list", include_str!("templates/list.html"))
            .unwrap();
        env.add_template("date", include_str!("templates/date.html"))
            .unwrap();
        env.add_template("input", include_str!("templates/input.html"))
            .unwrap();
        Arc::new(Self { dating, env })
    }
    pub async fn start(self: &Arc<Web>) {
        // build our application with a route
        let app = Router::new()
            // `GET /` goes to `root`
            .route("/", get(Self::list))
            .route("/newdate", get(Self::input))
            .route("/", post(Self::add_date))
            .route("/date/:date_id", get(Self::show_date))
            .route("/date/:date_id", post(Self::delete_date))
            .nest_service("/public", get_service(ServeDir::new("public")))
            .with_state(self.clone());

        // run our app with hyper
        // `axum::Server` is a re-export of `hyper::Server`
        let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
        tracing::debug!("listening on {}", addr);
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    }

    pub async fn show_date(
        State(web): State<Arc<Self>>,
        Path(user_id): Path<String>,
    ) -> Html<String> {
        let user_id = match Uuid::parse_str(&user_id) {
            Err(_) => return Html(StatusCode::BAD_REQUEST.to_string()),
            Ok(user_id) => user_id,
        };
        let errors: Vec<String> = Vec::new();

        match web.dating.get_date(user_id).await {
            Ok(current_date) => {
                let tmpl = web.env.get_template("date").unwrap();

                Html(
                    tmpl.render(context!(errors => errors,date => current_date))
                        .unwrap(),
                )
            }
            Err(_) => return Html("Date not found :(".to_string()),
        }
    }

    pub async fn delete_date(
        State(web): State<Arc<Self>>,
        Path(user_id): Path<String>,
        Form(delete_data): Form<DeleteRequest>,
    ) -> Html<String> {
        let uuid = match Uuid::parse_str(&user_id) {
            Err(_) => return Html("Date not found :(".to_string()),
            Ok(uuid) => uuid,
        };

        match web.dating.delete(uuid, delete_data.password).await {
            Err(error) => Html(error),
            Ok(_) => Html("Deleted (todo: better page here)".to_string()),
        }
    }

    pub async fn input(State(web): State<Arc<Web>>) -> Html<String> {
        let errors: Vec<String> = Vec::new();
        let tmpl = web.env.get_template("input").unwrap();
        let date_content = DateContent::new();
        Html(
            tmpl.render(context!( errors => errors,date => date_content))
                .unwrap(),
        )
    }

    pub async fn list(State(web): State<Arc<Web>>) -> Html<String> {
        let tmpl = web.env.get_template("list").unwrap();
        Html(
            tmpl.render(context!(dates => web.dating.list().await))
                .unwrap(),
        )
    }

    pub async fn add_date(
        State(web): State<Arc<Self>>,
        Form(new_date): Form<DateContent>,
    ) -> Html<String> {
        web.dating.add_date(new_date).await.unwrap();
        Html("Added (todo: better page here)".to_string())
    }
}
