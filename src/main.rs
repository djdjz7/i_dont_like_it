use core::panic;
use std::{
    fmt::Display,
    io::{stdin, stdout, Write},
    process,
};

use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut username = String::new();
    let mut password = String::new();
    println!("username:");
    stdin().read_line(&mut username).unwrap();
    println!("password:");
    stdin().read_line(&mut password).unwrap();
    let username = username.trim();
    let password = password.trim();

    let client = reqwest::Client::new();
    let login_response = match login_async(username, password, &client).await {
        Ok(resp) => resp,
        Err(e) => {
            println!("Login failed: {}", e);
            process::exit(1);
        }
    };

    let mut page_id = String::new();
    println!("pageId: (Hint: 143991)");
    stdin().read_line(&mut page_id).unwrap();

    println!("interval(ms): (default: 200)");

    let interval = input_and_parse(Some(200u64));

    println!("mode: (0: add(default), 1: remove)");
    let mut mode = input_and_parse(Some(0u8));

    if mode >= 2 {
        println!("Invalid mode, using 0");
        mode = 0;
    }

    let mut lock = stdout().lock();
    let mut count = 0;

    let mut access_token = login_response.access_token.clone();
    let mut refresh_token = login_response.refresh_token.clone();

    if mode == 0 {
        loop {
            count += 1;
            let resp = client
                .post(format!(
                    "http://sxz.api6.zykj.org/api/services/app/appWebSite/AddStarAsync?pageId={}",
                    &page_id
                ))
                .bearer_auth(&access_token)
                .send()
                .await?;
            match resp.status() {
                reqwest::StatusCode::OK => {
                    writeln!(lock, "Add star success (x{count})").unwrap();
                }
                reqwest::StatusCode::UNAUTHORIZED => {
                    println!("Login expired, refreshing token...");
                    match refresh_token_async(&access_token, &refresh_token, &client).await {
                        Ok(resp) => {
                            count -= 1;
                            access_token = resp.access_token.clone();
                            refresh_token = resp.refresh_token.clone();
                            continue;
                        }
                        Err(e) => {
                            panic!("error refreshing token: {}", e);
                        }
                    }
                }
                e => {
                    panic!("unknown error: {}", e);
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(interval)).await;
        }
    } else {
        loop {
            count += 1;
            let resp = client
                .delete(format!(
                "http://sxz.api6.zykj.org/api/services/app/appWebSite/RemoveStarAsync?pageId={}",
                &page_id
            ))
                .bearer_auth(&login_response.access_token)
                .send()
                .await?;
            match resp.status() {
                reqwest::StatusCode::OK => {
                    writeln!(lock, "Remove star success (x{count})").unwrap();
                }
                reqwest::StatusCode::UNAUTHORIZED => {
                    println!("Login expired, refreshing token...");
                    match refresh_token_async(&access_token, &refresh_token, &client).await {
                        Ok(resp) => {
                            count -= 1;
                            access_token = resp.access_token.clone();
                            refresh_token = resp.refresh_token.clone();
                            continue;
                        }
                        Err(e) => {
                            panic!("error refreshing token: {}", e);
                        }
                    }
                }
                e => {
                    panic!("unknown error: {}", e);
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(interval)).await;
        }
    }
}

async fn login_async<'a>(
    username: &'a str,
    password: &'a str,
    client: &reqwest::Client,
) -> Result<LoginResponse, Box<dyn std::error::Error>> {
    let login_info = LoginInfo {
        user_name: username,
        password: password,
        client_type: 1,
    };
    let resp = client
        .post("http://sxz.api6.zykj.org/api/TokenAuth/Login")
        .json(&login_info)
        .send()
        .await?
        .json::<CommonResponse<LoginResponse>>()
        .await?
        .result;
    Ok(resp)
}

async fn refresh_token_async(
    access_token: &str,
    refresh_token: &str,
    client: &reqwest::Client,
) -> Result<RefreshTokenResult, Box<dyn std::error::Error>> {
    let result = client
        .post("http://sxz.api6.zykj.org/api/TokenAuth/RefreshToken")
        .bearer_auth(access_token)
        .header("RefreshToken", refresh_token)
        .send()
        .await?
        .json::<CommonResponse<RefreshTokenResult>>()
        .await?
        .result;
    Ok(result)
}

fn input_and_parse<T>(default: Option<T>) -> T
where
    T: std::str::FromStr + Display,
{
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    input.trim().parse::<T>().unwrap_or_else(|_| match default {
        Some(def) => {
            println!("error parsing input, using {def}");
            def
        }
        None => panic!("error parsing input"),
    })
}

#[derive(Serialize, Deserialize, Debug)]
struct LoginInfo<'a> {
    #[serde(rename = "userName")]
    user_name: &'a str,
    password: &'a str,
    #[serde(rename = "clientType")]
    client_type: i8,
}

#[derive(Serialize, Deserialize, Debug)]
struct CommonResponse<T> {
    result: T,
    success: bool,
    #[serde(rename = "unAuthorizedRequest")]
    un_authorized_request: bool,
    __abp: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct LoginResponse {
    #[serde(rename = "accessToken")]
    access_token: String,
    #[serde(rename = "expireInSeconds")]
    expire_in_seconds: i32,
    #[serde(rename = "refreshToken")]
    refresh_token: String,
    #[serde(rename = "refreshExpireInSeconds")]
    refresh_expire_in_seconds: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct RefreshTokenResult {
    #[serde(rename = "AccessToken")]
    access_token: String,
    #[serde(rename = "ExpireInSeconds")]
    expire_in_seconds: i32,
    #[serde(rename = "RefreshToken")]
    refresh_token: String,
    #[serde(rename = "RefreshExpireInSeconds")]
    refresh_expire_in_seconds: i32,
}
