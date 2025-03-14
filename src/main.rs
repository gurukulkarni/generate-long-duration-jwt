use clap::{Parser, ValueEnum};
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io::Write;

#[derive(Parser)]
#[clap(name = "generate-long-duration-jwt", version = "1.0", author = "Author Name <kulkarni@safenow.de>", about = "Generates a long duration JWT")]
pub struct Cli {
    #[clap(short = 'l', long = "url", value_name = "URL", help = "Base URL of account authentication service")]
    url: String,

    #[clap(short = 'v', long = "value", value_name = "TIME-VALUE", default_value_t = 1, help = "how long should the access token be valid for")]
    value: i64,

    #[clap(short = 'u', long = "unit", value_name = "TIME-UNIT", value_enum, default_value_t = TimeUnit::Hours, help = "unit of time for the value")]
    unit: TimeUnit,

    #[clap(short = 'a', long = "access-token", value_name = "ENVIRONEMNT_VARIABLE_NAME", default_value = "OPERATOR_ACCESS_TOKEN", help = "Environment variable name for the access token, this needs to be present in the environment else the program will exit")]
    access_token_env_var_name: String,

    #[clap(short = 'r', long = "refresh-token", value_name = "ENVIRONEMNT_VARIABLE_NAME", default_value = "OPERATOR_REFRESH_TOKEN", help = "Environment variable name for the refresh token, this needs to be present in the environment else the program will exit")]
    refresh_token_env_var_name: String,

    #[clap(short = 'o', long = "output", value_name = "OUTPUT-FILE", default_value = "long_duration_jwt.json", help = "Output file for the success response JSON")]
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

#[tokio::main]
async fn main() {
    // Parse command line arguments using clap
    let args = Cli::parse();

    let url = args.get_url();
    let unit = args.get_unit();
    let value = args.get_value();
    let access_token = args.get_access_token();
    let refresh_token = args.get_refresh_token();
    let output_file = args.output;

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
    let response_body = response.text().await.expect("Failed to read response body");

    if status.is_success() {
        let jwt_response: JwtResponse = serde_json::from_str(&response_body).expect("Failed to parse JWT response");
        println!("JWT Response: {:?}", jwt_response);

        // Write the success response JSON to the output file
        let mut file = File::create(&output_file).expect("Failed to create output file");
        file.write_all(response_body.as_bytes()).expect("Failed to write to output file");

        // Log the absolute file path
        let absolute_path = std::fs::canonicalize(&output_file).expect("Failed to get absolute path");
        println!("Output written to: {:?}", absolute_path);
    } else {
        let error_response: ErrorResponse = serde_json::from_str(&response_body).expect("Failed to parse error response");
        eprintln!("Error Response: {:?}", error_response);
        std::process::exit(1); // Exit with error code 1
    }
}
