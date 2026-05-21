use std::io::BufRead;
use std::io::BufReader;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread::JoinHandle;

/// Represents a spawned terminal process (currently opaque, reserved for future use).
#[allow(dead_code)]
pub struct TerminalProcess;

/// Spawn a shell command in a background thread.
///
/// Uses `std::process::Command` with `Stdio::piped()` for stdout/stderr.
/// A `std::thread` reads lines from both streams via `BufReader` and sends
/// each line through an `mpsc::Sender<String>`.
///
/// Returns:
/// - `mpsc::Receiver<String>` — polled in the egui update loop via `try_recv()`
/// - `JoinHandle<()>` — stored (but not joined) to keep the thread alive
pub fn spawn_command(command: &str) -> (mpsc::Receiver<String>, JoinHandle<()>) {
    let (tx, rx) = mpsc::channel::<String>();

    // Use `cmd /c` on Windows, `sh -c` on Unix
    #[cfg(target_os = "windows")]
    let (shell, flag) = ("cmd", "/c");
    #[cfg(not(target_os = "windows"))]
    let (shell, flag) = ("sh", "-c");

    let mut child = Command::new(shell)
        .arg(flag)
        .arg(command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn command");

    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let stderr = child.stderr.take().expect("Failed to capture stderr");

    let handle = std::thread::spawn(move || {
        let stdout_reader = BufReader::new(stdout);
        let stderr_reader = BufReader::new(stderr);

        // Merge stdout and stderr into a single stream of lines
        let stdout_lines = stdout_reader.lines().map(|l| l.unwrap_or_default());
        let stderr_lines = stderr_reader.lines().map(|l| l.unwrap_or_default());
        let combined = stdout_lines.chain(stderr_lines);

        for line in combined {
            if tx.send(line).is_err() {
                break; // Receiver dropped — exit the thread
            }
        }

        let _ = child.wait(); // reap the child process
    });

    (rx, handle)
}