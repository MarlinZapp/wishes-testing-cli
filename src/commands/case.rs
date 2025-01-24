use clap_complete::Shell;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde::{Deserialize, Serialize};
use surrealdb::RecordId;

use crate::runnable::Runnable;
use std::path::Path;
use std::process::Command;
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
    Two,
}

#[derive(Debug)]
pub struct CaseOneArgs {
    pub n: u32,
}

const URL_PREFIX: &str = "http://localhost:8080/api";

impl<'e> Runnable for TestCase<'e> {
    async fn run(&mut self) -> Result<(), reqwest::Error> {
        let client = reqwest::Client::new();
        self.before();
        let start_time = std::time::Instant::now();
        match &self.case {
            CaseNum::One(args) => {
                println!("Test case one: Register {} users.", args.n);
                register_users(client, args.n as usize).await?;
            }
            CaseNum::Two => {
                println!("Test case two");
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

async fn register_users(client: reqwest::Client, n: usize) -> Result<(), reqwest::Error> {
    let tasks = futures::stream::iter(0..n)
        .map(|_| {
            let client = client.clone();
            async move {
                let credentials = Credentials {
                    name: generate_username(),
                    pass: generate_password(),
                };
                let register_url = format!("{}/register", URL_PREFIX);
                client
                    .post(register_url)
                    .header("Content-Type", "application/json")
                    .json(&credentials)
                    .send()
                    .await
            }
        })
        .buffer_unordered(16);

    tasks
        .for_each(|res| async {
            match res {
                Ok(_) => {}
                Err(err) => eprintln!("Request failed: {}", err),
            }
        })
        .await;

    Ok(())
}

impl<'e> TestCase<'e> {
    pub fn new(case: CaseNum, executable: &'e Path, shell: &'e Shell) -> Self {
        Self {
            case,
            shell,
            executable,
            surrealdb_handle: Arc::new(Mutex::new(None)),
        }
    }

    fn before(&mut self) {
        match self.shell {
            Shell::Zsh => {
                Command::new("zsh")
                    .arg("-c")
                    .arg("exec tiup playground --tag surrealdb --mode tikv-slim --pd 1 --kv 1")
                    .spawn()
                    .expect("failed to start tiup playground");
            }
            _ => {
                eprintln!("Shell not yet supported for this test case.");
            }
        }
        // await tiup playground to be ready
        thread::sleep(Duration::from_secs(8));
        *self
            .surrealdb_handle
            .lock()
            .expect("Failed to lock surrealdb handler") = Some(
            Command::new(self.executable)
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
