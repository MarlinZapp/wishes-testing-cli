use clap_complete::Shell;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde::{Deserialize, Serialize};
use surrealdb::RecordId;

use crate::runnable::Runnable;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct TestCase<'e> {
    case: CaseNum,
    shell: &'e Shell,
    executable: &'e Path,
    surrealdb_handle: Arc<Mutex<Option<std::process::Child>>>,
}

pub enum CaseNum {
    One(CaseOneArgs),
    Two(CaseTwoArgs),
    Three(CaseThreeArgs),
    Four(CaseFourArgs),
}

#[derive(Debug)]
pub struct CaseOneArgs {
    pub n: u32,
}

#[derive(Debug)]
pub struct CaseTwoArgs {
    pub wishes: u32,
}

#[derive(Debug)]
pub struct CaseThreeArgs {
    pub wishes: u32,
}

#[derive(Debug)]
pub struct CaseFourArgs {
    pub times: u32,
}

const URL_PREFIX: &str = "http://localhost:8080/api";

impl<'e> Runnable for TestCase<'e> {
    async fn run(&mut self) -> Result<(), reqwest::Error> {
        let client = reqwest::Client::new();
        self.before();
        let mut start_time = std::time::Instant::now();
        match &self.case {
            CaseNum::One(args) => {
                println!("Test case one: Register {} users.", args.n);
                register_users(&client, args.n as usize).await?;
            }
            CaseNum::Two(args) => {
                println!(
                    "Test case two: Register 10 users and create {} wishes each.",
                    args.wishes / 10
                );
                let credentials = register_users(&client, 10).await?;
                let jwts = login_users(&client, &credentials).await?;
                create_wishes(&client, Arc::new(jwts), args.wishes).await?;
            }
            CaseNum::Three(args) => {
                println!(
                    "Test case three: Select {} wishes in one request.",
                    args.wishes
                );
                println!("Registering and creating wishes...");
                let credentials = register_users(&client, 1).await?;
                let jwts = login_users(&client, &credentials).await?;
                let arc_jwts = Arc::new(jwts);
                create_wishes(&client, Arc::clone(&arc_jwts), args.wishes).await?;
                println!("Start selecting...");
                // restart timer because registration does not count in this test case
                start_time = std::time::Instant::now();
                get_wishes(&client, arc_jwts.get(0).unwrap()).await?;
            }
            CaseNum::Four(args) => {
                println!("Test case four: Select one wish {} times.", args.times);
                println!("Registering user and creating wish...");
                let credentials = register_users(&client, 1).await?;
                let jwts = login_users(&client, &credentials).await?;
                let arc_jwts = Arc::new(jwts);
                create_wishes(&client, Arc::clone(&arc_jwts), 1).await?;
                println!("Start selecting...");
                // restart timer because registration does not count in this test case
                start_time = std::time::Instant::now();
                for _ in 0..args.times {
                    get_wishes(&client, arc_jwts.get(0).unwrap()).await?;
                }
            }
        }
        let elapsed = start_time.elapsed();
        println!(
            "Test case completed in {},{} seconds.",
            elapsed.as_secs(),
            elapsed.as_millis() % 1000
        );
        self.after();
        Ok(())
    }
}

async fn get_wishes(client: &reqwest::Client, jwt: &String) -> Result<(), reqwest::Error> {
    let wish_url = format!("{}/wishes?with_username=false", URL_PREFIX);
    let res = client.get(wish_url).bearer_auth(jwt).send().await;
    match res {
        Ok(response) => {
            if response.status() != 200 {
                eprintln!("Failed to get wishes! {}", response.text().await.unwrap());
            }
        }
        Err(err) => {
            eprintln!("Failed to get wishes: {}", err);
        }
    }
    Ok(())
}

