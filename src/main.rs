use clap::{Parser, ValueEnum};
use env_logger;
use log::{debug, error, info};
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io::Write;

#[tokio::main]
async fn main() {
    // Initialize the logger
    env_logger::init();

    // Parse command line arguments using clap
    let args = Cli::parse();
    debug!("all input arguments: {:?}", &args);

    let url = args.get_url();
    let unit = args.get_unit();
    let value = args.get_value();
    let access_token = args.get_access_token();
    let refresh_token = args.get_refresh_token();
    let output_file = args.get_output_file();

    // Create request headers
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", access_token)).unwrap());
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

    // Create request body
    let body = JwtRequest { refresh_token, unit, value };

    // Send request
    let client = reqwest::Client::new();
    let response = client.post(&url).headers(headers).json(&body).send().await.expect("Failed to send request");

    // Handle response
    let status = response.status();
    debug!("Response Status: {:?}", status);
    let response_body = response.text().await.expect("Failed to read response body");

    if !status.is_success() {
        let error_response: ErrorResponse = serde_json::from_str(&response_body).expect("Failed to parse error response");
        error!("Error Response: {:?}", error_response);
        std::process::exit(1);
    }

    let jwt_response: JwtResponse = serde_json::from_str(&response_body).expect("Failed to parse JWT response");
    debug!("JWT Response: {:?}", jwt_response);

    let mut output_path = std::path::PathBuf::from(&output_file);
    if output_path.is_dir() {
        output_path.push("long_duration_jwt.json");
    }

    let mut file = File::create(&output_path).expect("Failed to create output file");
    file.write_all(response_body.clone().as_bytes()).expect("Failed to write to output file");

    let absolute_path = std::fs::canonicalize(&output_path).expect("Failed to get absolute path");
    info!("wrote {:?}", absolute_path);
}

#[derive(Parser)]
#[clap(
    name = "generate-long-duration-jwt",
    version = "1.0",
    author = "Author Name <kulkarni@safenow.de>",
    about = "Generates a long duration JWT"
)]
#[derive(Debug)]
pub struct Cli {
    #[clap(short = 'l', long = "url", value_name = "URL", help = "Base URL of account authentication service")]
    url: String,

    #[clap(
        short = 'v',
        long = "value",
        value_name = "TIME-VALUE",
        default_value_t = 1,
        help = "how long should the access token be valid for"
    )]
    value: i64,

    #[clap(short = 'u', long = "unit", value_name = "TIME-UNIT", value_enum, default_value_t = TimeUnit::Hours, help = "unit of time for the value")]
    unit: TimeUnit,

    #[clap(
        short = 'a',
        long = "access-token",
        value_name = "ENVIRONEMNT_VARIABLE_NAME",
        default_value = "OPERATOR_ACCESS_TOKEN",
        help = "Environment variable name for the access token, this needs to be present in the environment else the program will exit"
    )]
    access_token_env_var_name: String,

    #[clap(
        short = 'r',
        long = "refresh-token",
        value_name = "ENVIRONEMNT_VARIABLE_NAME",
        default_value = "OPERATOR_REFRESH_TOKEN",
        help = "Environment variable name for the refresh token, this needs to be present in the environment else the program will exit"
    )]
    refresh_token_env_var_name: String,

    #[clap(
        short = 'o',
        long = "output",
        value_name = "OUTPUT-FILE",
        default_value = "long_duration_jwt.json",
        help = "Output file for the success response JSON"
    )]
    output: String,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum TimeUnit {
    Seconds,
    Minutes,
    Hours,
    Days,
}

impl Cli {
    fn get_base_url(&self) -> &str {
        &self.url
    }

    fn get_unit(&self) -> String {
        format!("{:?}", self.unit).to_uppercase()
    }

    fn get_value(&self) -> i64 {
        self.value
    }

    fn get_access_token(&self) -> String {
        env::var(&self.access_token_env_var_name).expect(&format!("environment variable '{}'", self.access_token_env_var_name))
    }

