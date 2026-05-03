use assert_cmd::Command;
use chrono::{DateTime, Utc};
use std::path::Path;
use tempfile::TempDir;
use wiremock::matchers::{body_json, method, path as path_matcher};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn clickup(dir: &Path, server: &MockServer) -> Command {
    let mut cmd = Command::cargo_bin("clickup").unwrap();
    cmd.current_dir(dir)
        .env("CLICKUP_API_URL", server.uri())
        .env("CLICKUP_TOKEN", "pk_test")
        .env("CLICKUP_WORKSPACE", "99")
        .env_remove("CLICKUP_GIT_DETECT")
        .env_remove("CLICKUP_TASK_ID");
    cmd
}

fn rfc3339_to_ms(input: &str) -> String {
    DateTime::parse_from_rfc3339(input)
        .map(|dt| dt.with_timezone(&Utc).timestamp_millis().to_string())
        .unwrap()
}

#[tokio::test]
async fn test_list_create_due_date() {
    let dir = TempDir::new().unwrap();
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path_matcher("/v2/folder/folder123/list"))
        .and(body_json(serde_json::json!({
            "name": "List",
            "due_date": "1735689600000",
            "due_date_time": false
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "list123",
            "name": "List",
            "due_date": "1735689600000",
            "due_date_time": false
        })))
        .expect(1)
        .mount(&server)
        .await;

    clickup(dir.path(), &server)
        .args([
            "list",
            "create",
            "--folder",
            "folder123",
            "--name",
            "List",
            "--due-date",
            "2025-01-01",
        ])
        .assert()
        .success();
}

#[tokio::test]
async fn test_list_update_due_datetime() {
    let dir = TempDir::new().unwrap();
    let server = MockServer::start().await;
    let due_date = "2025-01-01T12:34:56Z";
    let due_date_ms = rfc3339_to_ms(due_date);

    Mock::given(method("PUT"))
        .and(path_matcher("/v2/list/list123"))
        .and(body_json(serde_json::json!({
            "due_date": due_date_ms,
            "due_date_time": true
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "list123",
            "name": "List",
            "due_date": due_date_ms,
            "due_date_time": true
        })))
        .expect(1)
        .mount(&server)
        .await;

    clickup(dir.path(), &server)
        .args(["list", "update", "list123", "--due-date", due_date])
        .assert()
        .success();
}
