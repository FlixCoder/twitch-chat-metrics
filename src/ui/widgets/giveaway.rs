//! Giveaway UI.

use std::sync::Arc;

use druid::{
	im::{OrdMap, Vector},
	widget::{Button, Controller, Flex, Label, LineBreaking, List, Scroll, TextBox},
	Color, Data, Env, EventCtx, Insets, Lens, Selector, Widget, WidgetExt,
};
use rand::Rng;

use super::settings::SETTINGS_UPDATE;
use crate::{
	chat::{ClearMessage, Message, CLEAR_CHAT_MESSAGE, NEW_CHAT_MESSAGE},
	ui::UIState,
};

/// Giveaway UI data/state.
#[derive(Debug, Clone, Default, Data, Lens)]
pub struct Giveaway {
	/// Text to put in front of message to enter the giveaway.
	message_start: String,
	/// People who entered the giveaway as a map from ID to display name.
	#[data(same_fn = "fast_people_entered_same")]
	people_entered: OrdMap<String, String>,
	/// Winner
	winner: Option<String>,
	/// Winner's messages
	#[data(same_fn = "fast_winners_messages_same")]
	winners_messages: Vector<Message>,
}

/// Simple fast comparison function for entered people. Members are only added,
/// so compare the length only.
fn fast_people_entered_same(a: &OrdMap<String, String>, b: &OrdMap<String, String>) -> bool {
	a.len() == b.len()
}

/// Fast way to make sure two vectors of messages are not the same: check first
/// and last element for equality.
fn fast_winners_messages_same(a: &Vector<Message>, b: &Vector<Message>) -> bool {
	a.front() == b.front() && a.back() == b.back()
}

/// The UI widget.
#[must_use]
pub fn widget() -> impl Widget<UIState> {
	let chat_column = super::chat::widget().border(Color::GRAY, 1.0);

	let message_start = TextBox::new()
		.with_placeholder("<enter-command>")
		.expand_width()
		.lens(Giveaway::message_start);
	let people_entered =
		Scroll::new(List::new(|| Label::dynamic(|name: &String, _env| name.clone())))
			.vertical()
			.expand()
			.border(Color::GRAY, 1.0)
			.lens(Giveaway::people_entered);
	let clear = Button::new("Clear").on_click(on_clear);
	let draw_winner = Button::new("Draw winner").on_click(on_draw_winner);
	let give_away_column = Flex::column()
		.with_child(message_start)
		.with_default_spacer()
		.with_flex_child(people_entered, 9.0)
		.with_default_spacer()
		.with_child(clear)
		.with_child(draw_winner)
		.lens(UIState::giveaway);

	let winner = Label::dynamic(|data: &Giveaway, _env| {
		let winner = data
			.winner
			.as_ref()
			.and_then(|winner| data.people_entered.get(winner))
			.map(String::as_str)
			.unwrap_or_default();
		format!("Winner: {winner}")
	});
	let winners_messages = Scroll::new(List::new(|| {
		Label::dynamic(|msg: &Message, _env| format!("{}: {}", msg.author.name, msg.message))
			.with_line_break_mode(LineBreaking::WordWrap)
			.padding(Insets::uniform_xy(0.0, 2.0))
	}))
	.vertical()
	.expand()
	.border(Color::GRAY, 1.0)
	.lens(Giveaway::winners_messages);
	let winner_column = Flex::column()
		.with_child(winner)
		.with_flex_child(winners_messages, 9.0)
		.lens(UIState::giveaway);

	Flex::row()
		.with_flex_child(chat_column, 1.0)
		.with_default_spacer()
		.with_flex_child(give_away_column, 1.0)
		.with_default_spacer()
		.with_flex_child(winner_column, 1.0)
		.controller(MessageAnalytics::default())
}

/// On click of the "clear" button.
fn on_clear(_ctx: &mut EventCtx, data: &mut Giveaway, _env: &Env) {
	data.people_entered.clear();
	data.winner = None;
	data.winners_messages.clear();
}

/// On click of the "draw winner" button.
fn on_draw_winner(_ctx: &mut EventCtx, data: &mut Giveaway, _env: &Env) {
	let mut rng = rand::thread_rng();
	let winner = rng.gen_range(0..data.people_entered.len());
	data.winner = data.people_entered.keys().nth(winner).cloned();
}

/// Controller for handling chat messages for giveaways.
#[derive(Debug, Default)]
struct MessageAnalytics {}

impl<W: Widget<UIState>> Controller<UIState, W> for MessageAnalytics {
	fn event(
		&mut self,
		child: &mut W,
		ctx: &mut EventCtx,
		event: &druid::Event,
		data: &mut UIState,
		env: &Env,
	) {
		if let druid::Event::Command(command) = event {
			if let Some(message) = command.get(Selector::<Arc<Message>>::new(NEW_CHAT_MESSAGE)) {
				// New message, update internal data to show.
				if message.message.starts_with(&data.giveaway.message_start) {
					data.giveaway
						.people_entered
						.insert(message.author.id.clone(), message.author.name.clone());
				}
				if data
					.giveaway
					.winner
					.as_ref()
					.map_or(false, |winner| *winner == message.author.id)
				{
					data.giveaway.winners_messages.push_front(message.as_ref().clone());
					if data.giveaway.winners_messages.len() > 100 {
						data.giveaway.winners_messages.truncate(100);
					}
				}
			} else if let Some(_cleared) =
				command.get(Selector::<Arc<ClearMessage>>::new(CLEAR_CHAT_MESSAGE))
			{
				// Cleared message.
			} else if command.get(Selector::<()>::new(SETTINGS_UPDATE)).is_some() {
				// Settings changed, reset all the data.
				data.giveaway.people_entered.clear();
				data.giveaway.winner = None;
				data.giveaway.winners_messages.clear();
			}
		}

		child.event(ctx, event, data, env);
	}
}
