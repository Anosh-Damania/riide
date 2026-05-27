use std::io::{BufWriter, Read, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::thread::JoinHandle;

/// Manages the lifecycle of a Language Server Protocol process.
///
/// Spawns the server process, owns a writer to its stdin for sending requests,
/// and exposes an mpsc receiver for reading JSON responses parsed from stdout.
pub struct LspClient {
    stdin: BufWriter<ChildStdin>,
    /// Incoming JSON messages from the language server.
    pub rx: mpsc::Receiver<String>,
    _child: Child,
    _thread: JoinHandle<()>,
    next_id: AtomicU64,
}

impl LspClient {
    /// Start a language server process and begin reading its stdout.
    ///
    /// Spawns `server_cmd` with piped stdin/stdout. A background thread reads
    /// LSP messages by parsing the `Content-Length: N\r\n\r\n` header byte-by-byte
    /// (buffered via `BufReader`), then reading exactly N bytes of JSON body.
    pub fn start(server_cmd: &str) -> Self {
        let mut child = Command::new(server_cmd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("Failed to start LSP server");

        let stdin = BufWriter::new(child.stdin.take().expect("Failed to capture stdin"));
        let stdout = child.stdout.take().expect("Failed to capture stdout");

        let (tx, rx) = mpsc::channel::<String>();

        let thread = std::thread::spawn(move || {
            let mut reader = std::io::BufReader::new(stdout);
            let mut header_buf = Vec::new();

            loop {
                // Read the header one byte at a time (buffered) until "\r\n\r\n"
                header_buf.clear();
                loop {
                    let mut byte = [0u8; 1];
                    if reader.read_exact(&mut byte).is_err() {
                        return; // stdout closed — server exited
                    }
                    header_buf.push(byte[0]);
                    // Check if the last 4 bytes are the header terminator
                    if header_buf.len() >= 4
                        && header_buf[header_buf.len() - 4..] == [b'\r', b'\n', b'\r', b'\n']
                    {
                        break;
                    }
                }

                // Parse Content-Length from the header
                let header: String = String::from_utf8_lossy(&header_buf).to_string();
                let content_length = header
                    .to_lowercase()
                    .lines()
                    .find_map(|line| {
                        line.strip_prefix("content-length:")
                            .and_then(|v| v.trim().parse::<usize>().ok())
                    })
                    .unwrap_or(0);

                if content_length == 0 {
                    continue;
                }

                // Read exactly content_length bytes of JSON body
                let mut body = vec![0u8; content_length];
                if reader.read_exact(&mut body).is_err() {
                    return;
                }

                let json = String::from_utf8_lossy(&body).to_string();
                if tx.send(json).is_err() {
                    return; // Receiver dropped — shut down
                }
            }
        });

        Self {
            stdin,
            rx,
            _child: child,
            _thread: thread,
            next_id: AtomicU64::new(1),
        }
    }

    /// Send a JSON-RPC 2.0 request to the language server.
    ///
    /// Constructs the request object, serializes it, prepends the Content-Length
    /// header, and writes to the server's stdin.
    pub fn send_request(&mut self, method: &str, params: serde_json::Value) {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });

        let body = serde_json::to_string(&request).expect("Failed to serialize JSON-RPC request");
        let header = format!("Content-Length: {}\r\n\r\n", body.len());

        self.stdin.write_all(header.as_bytes()).ok();
        self.stdin.write_all(body.as_bytes()).ok();
        self.stdin.flush().ok();
    }
}