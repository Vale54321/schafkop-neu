use octocrab::{Octocrab, Result};

use http::{header, HeaderValue};

use axum::{
    routing::{get},
    http::StatusCode,
    Json, Router,
};
use axum::extract::Path;
use axum::response::IntoResponse;
use axum::response::Response;
use http::HeaderMap;

use tower_http::{
    services::{ServeDir},
};
use tower_http::set_header::SetResponseHeaderLayer;
use tower::ServiceBuilder;

use askama::Template;

#[tokio::main]
async fn main() -> Result<()> {
    let static_service = ServiceBuilder::new()
    .layer(SetResponseHeaderLayer::if_not_present(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-cache, no-store, must-revalidate"),
    ))
    .layer(SetResponseHeaderLayer::if_not_present(
        header::PRAGMA,
        HeaderValue::from_static("no-cache"),
    ))
    .layer(SetResponseHeaderLayer::if_not_present(
        header::EXPIRES,
        HeaderValue::from_static("0"),
    ))
    .service(ServeDir::new("static"));

    // build our application with a route
    let app: Router = Router::new()
        .route("/api", get(root))
        .route("/", get(index))
        .route("/api/firmware/{tag}", get(download_firmware))
        .nest_service("/static", static_service);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

async fn root() -> Result<Json<Vec<String>>, StatusCode> {
    let gh = Octocrab::builder().build().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let owner = "Vale54321";
    let repo = "schafkop-neu";

    let mut tags: Vec<String> = vec![]; 

    let page = gh.repos(owner, repo).releases().list().per_page(100).send().await.map_err(|_| StatusCode::BAD_GATEWAY)?;

    for release in &page.items {
        tags.push(release.tag_name.clone());
    }

    Ok(Json(tags))
}

async fn index() -> impl axum::response::IntoResponse {
    let index = Index {
        title: "Schafkopf OS",
        message: "Welcome to Schafkopf OS!",
        version: app_version(),
    };
    axum::response::Html(index.render().unwrap())
}

pub fn app_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[derive(Template)]
#[template(path = "index.html")]
struct Index<'a> {
    title: &'a str,
    message: &'a str,
    version: &'a str,
}

async fn download_firmware(
    Path(tag): Path<String>,
) -> std::result::Result<Response, StatusCode> {
    let gh = Octocrab::builder()
        .build()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let owner = "Vale54321";
    let repo = "schafkop-neu";

    // Get the release for the provided tag
    let release = gh
        .repos(owner, repo)
        .releases()
        .get_by_tag(&tag)
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    // List assets for the release
    let assets = gh
        .repos(owner, repo)
        .releases()
        .assets(release.id.0)
        .per_page(100)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    // Find the "firmware.uf2" asset
    let Some(asset) = assets.items.into_iter().find(|a| a.name == "firmware.uf2") else {
        return Err(StatusCode::NOT_FOUND);
    };

    let location = asset.browser_download_url.to_string();

    let mut headers = HeaderMap::new();
    headers.insert(
        header::LOCATION,
        HeaderValue::from_str(&location).map_err(|_| StatusCode::BAD_GATEWAY)?,
    );

    // 302 Found with Location header
    let resp = (StatusCode::FOUND, headers).into_response();
    Ok(resp)
}