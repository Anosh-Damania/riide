/// Events emitted by the bottom terminal panel.
pub enum TerminalEvent {
    /// The user typed a command and pressed Enter.
    RunCommand(String),
}

/// Render the integrated terminal panel at the bottom of the IDE.
///
/// - Top area: scrollable read-only text area showing `terminal_output`
/// - Bottom area: single-line `TextEdit` for `terminal_input`
/// - Pressing Enter emits `TerminalEvent::RunCommand(cmd)`
pub fn render_terminal(
    terminal_output: &[String],
    terminal_input: &mut String,
    ui: &mut egui::Ui,
) -> Option<TerminalEvent> {
    let mut event = None;

    egui::ScrollArea::vertical()
        .id_source("terminal_output_scroll")
        .max_height(150.0)
        .stick_to_bottom(true)
        .show(ui, |ui| {
            for line in terminal_output {
                ui.label(line.as_str());
            }
        });

    ui.separator();

    ui.horizontal(|ui| {
        ui.label("> ");
        let response = ui.add_sized(
            ui.available_size(),
            egui::TextEdit::singleline(terminal_input)
                .hint_text("Type a command and press Enter..."),
        );
        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            let cmd = terminal_input.trim().to_string();
            if !cmd.is_empty() {
                event = Some(TerminalEvent::RunCommand(cmd));
            }
        }
    });

    event
}