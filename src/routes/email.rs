use crate::domain::{EmailError, send_email};
use crate::routes::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;

pub async fn send(State(state): State<AppState>, Path(recipient): Path<String>) -> StatusCode {
    let result = send_email(
        recipient,
        "testing thaichess emails".to_string(),
        &state.email,
    )
    .await;

    match result {
        Ok(()) => StatusCode::OK,
        Err(EmailError::Connection(_)) | Err(EmailError::Rejected { .. }) => {
            StatusCode::BAD_GATEWAY
        }
        Err(EmailError::MalformedResponse(_)) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