/// For each user represented by a jwt, create n / len(jwts) wishes
async fn create_wishes(
    client: &reqwest::Client,
    jwts: Arc<Vec<String>>,
    n_wishes: u32,
) -> Result<(), reqwest::Error> {
    let mut tasks = FuturesUnordered::new();

    for i in 0..jwts.len() {
        let jwts = Arc::clone(&jwts);
        let client = client.clone(); // Clone the client for each task
        tasks.push(tokio::spawn(async move {
            for _ in 0..n_wishes / jwts.len() as u32 {
                let wish = WishCreateRequest {
                    content: generate_wish(),
                };
                let wish_url = format!("{}/wish", URL_PREFIX);
                let res = client
                    .post(wish_url)
                    .bearer_auth(jwts[i].clone())
                    .json(&wish)
                    .send()
                    .await;

                match &res {
                    Ok(response) => {
                        if response.status() != 200 {
                            eprintln!("Failed to create wish: {}", wish.content);
                        }
                    }
                    Err(err) => {
                        eprintln!("Request failed for wish {}: {}", wish.content, err);
                    }
                }
            }
        }));
    }

    // Wait for all tasks to finish
    while let Some(result) = tasks.next().await {
        if let Err(err) = result {
            eprintln!("Task panicked: {}", err);
        }
    }

    Ok(())
}

async fn login_users(
    client: &reqwest::Client,
    credentials: &Vec<Credentials>,
) -> Result<Vec<String>, reqwest::Error> {
    let mut jwts = Vec::new();
    for cred in credentials {
        let login_url = format!("{}/login", URL_PREFIX);
        let response = client
            .post(login_url)
            .header("Content-Type", "application/json")
            .json(cred)
            .send()
            .await?;
        let jwt = response.text().await?;
        if jwt.starts_with("\"") {
            jwts.push(jwt[1..jwt.len() - 1].to_string());
        } else {
            jwts.push(jwt);
        }
    }
    Ok(jwts)
}

async fn register_users(
    client: &reqwest::Client,
    n: usize,
) -> Result<Vec<Credentials>, reqwest::Error> {
    let all_credentials = Arc::new(Mutex::new(Vec::new()));
    let mut tasks = FuturesUnordered::new();

    for _ in 0..n {
        let all_credentials = Arc::clone(&all_credentials);
        let client = client.clone(); // Clone the client for each task
        tasks.push(tokio::spawn(async move {
            let credentials = Credentials {
                name: generate_username(),
                pass: generate_password(),
            };
            all_credentials.lock().unwrap().push(credentials.clone());
            let register_url = format!("{}/register", URL_PREFIX);
            let res = client
                .post(register_url)
                .header("Content-Type", "application/json")
                .json(&credentials)
                .send()
                .await;

            match &res {
                Ok(response) => {
                    if response.status() != 200 {
                        eprintln!("Failed to register user: {}", credentials.name);
                    }
                }
                Err(err) => {
                    eprintln!(
                        "Request failed for register user {}: {}",
                        credentials.name, err
                    );
                }
            }
        }));
    }

    // Wait for all tasks to finish
    while let Some(result) = tasks.next().await {
        if let Err(err) = result {
            eprintln!("Task panicked: {}", err);
        }
    }

    let all_credentials = all_credentials.lock().unwrap();
    Ok(all_credentials.clone())
}

impl<'e> TestCase<'e> {
    pub fn new(case: CaseNum, executable: &'e Path, shell: &'e Shell) -> Self {
        match std::env::consts::OS {
            "linux" => {}
            "macos" => {}
            _ => {
                panic!("tiup does not support the current OS. Please use a Linux or MacOS system.")
            }
        }
        match Command::new(shell.to_string())
            .arg("-c")
            .arg("exec tiup --version")
            .stdout(Stdio::piped())
            .spawn()
        {
            Ok(_) => {}
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => {
                    println!("`tiup` was not found! Please install it and add it to the path of your shell!")
                }
                _ => {
                    println!("Failed to start tiup playground: {}", e)
                }
            },
        }

