use reqwest::redirect::Policy;
use serde::Deserialize;
use serde::Serialize;
use simplelog::{ConfigBuilder, LevelFilter, TermLogger, TerminalMode};
use std::process::Command;
use std::str::FromStr;
use std::{env, process, thread};
use tokio::time::Duration;

// Arguments:
// - Executable
// - Workdir
// - Endpoint

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    TermLogger::init(
        LevelFilter::Debug,
        ConfigBuilder::new()
            .set_time_format_str("%Y-%m-%d %H:%M:%S%.3f")
            .build(),
        TerminalMode::Mixed,
    )
    .expect("Unable to bind terminal logger.");
    let args: Vec<_> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Wrong number of arguments.");
        process::exit(1);
    }

    let mut args = args.into_iter();
    args.next();
    let executable = args.next().unwrap();
    let work_dir = args.next().unwrap();
    let endpoint = args.next().unwrap();
    let access_token = env::var("access_token").expect("No access_token found.");
    let corp_id = env::var("corp_id").unwrap();
    let secret = env::var("secret").unwrap();
    let agent_id = i32::from_str(&env::var("agent_id").unwrap()).unwrap();

    // 0. Start server.
    thread::spawn(move || {
        println!("Starting the server.");
        Command::new(executable)
            .current_dir(work_dir)
            .spawn()
            .expect("Failed to start");
    });

    tokio::time::delay_for(Duration::from_secs(10)).await;
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .redirect(Policy::none())
        .build()
        .unwrap();
    // 1. I can access index.
    let resp = client.get(&format!("{}/", endpoint)).send().await?;
    assert!(resp.status().is_success());

    // 2. I have to login.
    let resp = client
        .post(&format!("{}/login", endpoint))
        .query(&[("access_token", &access_token)])
        .send()
        .await?;
    assert!(resp.status().is_redirection());
    let location = resp.headers().get("Location").expect("No Location found.");
    assert_eq!("/#/user", location.to_str().unwrap());

    #[derive(Serialize)]
    struct Wechat {
        corp_id: String,
        agent_id: i32,
        secret: String,
    }
    // 3. I will fill my information.
    let resp = client
        .put(&format!("{}/wechat", endpoint))
        .json(&Wechat {
            corp_id,
            agent_id,
            secret,
        })
        .send()
        .await?;
    assert!(resp.status().is_success());

    #[derive(Deserialize)]
    pub struct User {
        callback_url: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct Response {
        success: bool,
    }

    // 4. Get my callback url.
    let resp = client.get(&format!("{}/user", endpoint)).send().await?;
    assert!(resp.status().is_success());
    let callback = resp.json::<User>().await?;
    assert!(!callback.callback_url.is_empty());

    // 5. Send message.
    let resp = client
        .get(&callback.callback_url)
        .query(&[("text", "Message challenge sent from PipeHub test.")])
        .send()
        .await?;
    assert!(resp.status().is_success());
    let resp = resp.json::<Response>().await?;
    assert!(resp.success);

    Ok(())
}
