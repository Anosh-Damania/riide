use crate::editor::buffer::Buffer;

/// Events that can be emitted by the central editor panel.
pub enum EditorEvent {
    /// User requested to save the current file.
    SaveFile,
}

/// Render the central editor panel.
///
/// Only receives `Buffer` (text content).
/// No `FileSystemOps` — save events are returned and handled by the orchestrator.
///
/// Returns `Some(EditorEvent)` when the user triggers a save.
pub fn render_editor(
    buffer: &mut Buffer,
    ctx: &egui::Context,
    ui: &mut egui::Ui,
) -> Option<EditorEvent> {
    let mut event: Option<EditorEvent> = None;

    // Handle Ctrl+S / Cmd+S
    let needs_save = ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::S));
    if needs_save {
        event = Some(EditorEvent::SaveFile);
    }

    if let Some(path) = buffer.path().cloned() {
        ui.horizontal(|ui| {
            ui.label(format!("✏️  {}", path.display()));
            if ui.button("💾  Save").clicked() {
                event = Some(EditorEvent::SaveFile);
            }
        });
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_sized(
                ui.available_size(),
                egui::TextEdit::multiline(buffer.content_mut())
                    .code_editor()
                    .desired_width(f32::INFINITY),
            );
        });
    } else {
        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() / 3.0);
            ui.heading("Open a file from the sidebar to start editing");
        });
    }

    event
}