use chrono;
use tokio::sync::mpsc::UnboundedSender;
use tokio_util::sync::CancellationToken;

pub struct Log {
    pub source_name: String,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Local>,
}

pub trait AsyncLogMonitor {
    fn get_common_name(&self) -> String;
    async fn monitor(
        &mut self,
        cancel_token: CancellationToken,
        sender_queue: UnboundedSender<Log>,
    );
}
