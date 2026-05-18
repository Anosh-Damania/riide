use std::path::PathBuf;

use crate::state::workspace::WorkspaceState;

/// Events that can be emitted by the file explorer side panel.
pub enum ExplorerEvent {
    /// Navigate into a directory.
    NavigateTo(PathBuf),
    /// Open a file for editing.
    OpenFile(PathBuf),
    /// Move up to the parent directory.
    GoToParent,
}

/// Render the file explorer side panel.
///
/// Takes a pre-computed (sorted) list of `entries` — the orchestrator
/// calls `FileSystemOps::read_dir_entries` and passes the result here.
///
/// Returns `Some(ExplorerEvent)` when the user interacts with the panel.
pub fn render_file_explorer(
    workspace: &mut WorkspaceState,
    entries: &[PathBuf],
    ui: &mut egui::Ui,
) -> Option<ExplorerEvent> {
    let mut event: Option<ExplorerEvent> = None;

    // Current directory header
    ui.label(format!("📁 {}", workspace.current_dir().display()));

    // Parent directory button
    if ui.button("⬆  ..").clicked() {
        event = Some(ExplorerEvent::GoToParent);
    }
    ui.separator();

    // Scrollable file listing
    egui::ScrollArea::vertical().show(ui, |ui| {
        for path in entries {
            let file_name = match path.file_name().and_then(|s| s.to_str()) {
                Some(name) => name.to_string(),
                None => continue,
            };

            let label = if path.is_dir() {
                egui::RichText::new(format!("📁  {}", file_name))
                    .color(egui::Color32::LIGHT_BLUE)
            } else {
                egui::RichText::new(format!("📄  {}", file_name))
            };

            if ui.label(label).interact(egui::Sense::click()).clicked() {
                if path.is_dir() {
                    event = Some(ExplorerEvent::NavigateTo(path.clone()));
                } else {
                    event = Some(ExplorerEvent::OpenFile(path.clone()));
                }
            }
        }
    });

    event
}