use super::{
    PeerResult,
    command::{PeerCommand, PeerCommandResponse},
};
use tokio::sync::{
    mpsc::UnboundedSender,
    oneshot::{self, Sender},
};

pub struct PeerCommandBus {
    sender: UnboundedSender<PeerCommand>,
}

impl PeerCommandBus {
    pub fn new(sender: UnboundedSender<PeerCommand>) -> Self {
        Self { sender }
    }

    pub async fn send<C: IntoPeerCommand>(
        &self,
        command: C,
    ) -> PeerResult<PeerCommandResponse<C::Output>> {
        let (tx, rx) = oneshot::channel();
        self.sender.send(command.into_command(tx)).unwrap();
        Ok(PeerCommandResponse::new(rx))
    }
}

pub trait IntoPeerCommand {
    type Output;
    fn into_command(self, tx: Sender<PeerResult<PeerCommandResponse<Self::Output>>>)
    -> PeerCommand;
}
