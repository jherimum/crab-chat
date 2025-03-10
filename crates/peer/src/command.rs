use super::PeerResult;
use bon::Builder;
use derive_getters::Getters;
use libp2p::gossipsub::MessageId;
use std::fmt::Debug;
use tokio::sync::{
    mpsc::UnboundedSender,
    oneshot::{self, Sender},
};

#[derive(Debug)]
pub enum PeerCommand {
    SendMessage(Command<SendMessageCommand, MessageId>),
    Subscribe(Command<SubscribeCommand, bool>),
    Unsubscribe(Command<UnsubscribeCommand, bool>),
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

#[derive(Debug, Getters, Builder)]
pub struct UnsubscribeCommand {
    topic: String,
}

impl IntoPeerCommand for UnsubscribeCommand {
    type Output = bool;
    fn into_command(self, sender: oneshot::Sender<PeerResult<Self::Output>>) -> PeerCommand {
        PeerCommand::Unsubscribe(Command {
            command: self,
            sender,
        })
    }
}

#[derive(Debug)]
pub struct PeerCommandBus {
    sender: UnboundedSender<PeerCommand>,
}

impl PeerCommandBus {
    pub fn new(sender: UnboundedSender<PeerCommand>) -> Self {
        Self { sender }
    }

    pub async fn send<C: IntoPeerCommand>(&self, command: C) -> PeerResult<C::Output> {
        let (tx, rx) = oneshot::channel();
        self.sender.send(command.into_command(tx))?;
        rx.await?
    }
}

pub trait IntoPeerCommand {
    type Output;
    fn into_command(self, tx: Sender<PeerResult<Self::Output>>) -> PeerCommand;
}
