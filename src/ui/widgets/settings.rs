//! Settings tab.

use std::fs::File;

use color_eyre::Result;
use druid::{
	text::ParseFormatter,
	widget::{Button, Controller, Flex, Label, TextBox},
	Command, Data, Env, EventCtx, Lens, Selector, Target, Widget, WidgetExt,
};
use serde::{Deserialize, Serialize};

use crate::ui::UIState;

/// Selector string for notifying of settings updates via a command.
pub const SETTINGS_UPDATE: Selector<()> = Selector::new("SETTINGS_UPDATE");

/// The Settings data + UI state.
#[derive(Debug, Clone, Serialize, Deserialize, Data, Lens)]
pub struct Settings {
	/// Indicates whether a setting was edited and need to be saved.
	#[serde(default, skip_serializing)]
	pub changes: bool,
	#[serde(default)]
	/// The twitch channel login name.
	pub twitch_channel: String,
	/// The number of messages the chat should buffer.
	#[serde(default = "Settings::default_chat_buffer")]
	pub chat_buffer: usize,
}

impl Default for Settings {
	fn default() -> Self {
		Self {
			changes: false,
			twitch_channel: String::new(),
			chat_buffer: Self::default_chat_buffer(),
		}
	}
}

impl Settings {
	/// The filename where the settings file is stored.
	const STATE_FILE: &'static str = "settings.json";

	/// Load the settings from the file.
	pub fn from_file() -> Result<Self> {
		let file = File::open(Self::STATE_FILE)?;
		let state = serde_json::from_reader(file)?;
		Ok(state)
	}

	/// Save the settings to the file.
	pub fn save(&self) -> Result<()> {
		let file = File::create(Self::STATE_FILE)?;
		serde_json::to_writer_pretty(file, self)?;
		Ok(())
	}

	/// Settings default value for `chat_buffer`.
	fn default_chat_buffer() -> usize {
		250
	}
}

/// The settings UI widget.
#[must_use]
pub fn widget() -> impl Widget<UIState> {
	let labels = Flex::column()
		.with_child(Label::new("Twitch channel: "))
		.with_child(Label::new("Chat buffer size:"));

	let text_boxes = Flex::column()
		.with_child(
			TextBox::new()
				.with_line_wrapping(true)
				.with_placeholder("<channel-name>")
				.lens(Settings::twitch_channel)
				.expand_width(),
		)
		.with_child(
			TextBox::new()
				.with_formatter(ParseFormatter::default())
				.lens(Settings::chat_buffer)
				.expand_width(),
		);

	let columns = Flex::row().with_child(labels).with_flex_child(text_boxes, 2.0).expand_width();

	let save =
		Button::new("Save").disabled_if(|data: &Settings, _env| !data.changes).on_click(on_save);

	Flex::column()
		.with_child(columns)
		.with_default_spacer()
		.with_child(save)
		.expand_width()
		.controller(DataChangeDetector::default())
		.lens(UIState::settings)
}

/// On click of the settings save button.
fn on_save(ctx: &mut EventCtx, data: &mut Settings, _env: &Env) {
	data.changes = false;
	data.save().expect("saving settings");
	ctx.submit_command(Command::new(SETTINGS_UPDATE, (), Target::Auto));
	tracing::debug!("Settings were saved!");
}

/// Controller for detecting changes of the settings-values.
#[derive(Debug, Default)]
struct DataChangeDetector(bool);

impl<W: Widget<Settings>> Controller<Settings, W> for DataChangeDetector {
	fn event(
		&mut self,
		child: &mut W,
		ctx: &mut EventCtx,
		event: &druid::Event,
		data: &mut Settings,
		env: &Env,
	) {
		if let druid::Event::WindowConnected = event {
			*data = Settings::from_file().unwrap_or_default();
			ctx.submit_command(Command::new(SETTINGS_UPDATE, (), Target::Auto));
		}

		if self.0 {
			data.changes = true;
			self.0 = false;
		}

		child.event(ctx, event, data, env);
	}

	fn update(
		&mut self,
		child: &mut W,
		ctx: &mut druid::UpdateCtx,
		old_data: &Settings,
		data: &Settings,
		env: &Env,
	) {
		if !data.changes && !old_data.changes && !old_data.same(data) {
			self.0 = true;
		}

		child.update(ctx, old_data, data, env);
	}
}