        Self {
            case,
            shell,
            executable,
            surrealdb_handle: Arc::new(Mutex::new(None)),
        }
    }

    fn before(&mut self) {
        let mut tiup = Command::new(self.shell.to_string())
            .arg("-c")
            .arg("exec tiup playground --tag surrealdb --mode tikv-slim --pd 1 --kv 1")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("failed to start tiup playground");
        let stdout = tiup.stdout.take().expect("failed to capture stdout");
        // Wrap stdout in a buffered reader for line-by-line reading
        let reader = BufReader::new(stdout);
        let mut port = "".to_owned();
        for line in reader.lines() {
            let line = line.expect("failed to read line");
            println!("{}", line);
            if line.contains("PD Endpoints") {
                port += line.split_whitespace().last().unwrap();
            };
            if line.contains("Grafana:") {
                break; // tiup startup complete
            }
        }
        thread::sleep(Duration::from_secs(1));
        *self
            .surrealdb_handle
            .lock()
            .expect("Failed to lock surrealdb handler") = Some(
            Command::new(self.executable)
                .arg(port)
                .spawn()
                .expect("failed to start surrealdb executable"),
        );
        println!("Successfully started SurrealDB server!");
        // await backend to be ready
        thread::sleep(Duration::from_secs(2));

        // Handle Ctrl+C and send SIGINT to tiup
        let surrealdb_handle = Arc::clone(&self.surrealdb_handle);
        ctrlc::set_handler(move || {
            println!("Stopping tiup cluster!");
            let pid = get_surreal_tiup_playground_pid().expect("failed to get tiup pid");
            Command::new("kill")
                .arg("-2")
                .arg(pid.to_string())
                .spawn()
                .expect("failed to send SIGINT to tiup");
            println!("Stopping surrealdb server!");
            if let Some(ref mut handle) = *surrealdb_handle.lock().unwrap() {
                eprintln!("Sending interrupt signal to tiup...");
                handle.kill().expect("failed to send SIGINT to tiup");
            } else {
                eprintln!("surrealdb handle is None");
            }
            std::process::exit(0);
        })
        .expect("Error setting Ctrl-C handler");
    }

    fn after(&self) {
        println!("Stopping SurrealDB server!");
        self.surrealdb_handle
            .lock()
            .expect("failed to lock surreal handler")
            .as_mut()
            .unwrap()
            .kill()
            .expect("failed to send SIGINT to surreal");
        println!("Stopping tiup cluster!");
        let pid = get_surreal_tiup_playground_pid().expect("failed to get tiup pid");
        Command::new("kill")
            .arg("-2")
            .arg(pid.to_string())
            .spawn()
            .expect("failed to send SIGINT to tiup");
    }
}

fn generate_username() -> String {
    let random_chars: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(20)
        .map(char::from)
        .collect();

    format!("test{}", random_chars)
}

fn generate_password() -> String {
    let random_chars: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(20)
        .map(char::from)
        .collect();

    format!("pass{}", random_chars)
}

fn generate_wish() -> String {
    let random_chars: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(20)
        .map(char::from)
        .collect();

    format!("wish{}", random_chars)
}

fn get_surreal_tiup_playground_pid() -> Option<u32> {
    // Run the `tiup status` command
    let output = Command::new("tiup")
        .arg("status")
        .output()
        .expect("failed to execute tiup status");

    // Convert the output to a string
    let output_str = String::from_utf8_lossy(&output.stdout);

    // Iterate through the lines to find the component
    for line in output_str.lines().skip(1) {
        // Skip the header line
        let columns: Vec<&str> = line.split_whitespace().collect();
        if columns.len() >= 3 && columns[0] == "surrealdb" {
            // Parse the PID
            if let Ok(pid) = columns[2].parse::<u32>() {
                return Some(pid);
            }
        }
    }

    // If the component is not found
    None
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WishStatus {
    Submitted,
    CreationInProgress,
    InDelivery,
    Delivered,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WishCreateRequest {
    content: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InfoResponse {
    info: String,
    user: Option<User>,
    session: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum UserRole {
    Default,
    Admin,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Credentials {
    name: String,
    pass: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WishContent {
    content: String,
    status: WishStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Wish {
    id: RecordId,
    content: String,
    status: WishStatus,
    created_by: Option<RecordId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WishWithUsername {
    id: RecordId,
    content: String,
    status: WishStatus,
    created_by: Option<RecordId>,
    username: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    id: RecordId,
    name: String,
    pass: String,
    roles: Vec<UserRole>,
}
