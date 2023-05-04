use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Html,
    routing::{get, get_service, post},
    Form, Json, Router,
};
use minijinja::{context, Environment};
use std::{net::SocketAddr, sync::Arc};
use uuid::Uuid;

use tower_http::services::ServeDir;

use crate::dating_service::{delete_date, Date, DateContent, DatingService};

pub struct Web {
    dating: DatingService,
    env: Environment<'static>,
}

impl Web {
    pub fn new(dating: DatingService) -> Arc<Self> {
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
            .route("/date/:date_id/delete", post(delete_date))
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

        let current_date = web.dating.get_date(user_id).await.unwrap();
        let tmpl = web.env.get_template("date").unwrap();

        Html(
            tmpl.render(context!(errors => errors,date => current_date))
                .unwrap(),
        )
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
        Html("Deleted (todo: better page here)".to_string())
    }
}
