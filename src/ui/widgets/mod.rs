//! The UI widgets.

pub mod chat;
pub mod giveaway;
pub mod overview;
pub mod settings;

use std::thread::JoinHandle;

use druid::{
	widget::{Controller, Tabs, TabsTransition},
	Selector, Widget, WidgetExt,
};
use tokio::sync::oneshot;

use self::settings::SETTINGS_UPDATE;
use super::UIState;
use crate::chat::ChatReceiver;

/// The root UI widget.
#[must_use]
pub fn root_widget() -> impl Widget<UIState> {
	let overview = overview::widget();
	let chat = chat::widget();
	let giveaway = giveaway::widget();
	let settings = settings::widget();

	Tabs::new()
		.with_transition(TabsTransition::Slide(100_000_000))
		.with_tab("Overview", overview)
		.with_tab("Chat", chat)
		.with_tab("Giveaway", giveaway)
		.with_tab("Settings", settings)
		.controller(ChatReceiverSpawner::default())
}

/// Controller that receives settings changes and spawns the chat listener based
/// on it.
#[derive(Debug, Default)]
struct ChatReceiverSpawner {
	/// The chat listener's thread's join handle.
	join_handle: Option<JoinHandle<()>>,
	/// Sender to send a signal to stop the chat listener thread.
	stop_trigger: Option<oneshot::Sender<()>>,
}

impl<W: Widget<UIState>> Controller<UIState, W> for ChatReceiverSpawner {
	fn event(
		&mut self,
		child: &mut W,
		ctx: &mut druid::EventCtx,
		event: &druid::Event,
		data: &mut UIState,
		env: &druid::Env,
	) {
		if let druid::Event::Command(command) = event {
			if command.get(Selector::<()>::new(SETTINGS_UPDATE)).is_some() {
				// Clean up previous client thread
				if let Some(stop_trigger) = self.stop_trigger.take() {
					stop_trigger.send(()).ok();
				}
				data.chat.messages.clear();

				// Start new client for new channel
				let (stop_trigger_sender, stop_trigger_receiver) = oneshot::channel();
				let join_handle = ChatReceiver::builder()
					.channel(data.settings.twitch_channel.to_lowercase())
					.event_sender(ctx.get_external_handle())
					.stop_trigger(stop_trigger_receiver)
					.build()
					.spawn();

				self.join_handle = Some(join_handle);
				self.stop_trigger = Some(stop_trigger_sender);
			}
		}

		child.event(ctx, event, data, env);
	}
}
