use std::path::PathBuf;

use crate::editor::buffer_manager::BufferManager;
use crate::io::fs::{FileSystemOps, RealFileSystem};
use crate::io::lsp;
use crate::io::terminal;
use crate::state::error::ErrorState;
use crate::state::workspace::WorkspaceState;
use crate::ui::bottom_panel::{self, TerminalEvent};
use crate::ui::central_panel::{self, EditorEvent};
use crate::ui::side_panel::{self, DirEntry, ExplorerEvent};

/// The root application struct. Owns all state, I/O, and orchestrates
/// the UI rendering loop through an event-driven pattern.
pub struct RiideApp {
    workspace: WorkspaceState,
    buffers: BufferManager,
    errors: ErrorState,
    fs: RealFileSystem,
    dir_tree: Vec<DirEntry>,
    terminal_output: Vec<String>,
    terminal_rx: Option<std::sync::mpsc::Receiver<String>>,
    terminal_input: String,
    lsp_client: Option<lsp::LspClient>,
}

impl Default for RiideApp {
    fn default() -> Self {
        let mut app = Self {
            workspace: WorkspaceState::new(std::env::current_dir().unwrap_or_default()),
            buffers: BufferManager::new(),
            errors: ErrorState::default(),
            fs: RealFileSystem,
            dir_tree: Vec::new(),
            terminal_output: Vec::new(),
            terminal_rx: None,
            terminal_input: String::new(),
            lsp_client: None,
        };
        app.rebuild_dir_tree();
        app
    }
}

impl RiideApp {
    /// Rebuild the cached directory tree.
    /// Call this during initialization and whenever the tree structure changes
    /// (after ToggleDir, GoToParent, NavigateTo).
    fn rebuild_dir_tree(&mut self) {
        self.dir_tree = build_tree(
            &self.fs,
            self.workspace.current_dir(),
            &self.workspace.expanded_dirs,
        );
    }
}

