use std::io;
use std::sync::Arc;

use tokio::sync::mpsc;

use super::super::protocol::AppServerEvent;
use super::super::protocol::RequestId;
use super::super::protocol::ServerRequest;

#[derive(Clone)]
pub(super) struct EventForwarder {
    pub(super) event_tx: mpsc::Sender<AppServerEvent>,
    pub(super) skipped_events: Arc<tokio::sync::Mutex<usize>>,
    pub(super) pending_server_requests: PendingServerRequests,
}

#[derive(Clone, Default)]
pub(super) struct PendingServerRequests {
    pending: Arc<tokio::sync::Mutex<std::collections::HashSet<RequestId>>>,
}

impl EventForwarder {
    pub(super) async fn forward(&self, event: AppServerEvent) -> io::Result<()> {
        let server_request_id = match &event {
            AppServerEvent::ServerRequest(ServerRequest::Confirm { request_id, .. }) => {
                Some(*request_id)
            }
            AppServerEvent::Lagged { .. }
            | AppServerEvent::ServerNotification(_)
            | AppServerEvent::Disconnected { .. } => None,
        };
        if let Some(request_id) = server_request_id {
            self.pending_server_requests.register(request_id).await;
        }

        let mut skipped_events = self.skipped_events.lock().await;

        if *skipped_events > 0 {
            if event.requires_delivery() {
                if let Err(error) = self
                    .event_tx
                    .send(AppServerEvent::Lagged {
                        skipped: *skipped_events,
                    })
                    .await
                    .map_err(closed_event_channel)
                {
                    self.remove_registered_request(server_request_id).await;
                    return Err(error);
                }
                *skipped_events = 0;
            } else {
                match self.event_tx.try_send(AppServerEvent::Lagged {
                    skipped: *skipped_events,
                }) {
                    Ok(()) => *skipped_events = 0,
                    Err(mpsc::error::TrySendError::Full(_)) => {
                        *skipped_events = skipped_events.saturating_add(1);
                        return Ok(());
                    }
                    Err(mpsc::error::TrySendError::Closed(_)) => {
                        return Err(closed_event_channel(()));
                    }
                }
            }
        }

        if event.requires_delivery() {
            if let Err(error) = self
                .event_tx
                .send(event)
                .await
                .map_err(closed_event_channel)
            {
                self.remove_registered_request(server_request_id).await;
                return Err(error);
            }
            return Ok(());
        }

        match self.event_tx.try_send(event) {
            Ok(()) => Ok(()),
            Err(mpsc::error::TrySendError::Full(_)) => {
                *skipped_events = skipped_events.saturating_add(1);
                self.remove_registered_request(server_request_id).await;
                Ok(())
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                self.remove_registered_request(server_request_id).await;
                Err(closed_event_channel(()))
            }
        }
    }

    async fn remove_registered_request(&self, request_id: Option<RequestId>) {
        if let Some(request_id) = request_id {
            self.pending_server_requests.remove(request_id).await;
        }
    }
}

impl PendingServerRequests {
    async fn register(&self, request_id: RequestId) {
        self.pending.lock().await.insert(request_id);
    }

    async fn remove(&self, request_id: RequestId) {
        self.pending.lock().await.remove(&request_id);
    }

    pub(super) async fn complete(&self, request_id: RequestId) -> io::Result<()> {
        if self.pending.lock().await.remove(&request_id) {
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("server request {request_id} is not pending"),
            ))
        }
    }
}

fn closed_event_channel<T>(_: T) -> io::Error {
    io::Error::new(
        io::ErrorKind::BrokenPipe,
        "RunHaven TUI backend event channel is closed",
    )
}
