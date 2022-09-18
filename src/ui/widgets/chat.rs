//! Chat UI tab.

use std::thread::JoinHandle;

use druid::{
	im::Vector,
	widget::{Controller, Label, LineBreaking, List, Scroll},
	Data, Lens, Selector, TextAlignment, Widget, WidgetExt,
};
use tokio::sync::oneshot;

use super::settings::SETTINGS_UPDATE;
use crate::{
	chat::{ChatReceiver, Message},
	ui::UIState,
};

/// Chat UI data/state.
#[derive(Debug, Clone, Default, Data, Lens)]
pub struct Chat {
	/// Buffer of chat messages.
	#[data(same_fn = "fast_messages_same")]
	pub messages: Vector<Message>,
}

/// Fast way to make sure two vectors of messages are not the same: check first
/// and last element for equality.
fn fast_messages_same(a: &Vector<Message>, b: &Vector<Message>) -> bool {
	a.front() == b.front() && a.back() == b.back()
}

/// The Chat widget.
#[must_use]
pub fn widget() -> impl Widget<UIState> {
	let messages = List::new(|| {
		Label::dynamic(|message: &Message, _env| format!("{message}"))
			.with_line_break_mode(LineBreaking::WordWrap)
			.with_text_alignment(TextAlignment::Start)
	})
	.lens(Chat::messages);

	Scroll::new(messages)
		.vertical()
		.expand()
		.lens(UIState::chat)
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
