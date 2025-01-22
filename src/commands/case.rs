use clap_complete::Shell;

use crate::runnable::Runnable;
use std::path::Path;
use std::process::{self, Command};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct TestCase<'e> {
    case: CaseNum,
    shell: &'e Shell,
    executable: &'e Path,
    tiup_handle: Arc<Mutex<Option<std::process::Child>>>,
    surrealdb_handle: Arc<Mutex<Option<std::process::Child>>>,
}

impl<'e> TestCase<'e> {
    pub fn new(case: CaseNum, executable: &'e Path, shell: &'e Shell) -> Self {
        Self {
            case,
            shell,
            executable,
            tiup_handle: Arc::new(Mutex::new(None)),
            surrealdb_handle: Arc::new(Mutex::new(None)),
        }
    }

    fn before(&mut self) {
        match self.shell {
            Shell::Zsh => {
                self.tiup_handle = Arc::new(Mutex::new(Some(
                    Command::new("zsh")
                        .arg("-c")
                        .arg("exec tiup playground --tag surrealdb --mode tikv-slim --pd 1 --kv 1")
                        .spawn()
                        .expect("failed to start tiup playground"),
                )));
            }
            _ => {
                eprintln!("Shell not yet supported for this test case.");
            }
        }
        // await tiup playground to be ready
        thread::sleep(Duration::from_secs(7));
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
        thread::sleep(Duration::from_secs(1));

        // Handle Ctrl+C and send SIGINT to tiup
        let tiup_handle = Arc::clone(&self.tiup_handle);
        let surrealdb_handle = Arc::clone(&self.surrealdb_handle);
        ctrlc::set_handler(move || {
            if let Some(ref mut handle) = *tiup_handle.lock().unwrap() {
                eprintln!("Sending interrupt signal to tiup...");
                handle.kill().expect("failed to send SIGINT to tiup");
            } else {
                eprintln!("tiup playground handle is None");
            }
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

pub enum CaseNum {
    One,
    Two,
}

impl<'e> Runnable for TestCase<'e> {
    fn run(&mut self) {
        self.before();
        match self.case {
            CaseNum::One => {
                println!("Test case one");
            }
            CaseNum::Two => {
                println!("Test case two");
            }
        }
        self.after();
    }
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
