//! Chat UI tab.

use druid::{
	im::Vector,
	widget::{Label, LineBreaking, List, Scroll},
	Data, Insets, Lens, TextAlignment, Widget, WidgetExt,
};

use crate::{chat::Message, ui::UIState};

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
			.padding(Insets::uniform_xy(0.0, 2.0))
	})
	.lens(Chat::messages);

	Scroll::new(messages).vertical().expand().lens(UIState::chat)
}
