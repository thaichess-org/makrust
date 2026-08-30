// * NOTE: check unione.io as an alternative to Postmark for better pricing.
use crate::configuration::EmailSettings;
use postmark::api::Body;
use postmark::api::email::SendEmailRequest;
use postmark::reqwest::PostmarkClient;
use postmark::reqwest::PostmarkClientError;
use postmark::{Query, QueryError};

#[derive(Debug)]
pub enum EmailError {
    // TODO: think about adding a retry logic here
    // we can't reach Postmark API
    Connection(String),
    // error format returned by Postmark
    Rejected {
        status: u16,
        error_code: Option<i64>,
        message: Option<String>,
    },
    // Postmark replied 2xx but the body has issues, most likely our issue, not Postmark.
    MalformedResponse(String),
}

// Turns the postmark's generic QueryError into our own EmailError
impl From<QueryError<PostmarkClientError>> for EmailError {
    fn from(err: QueryError<PostmarkClientError>) -> Self {
        match err {
            QueryError::Client { source } => EmailError::Connection(source.to_string()),
            QueryError::Api {
                status,
                error_code,
                message,
                ..
            } => EmailError::Rejected {
                status: status.as_u16(),
                error_code,
                message,
            },
            QueryError::Json { source } => EmailError::MalformedResponse(source.to_string()),
            QueryError::Body { source } => EmailError::MalformedResponse(source.to_string()),
        }
    }
}

pub async fn send_email(
    recipient: String,
    subject: String,
    email: &EmailSettings,
) -> Result<(), EmailError> {
    let client = PostmarkClient::builder()
        .base_url(email.base_url.clone())
        .server_token(email.server_token.clone())
        .build();

    let req = SendEmailRequest::builder()
        .subject(subject)
        .from(email.sender.clone())
        .to(recipient)
        .body(Body::text(
            "This is the admin team from thaichess.org".to_string(),
        ))
        .build();

    let response = req.execute(&client).await?;

    // even when status ok is 200, Postmark can return errors
    response
        .error_for_status()
        .map_err(|rejected| EmailError::Rejected {
            status: 200,
            error_code: Some(rejected.error_code),
            message: Some(rejected.message),
        })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    // let's use wiremock to mock the email call,
    // since every real call counts against the total montly emails allowed by Postmark.
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn test_email_settings(base_url: String) -> EmailSettings {
        EmailSettings {
            base_url,
            sender: "admin@thaichess.org".to_string(),
            server_token: "test-token-not-real".to_string(),
        }
    }

    #[tokio::test]
    async fn send_email_succeeds_on_valid_postmark_response() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/email"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "To": "receiver@example.com",
                "SubmittedAt": "2014-02-17T07:25:01.4178645-05:00",
                "MessageID": "0a129aee-e1cd-480d-b08d-4f48548ff48d",
                "ErrorCode": 0,
                "Message": "OK"
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        let settings = test_email_settings(mock_server.uri());
        let result = send_email(
            "receiver@example.com".to_string(),
            "hello".to_string(),
            &settings,
        )
        .await;
        assert!(result.is_ok(), "expected Ok(()), got {result:?}");
    }

    #[tokio::test]
    async fn send_email_returns_rejected_on_non_2xx_status() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/email"))
            .respond_with(ResponseTemplate::new(422).set_body_json(serde_json::json!({
                "ErrorCode": 300,
                "Message": "Invalid email request"
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        let settings = test_email_settings(mock_server.uri());

        let result = send_email(
            "receiver@example.com".to_string(),
            "hello".to_string(),
            &settings,
        )
        .await;

        match result {
            Err(EmailError::Rejected {
                status, error_code, ..
            }) => {
                assert_eq!(status, 422);
                assert_eq!(error_code, Some(300));
            }
            other => panic!("expected EmailError::Rejected, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn send_email_returns_rejected_on_200_with_postmark_error_code() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/email"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "To": "receiver@example.com",
                "SubmittedAt": "2014-02-17T07:25:01.4178645-05:00",
                "MessageID": "0a129aee-e1cd-480d-b08d-4f48548ff48d",
                "ErrorCode": 300,
                "Message": "Inactive recipient"
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        let settings = test_email_settings(mock_server.uri());

        let result = send_email(
            "receiver@example.com".to_string(),
            "hello".to_string(),
            &settings,
        )
        .await;

        match result {
            Err(EmailError::Rejected {
                status, error_code, ..
            }) => {
                // verify it's still 200, only the body error will tell you it's an error
                assert_eq!(status, 200);
                assert_eq!(error_code, Some(300));
            }
            other => panic!("expected EmailError::Rejected, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn send_email_returns_malformed_response_on_unparseable_body() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/email"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
            .expect(1)
            .mount(&mock_server)
            .await;

        let settings = test_email_settings(mock_server.uri());

        let result = send_email(
            "receiver@example.com".to_string(),
            "hello".to_string(),
            &settings,
        )
        .await;

        assert!(
            matches!(result, Err(EmailError::MalformedResponse(_))),
            "expected EmailError::MalformedResponse, got {result:?}"
        );
    }

    #[tokio::test]
    async fn send_email_returns_connection_error_when_server_unreachable() {
        // port 1 is a privilage port.
        let settings = test_email_settings("http://127.0.0.1:1".to_string());

        let result = send_email(
            "receiver@example.com".to_string(),
            "hello".to_string(),
            &settings,
        )
        .await;

        assert!(
            matches!(result, Err(EmailError::Connection(_))),
            "expected EmailError::Connection, got {result:?}"
        );
    }
}
