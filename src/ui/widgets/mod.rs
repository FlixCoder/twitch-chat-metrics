//! The UI widgets.

pub mod chat;
pub mod overview;
pub mod settings;

use druid::{
	widget::{Tabs, TabsTransition},
	Widget,
};

use super::UIState;

/// The root UI widget.
#[must_use]
pub fn root_widget() -> impl Widget<UIState> {
	let overview = overview::widget();
	let chat = chat::widget();
	let settings = settings::widget();

	Tabs::new()
		.with_transition(TabsTransition::Slide(100_000_000))
		.with_tab("Overview", overview)
		.with_tab("Chat", chat)
		.with_tab("Settings", settings)
}
