use assert_cmd::Command;
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

#[tokio::test]
async fn test_goal_update_due_date() {
    let dir = TempDir::new().unwrap();
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path_matcher("/v2/goal/goal123"))
        .and(body_json(serde_json::json!({
            "due_date": "1735689600000"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "goal": {
                "id": "goal123",
                "name": "Goal",
                "due_date": "1735689600000"
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    clickup(dir.path(), &server)
        .args(["goal", "update", "goal123", "--due-date", "2025-01-01"])
        .assert()
        .success();
}
