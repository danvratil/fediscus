use aide::axum::{routing::get, ApiRouter};
use aide::openapi::{Info, OpenApi};
use aide::swagger::Swagger;

use anyhow::Error;
use axum::Extension;
use fediscus_common::http_server::HttpServerConfig;
use tokio::net::TcpListener;

mod handlers;

pub struct HttpServer {
    config: HttpServerConfig,
}

impl HttpServer {
    pub fn new(config: HttpServerConfig) -> Self {
        HttpServer { config }
    }

    pub async fn run(self) -> Result<(), Error> {
        let listener = TcpListener::bind(self.config.listen).await?;

        let router = ApiRouter::new()
            .route("/api", Swagger::new("/api.json").axum_route())
            .api_route("/api/v1/comments", get(handlers::get_comments))
            //.api_route("/api/v1/comment_counts", post(handlers::comment_count))
            .route("/api.json", get(handlers::serve_api));

        let mut api = OpenApi {
            info: Info {
                description: Some("Fediscus API".to_string()),
                ..Info::default()
            },
            ..OpenApi::default()
        };

        Ok(axum::serve(
            listener,
            router
                .finish_api(&mut api)
                .layer(Extension(api))
                .into_make_service(),
        )
        .await?)
    }
}
