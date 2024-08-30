use clomonitor_apiserver::filters;

use clomonitor_core::{
    linter::{CheckSet, Report},
    score::Score,
};

use axum::{
    extract::FromRef,
    http::{
        header::{CACHE_CONTROL, CONTENT_TYPE},
        Response, StatusCode,HeaderMap,
    },
    middleware,
    routing::{get, get_service, post},
    Router,
    response::{self, IntoResponse},
    
};

use std::net::SocketAddr;
use tokio::{net::TcpListener, signal, sync::RwLock};
use tracing::{debug, info};
use anyhow::{Context, Result};
use askama_axum::Template;
use serde::{Deserialize, Serialize};
use serde_json;
use tide::http::headers::HeaderName;

#[tokio::main]
async fn main() -> Result<()> {

    let api_routes = Router::new()
        .route(
            "/report-summary",
            get(report_summary_svg),
        );
    let addr: SocketAddr = "0.0.0.0:8000".parse().expect("REASON");
    let listener = TcpListener::bind(addr).await?;

    info!("apiserver started");
    info!(%addr, "listening");
    axum::serve(listener, api_routes)
        .await?;

    Ok(())
}

pub(crate) async fn report_summary_svg() -> impl IntoResponse {
    // Get project score from database
    let score = Some(Score {
        global: 10.0,
        global_weight: 10,
        documentation: Some(80.0),
        documentation_weight: Some(10),
        license: Some(90.0),
        license_weight: Some(10),
        best_practices: Some(70.0),
        best_practices_weight: Some(10),
        security: Some(60.0),
        security_weight: Some(10),
        legal: Some(100.0),
        legal_weight: Some(10),
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
