use std::path::PathBuf;

use crate::editor::buffer_manager::BufferManager;
use crate::state::workspace::WorkspaceState;

/// Events that can be emitted by the central editor panel.
pub enum EditorEvent {
    SaveFile,
    SwitchTab(PathBuf),
    CloseTab(PathBuf),
}

/// Render the central editor panel with a tab bar on top.
///
/// The tab bar shows all open tabs from `workspace.open_tabs`.
/// Dirty tabs have a ` *` appended to their label.
/// The text editor below renders the buffer for the active tab.
pub fn render_editor(
    buffers: &mut BufferManager,
    workspace: &mut WorkspaceState,
    ctx: &egui::Context,
    ui: &mut egui::Ui,
) -> Option<EditorEvent> {
    let mut event: Option<EditorEvent> = None;

    // Handle Ctrl+S / Cmd+S
    let needs_save = ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::S));
    if needs_save {
        event = Some(EditorEvent::SaveFile);
    }

    // Tab bar
    if !workspace.open_tabs.is_empty() {
        egui::ScrollArea::horizontal().id_source("tab_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                let tabs = workspace.open_tabs.clone();
                for tab_path in &tabs {
                    let file_name = tab_path
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("?")
                        .to_string();

                    // Append dirty indicator if the buffer has unsaved changes
                    let display_name = if buffers.get(tab_path).map(|b| b.is_dirty()).unwrap_or(false)
                    {
                        format!("{} *", file_name)
                    } else {
                        file_name
                    };

                    let is_active = workspace.active_file_path.as_deref() == Some(tab_path);

                    let mut frame = egui::Frame::none();
                    if is_active {
                        frame = frame.fill(egui::Color32::from_gray(40));
                    }
                    frame.show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let label = if is_active {
                                egui::RichText::new(&display_name).color(egui::Color32::WHITE)
                            } else {
                                egui::RichText::new(&display_name).color(egui::Color32::LIGHT_GRAY)
                            };
                            if ui.selectable_label(is_active, label).clicked() {
                                event = Some(EditorEvent::SwitchTab(tab_path.clone()));
                            }
                            if ui.button("x").clicked() {
                                event = Some(EditorEvent::CloseTab(tab_path.clone()));
                            }
                        });
                    });
                }
            });
        });
        ui.separator();
    }

    // Text editor for the active buffer
    if let Some(active_path) = workspace.active_file_path.clone() {
        if let Some(buffer) = buffers.get_mut(&active_path) {
            if ui.button("Save").clicked() {
                event = Some(EditorEvent::SaveFile);
            }
            egui::ScrollArea::vertical().show(ui, |ui| {
                let response = ui.add_sized(
                    ui.available_size(),
                    egui::TextEdit::multiline(buffer.content_mut())
                        .code_editor()
                        .desired_width(f32::INFINITY),
                );
                if response.changed() {
                    buffer.mark_dirty();
                }
            });
        } else {
            ui.label("Buffer not loaded. Re-opening file...");
        }
    } else {
        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() / 3.0);
            ui.heading("Open a file from the sidebar to start editing");
        });
    }

    event
}