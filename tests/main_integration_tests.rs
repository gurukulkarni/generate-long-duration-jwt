#[cfg(test)]
mod tests {
    use assert_cmd::Command;
    use predicates::prelude::*;
    use serial_test::serial;
    use std::env;

    fn set_env_vars(access_token: Option<&str>, refresh_token: Option<&str>) {
        unsafe {
            if let Some(token) = access_token {
                env::set_var("OPERATOR_ACCESS_TOKEN", token);
            } else {
                env::remove_var("OPERATOR_ACCESS_TOKEN");
            }

            if let Some(token) = refresh_token {
                env::set_var("OPERATOR_REFRESH_TOKEN", token);
            } else {
                env::remove_var("OPERATOR_REFRESH_TOKEN");
            }
        }
    }

    #[test]
    #[serial]
    fn test_cli_arguments() {
        set_env_vars(Some("test_access_token"), Some("test_refresh_token"));

        let mut server = mockito::Server::new();
        let mock = server.mock("POST", "/v1/create-new-jwt")
            .with_status(200)
            .with_body(r#"{"personId":"9d129ad9-d44b-4ad2-8c21-88521ab24f05","accessToken":"jwtHeader.jwtBody.jwtSigType-jwtSig","refreshToken":"jwtHeader2.jwtBody2.jwtSigType2-jwtSig2"}"#)
            .create();

        let mut cmd = Command::cargo_bin("generate-long-duration-jwt").unwrap();
        cmd.arg("--url")
            .arg(&server.url())
            .arg("--value")
            .arg("2")
            .arg("--unit")
            .arg("days")
            .arg("--access-token")
            .arg("OPERATOR_ACCESS_TOKEN")
            .arg("--refresh-token")
            .arg("OPERATOR_REFRESH_TOKEN")
            .arg("--output")
            .arg("test_output.json");

        cmd.assert().success();
        mock.assert();
    }

    #[test]
    #[serial]
    fn test_missing_access_token() {
        set_env_vars(None, Some("test_refresh_token"));

        let mut server = mockito::Server::new();
        let mock = server.mock("POST", "/v1/create-new-jwt").expect(0).create();

        let mut cmd = Command::cargo_bin("generate-long-duration-jwt").unwrap();
        cmd.arg("--url")
            .arg(&server.url())
            .arg("--value")
            .arg("2")
            .arg("--unit")
            .arg("days")
            .arg("--access-token")
            .arg("OPERATOR_ACCESS_TOKEN")
            .arg("--refresh-token")
            .arg("OPERATOR_REFRESH_TOKEN")
            .arg("--output")
            .arg("test_output.json");

        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("environment variable 'OPERATOR_ACCESS_TOKEN'"));
        mock.assert();
    }

    #[test]
    #[serial]
    fn test_missing_refresh_token() {
        set_env_vars(Some("test_access_token"), None);

        let mut server = mockito::Server::new();
        let mock = server.mock("POST", "/v1/create-new-jwt").expect(0).create();

        let mut cmd = Command::cargo_bin("generate-long-duration-jwt").unwrap();
        cmd.arg("--url")
            .arg(&server.url())
            .arg("--value")
            .arg("2")
            .arg("--unit")
            .arg("days")
            .arg("--access-token")
            .arg("OPERATOR_ACCESS_TOKEN")
            .arg("--refresh-token")
            .arg("OPERATOR_REFRESH_TOKEN")
            .arg("--output")
            .arg("test_output.json");

        cmd.assert().failure().stderr(predicate::str::contains("OPERATOR_REFRESH_TOKEN"));
        mock.assert();
    }

    #[test]
    #[serial]
    fn test_successful_request() {
        set_env_vars(Some("test_access_token"), Some("test_refresh_token"));

        let mut server = mockito::Server::new();
        let mock = server.mock("POST", "/v1/create-new-jwt")
            .with_status(200)
            .with_body(r#"{"personId":"9d129ad9-d44b-4ad2-8c21-88521ab24f05","accessToken":"jwtHeader.jwtBody.jwtSigType-jwtSig","refreshToken":"jwtHeader2.jwtBody2.jwtSigType2-jwtSig2"}"#)
            .create();

        let mut cmd = Command::cargo_bin("generate-long-duration-jwt").unwrap();
        cmd.arg("--url")
            .arg(&server.url())
            .arg("--value")
            .arg("2")
            .arg("--unit")
            .arg("days")
            .arg("--access-token")
            .arg("OPERATOR_ACCESS_TOKEN")
            .arg("--refresh-token")
            .arg("OPERATOR_REFRESH_TOKEN")
            .arg("--output")
            .arg("test_output.json");

        cmd.assert().success().stdout(predicate::str::contains("Output written to"));
        mock.assert();
    }
}
