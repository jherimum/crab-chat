use tokio::sync::oneshot;

use super::{PeerError, PeerResult, command_bus::IntoPeerCommand};
use std::fmt::Debug;

#[derive(Debug)]
pub enum PeerCommand {
    SendMessage(Command<SendMessageCommand, ()>),
    Start(Command<StartCommand, ()>),
}

pub struct Command<C: Debug, R> {
    command: C,
    sender: oneshot::Sender<PeerResult<PeerCommandResponse<R>>>,
}

impl<C: Debug, R> Debug for Command<C, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Command")
            .field("command", &self.command)
            .finish()
    }
}

#[derive(Debug)]
pub struct SendMessageCommand {
    message: String,
}

impl IntoPeerCommand for SendMessageCommand {
    type Output = ();
    fn into_command(
        self,
        sender: oneshot::Sender<PeerResult<PeerCommandResponse<Self::Output>>>,
    ) -> PeerCommand {
        PeerCommand::SendMessage(Command {
            command: self,
            sender,
        })
    }
}

#[derive(Debug)]
pub struct StartCommand;

impl IntoPeerCommand for StartCommand {
    type Output = ();
    fn into_command(
        self,
        sender: oneshot::Sender<PeerResult<PeerCommandResponse<Self::Output>>>,
    ) -> PeerCommand {
        PeerCommand::Start(Command {
            command: self,
            sender,
        })
    }
}

pub struct PeerCommandResponse<R>(oneshot::Receiver<PeerResult<PeerCommandResponse<R>>>);

impl<R> PeerCommandResponse<R> {
    pub fn new(rx: oneshot::Receiver<PeerResult<PeerCommandResponse<R>>>) -> Self {
        Self(rx)
    }

    pub async fn result(self) -> PeerResult<PeerResult<PeerCommandResponse<R>>> {
        self.0.await.map_err(PeerError::from)
    }
}
