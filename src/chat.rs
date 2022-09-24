//! Chat handler part of the app.

use std::{
	fmt::Display,
	sync::Arc,
	thread::{self, JoinHandle},
};

use druid::{Data, ExtEventSink, Selector, Target};
use tokio::sync::oneshot::{self, error::TryRecvError};
use twitch_irc::{
	login::StaticLoginCredentials,
	message::{ClearMsgMessage, Emote, PrivmsgMessage, ServerMessage, TwitchUserBasics},
	ClientConfig, SecureTCPTransport, TwitchIRCClient,
};
use typed_builder::TypedBuilder;

use crate::ui::UIState;

/// Selector string for new chat messages' commands.
pub const NEW_CHAT_MESSAGE: Selector<Arc<Message>> = Selector::new("NEW_CHAT_MESSAGE");
/// Selector string for cleared chat messages' commands.
pub const CLEAR_CHAT_MESSAGE: Selector<Arc<ClearMessage>> = Selector::new("CLEAR_CHAT_MESSAGE");

/// Chat receiver spawner.
#[derive(TypedBuilder)]
pub struct ChatReceiver {
	/// The twitch channel to join to.
	channel: String,
	/// The event sender to send data to the UI.
	event_sender: ExtEventSink,
	/// Trigger to stop the client and stop receiving messages.
	stop_trigger: oneshot::Receiver<()>,
}

impl ChatReceiver {
	/// Spawn the receiver in a new thread.
	pub fn spawn(self) -> JoinHandle<()> {
		thread::spawn(move || {
			if self.channel.is_empty() {
				return;
			}

			let runtime = tokio::runtime::Builder::new_current_thread()
				.enable_all()
				.build()
				.expect("building tokio runtime");
			runtime.block_on(self.run());
		})
	}

	/// Run the receiver
	async fn run(mut self) {
		let (mut messages, client) =
			TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(
				ClientConfig::default(),
			);

		client.join(self.channel.clone()).expect("joining channel");

		while let Some(message) = messages.recv().await {
			match self.stop_trigger.try_recv() {
				Err(TryRecvError::Empty) => {}
				_ => break,
			}

			match message {
				ServerMessage::Privmsg(priv_msg) => self.receive_priv_msg(priv_msg),
				ServerMessage::ClearMsg(clear_msg) => self.receive_clear_msg(clear_msg),
				_ => {}
			}
		}
	}

	/// Handle a PrivMsg message.
	fn receive_priv_msg(&self, priv_msg: PrivmsgMessage) {
		let message = Message::from(priv_msg);

		let cloned_message = message.clone();
		self.event_sender.add_idle_callback(move |data: &mut UIState| {
			data.chat.messages.push_front(cloned_message);
			if data.chat.messages.len() > data.settings.chat_buffer {
				data.chat.messages.truncate(data.settings.chat_buffer);
			}
		});
		self.event_sender
			.submit_command(NEW_CHAT_MESSAGE, Arc::new(message), Target::Auto)
			.expect("sending new message as command");
	}

	/// Handle a ClearMsg message.
	fn receive_clear_msg(&self, clear_msg: ClearMsgMessage) {
		let message = ClearMessage::from(clear_msg);

		self.event_sender
			.submit_command(CLEAR_CHAT_MESSAGE, Arc::new(message), Target::Auto)
			.expect("sending clear message as command");
	}
}

/// A message in the chat.
#[derive(Debug, Clone, PartialEq, Data)]
pub struct Message {
	/// Unique ID of the message
	pub id: String,
	/// Timestamp when the message was sent.
	#[data(ignore)]
	pub timestamp: i64,
	/// Sender of the message.
	#[data(ignore)]
	pub author: TwitchUserBasics,
	/// Message text.
	#[data(ignore)]
	pub message: String,
	/// List of emotes used.
	#[data(ignore)]
	pub emotes: Vec<Emote>,
	/// Number of bits cheered in this message (if any).
	#[data(ignore)]
	pub bits: Option<u64>,
	/// Whether the message had a subscriber badge.
	#[data(ignore)]
	pub subscriber: bool,
}

impl From<PrivmsgMessage> for Message {
	fn from(msg: PrivmsgMessage) -> Self {
		let subscriber = msg.badge_info.iter().any(|info| info.name.as_str() == "subscriber");

		Self {
			id: msg.message_id,
			timestamp: msg.server_timestamp.timestamp(),
			author: msg.sender,
			message: msg.message_text,
			emotes: msg.emotes,
			bits: msg.bits,
			subscriber,
		}
	}
}

impl Display for Message {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{}: {}", self.author.name, self.message))
	}
}

/// Message to clear a message.
pub struct ClearMessage {
	/// Message ID of the message to clear.
	pub id: String,
	/// Message author's login name of the message.
	pub author: String,
	/// Message text of the message to clear.
	pub message: String,
}

impl From<ClearMsgMessage> for ClearMessage {
	fn from(clear_msg: ClearMsgMessage) -> Self {
		Self {
			id: clear_msg.message_id,
			author: clear_msg.sender_login,
			message: clear_msg.message_text,
		}
	}
}
