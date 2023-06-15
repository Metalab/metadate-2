use askama_axum::{IntoResponse, Response};
use axum::{
    extract::{Path, State},
    response::{Html, Redirect},
    routing::{get, get_service, post},
    Form, Router,
};
use minijinja::{context, Environment};
use std::{net::SocketAddr, sync::Arc};

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
        env.add_template("kiosk", include_str!("templates/kiosk.html"))
            .unwrap();
        Arc::new(Self { dating, env })
    }
    pub async fn start(self: &Arc<Web>) {
        // build our application with a route
        let app = Router::new()
            // `GET /` goes to `root`
            .route("/", get(Self::list))
            .route("/newdate", get(Self::input))
            .route("/kiosk", get(Self::show_kiosk))
            .route("/kiosk/:date_id", get(Self::show_kiosk_entry))
            .route("/", post(Self::add_date))
            .route("/date/:date_id", get(Self::show_date))
            .route("/date/:date_id", post(Self::edit_date))
            .nest_service("/public", get_service(ServeDir::new("public")))
            .with_state(self.clone());

        // run our app with hyper
        // `axum::Server` is a re-export of `hyper::Server`
        let addr = SocketAddr::from(([127, 0, 0, 1], 5000));
        tracing::debug!("listening on {}", addr);
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    }

    pub async fn show_kiosk(State(web): State<Arc<Self>>) -> Html<String> {
        let tmpl = web.env.get_template("kiosk").unwrap();

        Html(tmpl.render(context!()).unwrap())
    }

    pub async fn show_kiosk_entry(
        State(web): State<Arc<Self>>,
        Path(user_id): Path<String>,
    ) -> Response {
        let date = web.dating.get_next_date_of(Some(&user_id)).await;
        Html(serde_json::to_string(&date).unwrap()).into_response()
    }

    pub async fn show_date(
        State(web): State<Arc<Self>>,
        Path(user_id): Path<String>,
    ) -> Html<String> {
        let errors: Vec<String> = Vec::new();

        match web.dating.get_date(&user_id).await {
            Ok(current_date) => {
                let tmpl = web.env.get_template("date").unwrap();

                Html(
                    tmpl.render(context!(errors => errors,date => current_date))
                        .unwrap(),
                )
            }
            Err(_) => Html("Date not found :(".to_string()),
        }
    }

    pub async fn edit_date(
        State(web): State<Arc<Self>>,
        Path(user_id): Path<String>,
        Form(update_data): Form<DeleteRequest>,
    ) -> Html<String> {
        let mut errors: Vec<String> = Vec::new();
        if update_data.action_type.is_none() {
            errors.push("No action specified".to_string());
        } else {
            let time = update_data.action_type.unwrap();

            let reset_response = web
                .dating
                .reset_timeout(&user_id, update_data.password, time)
                .await;

            match reset_response {
                Err(error) => errors.push(error),
                _ => errors.push("Extended!".to_string()),
            }
        }

        match web.dating.get_date(&user_id).await {
            Ok(current_date) => {
                let tmpl = web.env.get_template("date").unwrap();

                Html(
                    tmpl.render(context!(errors => errors,date => current_date))
                        .unwrap(),
                )
            }
            Err(_) => Html("Date not found :(".to_string()),
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
    ) -> Response {
        let mut errors: Vec<String> = Vec::new();
        let tmpl = web.env.get_template("input").unwrap();
        match new_date.action_type {
            None => errors.push("No action specified".to_string()),
            Some(_) => match web.dating.add_date(new_date).await {
                Ok(uuid) => {
                    return Redirect::to(format!("/date/{}", uuid).as_str()).into_response()
                }
                Err(error_data) => {
                    return Html(
                        tmpl.render(
                            context!(errors => error_data.errors,date => error_data.content),
                        )
                        .unwrap(),
                    )
                    .into_response()
                }
            },
        }

        Html(
            tmpl.render(context!(errors => errors,date => new_date))
                .unwrap(),
        )
        .into_response()
    }
}
