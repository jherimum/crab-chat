use super::{PeerResult, command_bus::IntoPeerCommand};
use bon::Builder;
use derive_getters::Getters;
use libp2p::gossipsub::MessageId;
use std::fmt::Debug;
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum PeerCommand {
    SendMessage(Command<SendMessageCommand, MessageId>),
    Subscribe(Command<SubscribeCommand, bool>),
}

pub struct Command<C, R> {
    command: C,
    sender: oneshot::Sender<PeerResult<R>>,
}

impl<C, R> Command<C, R> {
    pub fn send(self, response: PeerResult<R>) {
        self.sender.send(response);
    }
}

impl<C, R> AsRef<C> for Command<C, R> {
    fn as_ref(&self) -> &C {
        &self.command
    }
}

impl<C: Debug, R> Debug for Command<C, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Command")
            .field("command", &self.command)
            .finish()
    }
}

#[derive(Debug, Getters, Builder)]
pub struct SendMessageCommand {
    message: String,
    topic: String,
    timestamp: u64,
}

impl IntoPeerCommand for SendMessageCommand {
    type Output = MessageId;
    fn into_command(self, sender: oneshot::Sender<PeerResult<Self::Output>>) -> PeerCommand {
        PeerCommand::SendMessage(Command {
            command: self,
            sender,
        })
    }
}

#[derive(Debug, Getters, Builder)]
pub struct SubscribeCommand {
    topic: String,
}

impl IntoPeerCommand for SubscribeCommand {
    type Output = bool;
    fn into_command(self, sender: oneshot::Sender<PeerResult<Self::Output>>) -> PeerCommand {
        PeerCommand::Subscribe(Command {
            command: self,
            sender,
        })
    }
}

pub struct PeerCommandResponse<R>(oneshot::Receiver<PeerResult<R>>);

impl<R> PeerCommandResponse<R> {
    pub fn new(rx: oneshot::Receiver<PeerResult<R>>) -> Self {
        Self(rx)
    }

    pub async fn result(self) -> PeerResult<R> {
        self.0.await.unwrap()
    }
}
