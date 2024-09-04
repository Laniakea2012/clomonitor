use clomonitor_apiserver::filters;

use clomonitor_core::score::Score;

use axum::{
    extract::Json,
    http::{
        header::CACHE_CONTROL, StatusCode,
    },
    routing::{get, post},
    Router,
    response::IntoResponse,
    
};

use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;
use anyhow::Result;
use askama_axum::Template;

use auth::{protected, authorize, Claims};

mod auth;

#[tokio::main]
async fn main() -> Result<()> {

    let api_routes = Router::new()
        .route("/report-summary", post(report_summary_svg))
        .route("/protected", get(protected))
        .route("/login", post(authorize));

    let addr: SocketAddr = "0.0.0.0:8000".parse().expect("Invalid address");
    let listener = TcpListener::bind(addr).await?;

    info!("apiserver started");
    info!(%addr, "listening");
    axum::serve(listener, api_routes)
        .await?;

    Ok(())
}

pub(crate) async fn report_summary_svg(claims: Claims, Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    // Get project score from request
    let global = body.get("global").and_then(|v| v.as_f64()).unwrap_or_default();
    let legal = body.get("legal").and_then(|v| v.as_f64());
    let technology_ecosystem = body.get("technology_ecosystem").and_then(|v| v.as_f64());
    let lifecycle = body.get("lifecycle").and_then(|v| v.as_f64());
    let security = body.get("security").and_then(|v| v.as_f64());

    let score = Some(Score {
        global: global,
        legal: legal,
        technology_ecosystem: technology_ecosystem,
        lifecycle: lifecycle,
        security: security,
        ..Score::default()
    });

    // Render report summary SVG and return it if the score was found
    match score {
        Some(score) => {
            let headers = [(CACHE_CONTROL, format!("max-age=300"))];
            let theme = Some("light".to_string());
            Ok((headers, ReportSummaryTemplate::new(score, theme)))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Template for the report summary SVG image.
#[derive(Debug, Clone, Template)]
#[template(path = "report-summary.svg")]
pub(crate) struct ReportSummaryTemplate {
    pub score: Score,
    pub theme: String,
}

impl ReportSummaryTemplate {
    fn new(score: Score, theme: Option<String>) -> Self {
        let theme = theme.unwrap_or_else(|| "light".to_string());
        Self { score, theme }
    }
}
