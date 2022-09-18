//! Binary executor of the app.

use color_eyre::Result;
use twitch_chat_metrics::ui;

fn main() -> Result<()> {
	color_eyre::install()?;

	let launcher = ui::window_launcher();
	let _event_sender = launcher.get_external_handle();

	launcher.log_to_console().launch(ui::UIState::default())?;

	Ok(())
}