    fn get_refresh_token(&self) -> String {
        env::var(&self.refresh_token_env_var_name).expect(&format!("environment variable '{}'", self.refresh_token_env_var_name))
    }

    fn get_url(&self) -> String {
        format!("{}/v1/create-new-jwt", self.get_base_url())
    }

    fn get_output_file(&self) -> &str {
        &self.output
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct JwtResponse {
    #[serde(rename = "personId")]
    person_id: String,
    #[serde(rename = "accessToken")]
    access_token: String,
    #[serde(rename = "refreshToken")]
    refresh_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ErrorResponse {
    #[serde(rename = "type")]
    error_type: String,
    class: String,
    #[serde(rename = "errorCode")]
    error_code: i32,
    #[serde(rename = "errorId")]
    error_id: String,
    #[serde(rename = "timestampMillis")]
    timestamp_millis: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct JwtRequest {
    #[serde(rename = "refreshToken")]
    refresh_token: String,
    unit: String,
    value: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_jwt_response_serialization() {
        let jwt_response = JwtResponse {
            person_id: "12345".to_string(),
            access_token: "access_token".to_string(),
            refresh_token: "refresh_token".to_string(),
        };
        let json = serde_json::to_string(&jwt_response).unwrap();
        assert!(json.contains("\"personId\":\"12345\""));
        assert!(json.contains("\"accessToken\":\"access_token\""));
        assert!(json.contains("\"refreshToken\":\"refresh_token\""));
    }

    #[test]
    fn test_jwt_response_deserialization() {
        let json = r#"{"personId":"12345","accessToken":"access_token","refreshToken":"refresh_token"}"#;
        let jwt_response: JwtResponse = serde_json::from_str(json).unwrap();
        assert_eq!(jwt_response.person_id, "12345");
        assert_eq!(jwt_response.access_token, "access_token");
        assert_eq!(jwt_response.refresh_token, "refresh_token");
    }

    #[test]
    fn test_error_response_serialization() {
        let error_response = ErrorResponse {
            error_type: "type".to_string(),
            class: "class".to_string(),
            error_code: 123,
            error_id: "error_id".to_string(),
            timestamp_millis: 1234567890,
        };
        let json = serde_json::to_string(&error_response).unwrap();
        assert!(json.contains("\"type\":\"type\""));
        assert!(json.contains("\"class\":\"class\""));
        assert!(json.contains("\"errorCode\":123"));
        assert!(json.contains("\"errorId\":\"error_id\""));
        assert!(json.contains("\"timestampMillis\":1234567890"));
    }

    #[test]
    fn test_error_response_deserialization() {
        let json = r#"{"type":"type","class":"class","errorCode":123,"errorId":"error_id","timestampMillis":1234567890}"#;
        let error_response: ErrorResponse = serde_json::from_str(json).unwrap();
        assert_eq!(error_response.error_type, "type");
        assert_eq!(error_response.class, "class");
        assert_eq!(error_response.error_code, 123);
        assert_eq!(error_response.error_id, "error_id");
        assert_eq!(error_response.timestamp_millis, 1234567890);
    }

    #[test]
    fn test_jwt_request_serialization() {
        let jwt_request = JwtRequest {
            refresh_token: "refresh_token".to_string(),
            unit: "HOURS".to_string(),
            value: 1,
        };
        let json = serde_json::to_string(&jwt_request).unwrap();
        assert!(json.contains("\"refreshToken\":\"refresh_token\""));
        assert!(json.contains("\"unit\":\"HOURS\""));
        assert!(json.contains("\"value\":1"));
    }

    #[test]
    fn test_jwt_request_deserialization() {
        let json = r#"{"refreshToken":"refresh_token","unit":"HOURS","value":1}"#;
        let jwt_request: JwtRequest = serde_json::from_str(json).unwrap();
        assert_eq!(jwt_request.refresh_token, "refresh_token");
        assert_eq!(jwt_request.unit, "HOURS");
        assert_eq!(jwt_request.value, 1);
    }
}
