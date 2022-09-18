//! Overview tab.

use std::sync::Arc;

use druid::{
	im::HashMap,
	widget::{Controller, Flex, Label},
	Data, Lens, Selector, Widget, WidgetExt,
};

use super::settings::SETTINGS_UPDATE;
use crate::{
	chat::{ClearMessage, Message, CLEAR_CHAT_MESSAGE, NEW_CHAT_MESSAGE},
	ui::UIState,
};

/// Overview UI data/state.
#[derive(Debug, Clone, Default, Data, Lens)]
pub struct Overview {
	/// Map from user IDs to user names.
	#[data(same_fn = "fast_unique_chatters_same")]
	unique_chatters: HashMap<String, String>,
	/// Number of total messages.
	total_messages: usize,
	/// Number of messages by subscribers.
	subscriber_messages: usize,
	/// Number of total bits cheered.
	total_bits: u64,
	/// Number of messages cleared.
	messages_cleared: usize,
}

/// Simple fast comparison function for unique chatters. Members are only added,
/// so compare the length only.
fn fast_unique_chatters_same(a: &HashMap<String, String>, b: &HashMap<String, String>) -> bool {
	a.len() == b.len()
}

/// The overview UI widget.
#[must_use]
pub fn widget() -> impl Widget<UIState> {
	let unique_chatters = Label::dynamic(|chatters: &HashMap<String, String>, _env| {
		format!("Unique chatters: {}", chatters.len())
	})
	.lens(Overview::unique_chatters);

	let total_messages = Label::dynamic(|num: &usize, _env| format!("Total messages: {num}"))
		.lens(Overview::total_messages);

	let subscriber_messages = Label::dynamic(|data: &Overview, _env| {
		format!(
			"Fraction of messages by subscribers: {:.2}%",
			100.0 * data.subscriber_messages as f64 / data.total_messages as f64
		)
	});

	let total_bits =
		Label::dynamic(|bits: &u64, _env| format!("Total bits: {bits}")).lens(Overview::total_bits);

	let messages_cleared =
		Label::dynamic(|cleared: &usize, _env| format!("Messages deleted: {cleared}"))
			.lens(Overview::messages_cleared);

	Flex::column()
		.with_child(unique_chatters)
		.with_child(total_messages)
		.with_child(subscriber_messages)
		.with_child(total_bits)
		.with_child(messages_cleared)
		.lens(UIState::overview)
		.controller(MessageAnalytics::default())
		.expand()
}

/// Controller for calculating chat analytics.
#[derive(Debug, Default)]
struct MessageAnalytics {}

impl<W: Widget<UIState>> Controller<UIState, W> for MessageAnalytics {
	fn event(
		&mut self,
		child: &mut W,
		ctx: &mut druid::EventCtx,
		event: &druid::Event,
		data: &mut UIState,
		env: &druid::Env,
	) {
		if let druid::Event::Command(command) = event {
			if let Some(message) = command.get(Selector::<Arc<Message>>::new(NEW_CHAT_MESSAGE)) {
				// New message, update internal data to show.
				data.overview
					.unique_chatters
					.entry(message.author.id.clone())
					.or_insert(message.author.name.clone());
				data.overview.total_messages += 1;
				data.overview.total_bits += message.bits.unwrap_or_default();

				if message.subscriber {
					data.overview.subscriber_messages += 1;
				}
			} else if let Some(_cleared) =
				command.get(Selector::<Arc<ClearMessage>>::new(CLEAR_CHAT_MESSAGE))
			{
				data.overview.messages_cleared += 1;
			} else if command.get(Selector::<()>::new(SETTINGS_UPDATE)).is_some() {
				// Settings changed, reset all the data.
				data.overview.unique_chatters.clear();
				data.overview.total_messages = 0;
				data.overview.subscriber_messages = 0;
				data.overview.total_bits = 0;
				data.overview.messages_cleared = 0;
			}
		}

		child.event(ctx, event, data, env);
	}
}