/// Recursively build a DirEntry tree for the given root directory.
/// Only reads directories that are in expanded_dirs.
fn build_tree(
    fs: &dyn FileSystemOps,
    root: &std::path::Path,
    expanded: &std::collections::HashSet<PathBuf>,
) -> Vec<DirEntry> {
    let entries = match fs.read_dir_entries(root) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut result: Vec<DirEntry> = Vec::new();
    for path in entries {
        let name = match path.file_name().and_then(|s| s.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        let is_dir = path.is_dir();
        let children = if is_dir && expanded.contains(&path) {
            build_tree(fs, &path, expanded)
        } else {
            Vec::new()
        };

        result.push(DirEntry {
            path,
            name,
            is_dir,
            children,
        });
    }
    result
}

impl eframe::App for RiideApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = ctx.input(|i| i.time);
        self.errors.update(now);

        ctx.set_visuals(egui::Visuals::dark());

        // Drain any pending terminal output from background processes
        if let Some(ref rx) = self.terminal_rx {
            let mut new_output = false;
            while let Ok(line) = rx.try_recv() {
                self.terminal_output.push(line);
                new_output = true;
            }
            if new_output {
                ctx.request_repaint(); // ensure immediate UI refresh
            }
        }

        // Drain any pending LSP responses from the language server
        if let Some(ref client) = self.lsp_client {
            let mut new_output = false;
            loop {
                match client.rx.try_recv() {
                    Ok(line) => {
                        // Parse into serde_json::Value for future routing
                        match serde_json::from_str::<serde_json::Value>(&line) {
                            Ok(_) => {
                                self.terminal_output
                                    .push(format!("[LSP] Received: {}", line));
                                new_output = true;
                            }
                            Err(_) => {
                                // Malformed JSON — still log it for debugging
                                self.terminal_output
                                    .push(format!("[LSP] Malformed: {}", line));
                                new_output = true;
                            }
                        }
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => break,
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        self.terminal_output
                            .push("[LSP] ERROR: Server disconnected!".to_string());
                        new_output = true;
                        break;
                    }
                }
            }
            if new_output {
                ctx.request_repaint();
            }
        }

        for msg in self.errors.active() {
            egui::Window::new("Error")
                .collapsible(false)
                .resizable(false)
                .title_bar(false)
                .fixed_pos(egui::Pos2::new(300.0, 30.0))
                .show(ctx, |ui| {
                    ui.label(egui::RichText::new(msg).color(egui::Color32::RED));
                });
        }

        // Unsaved Changes modal — rendered on top of the panels, which are
        // still drawn behind it so the IDE background remains visible.
        if let Some(ref pending_path) = self.workspace.pending_close_tab.clone() {
            let file_name = pending_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("?")
                .to_string();

            egui::Window::new("Unsaved Changes")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(format!(
                        "'{}' has unsaved changes.\nSave before closing?",
                        file_name
                    ));
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            if let Some(buf) = self.buffers.get_mut(pending_path) {
                                buf.sync_to_rope();
                                if let Err(e) = self.fs.write_file(pending_path, &buf.rope) {
                                    self.errors.push(e);
                                }
                            }
                            self.buffers.remove(pending_path);
                            self.workspace.close_tab(pending_path);
                            self.workspace.pending_close_tab = None;
                        }
                        if ui.button("Don't Save").clicked() {
                            self.buffers.remove(pending_path);
                            self.workspace.close_tab(pending_path);
                            self.workspace.pending_close_tab = None;
                        }
                        if ui.button("Cancel").clicked() {
                            self.workspace.pending_close_tab = None;
                        }
                    });
                });
        }

        let terminal_event = egui::TopBottomPanel::bottom("terminal_panel")
            .resizable(true)
            .default_height(200.0)
            .show(ctx, |ui| {
                bottom_panel::render_terminal(
                    &self.terminal_output,
                    &mut self.terminal_input,
                    ui,
                )
            })
            .inner;

        let explorer_event = egui::SidePanel::left("file_explorer_panel")
            .resizable(true)
            .default_width(250.0)
            .show(ctx, |ui| {
                ui.heading("File Explorer");
                ui.separator();
                side_panel::render_file_explorer(&mut self.workspace, &self.dir_tree, ui)
            })
            .inner;

        let editor_event = egui::CentralPanel::default()
            .show(ctx, |ui| {
                central_panel::render_editor(&mut self.buffers, &mut self.workspace, ctx, ui)
            })
            .inner;

        if let Some(event) = explorer_event {
            match event {
                ExplorerEvent::OpenFile(path) => {
                    if !self.buffers.contains(&path) {
                        match self.fs.read_file(&path) {
                            Ok(rope) => {
                                self.buffers.load(path.clone(), rope);
                            }
                            Err(e) => {
                                self.errors.push(e);
                                return;
                            }
                        }
                    }
                    self.workspace.open_tab(path);
                }
                ExplorerEvent::ToggleDir(path) => {
                    self.workspace.toggle_expanded(&path);
                    self.rebuild_dir_tree();
                }
                ExplorerEvent::GoToParent => {
                    if let Some(parent) = self.workspace.current_dir().parent() {
                        self.workspace.navigate_to(parent.to_path_buf());
                        self.rebuild_dir_tree();
                    }
                }
            }
        }

        if let Some(event) = editor_event {
            match event {
                EditorEvent::StartLsp => {
                    self.terminal_output
                        .push("Starting rust-analyzer...".to_string());
                    let mut client = lsp::LspClient::start("rust-analyzer");

                    // Send initialize request
                    client.send_request(
                        "initialize",
                        serde_json::json!({
                            "processId": null,
                            "rootUri": null,
                            "capabilities": {}
                        }),
                    );

                    self.terminal_output
                        .push("[LSP] Sent initialize request".to_string());
                    self.lsp_client = Some(client);
                }
                EditorEvent::SaveFile => {
                    let path: PathBuf = match self.workspace.active_file_path() {
                        Some(p) => p.clone(),
                        None => {
                            self.errors.push("No file is currently open for saving.");
                            return;
                        }
                    };
                    match self.buffers.get_mut(&path) {
                        Some(buf) => {
                            buf.sync_to_rope();
                            match self.fs.write_file(&path, &buf.rope) {
                                Ok(()) => {
                                    buf.clear_dirty();
                                    self.errors.push_with_expiry(
                                        format!("File saved: {}", path.display()),
                                        now,
                                    );
                                }
                                Err(e) => self.errors.push(e),
                            }
                        }
                        None => {
                            self.errors.push("Buffer not found for the active file.");
                            return;
                        }
                    }
                }
                EditorEvent::SwitchTab(path) => {
                    self.workspace.switch_tab(&path);
                }
                EditorEvent::CloseTab(path) => {
                    // If the buffer has unsaved changes, show the modal instead of closing
                    let is_dirty = self
                        .buffers
                        .get(&path)
                        .map(|b| b.is_dirty())
                        .unwrap_or(false);
                    if is_dirty {
                        self.workspace.pending_close_tab = Some(path);
                    } else {
                        self.buffers.remove(&path);
                        self.workspace.close_tab(&path);
                    }
                }
            }
        }

        if let Some(event) = terminal_event {
            match event {
                TerminalEvent::RunCommand(cmd) => {
                    // Push a visual indicator of the command
                    self.terminal_output.push(format!("> {}", cmd));
                    // Spawn the command and store the receiver
                    let (rx, _handle) = terminal::spawn_command(&cmd);
                    self.terminal_rx = Some(rx);
                    self.terminal_input.clear();
                }
            }
        }
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {}
}