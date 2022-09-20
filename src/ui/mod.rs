//! UI part of the app.

use druid::{AppLauncher, Data, Lens, LocalizedString, WindowDesc};

pub mod widgets;

/// The root UI data/state.
#[derive(Debug, Clone, Default, Data, Lens)]
pub struct UIState {
	/// Overview data/state.
	pub overview: widgets::overview::Overview,
	/// Chat data/state.
	pub chat: widgets::chat::Chat,
	/// Giveaway data/state.
	pub giveaway: widgets::giveaway::Giveaway,
	/// Settings data/state.
	pub settings: widgets::settings::Settings,
}

/// Get the window launcher for this UI.
#[must_use]
pub fn window_launcher() -> AppLauncher<UIState> {
	let window = WindowDesc::new(widgets::root_widget())
		.title(LocalizedString::new("Window-Title").with_placeholder("Twitch Chat Metrics"))
		.window_size((1100.0, 550.0));
	AppLauncher::with_window(window)
}
