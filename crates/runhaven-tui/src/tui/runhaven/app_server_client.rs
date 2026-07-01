use std::io;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;

use serde::de::DeserializeOwned;
use serde_json::Value;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::time::timeout;

use super::protocol::AppServerEvent;
use super::protocol::ClientRequest;
use super::protocol::RequestId;
use super::protocol::ServerNotification;
use super::protocol::ServerRequest;
use super::service::RunHavenServiceError;
use super::service::RunHavenTuiService;

#[path = "app_server_client_error.rs"]
mod app_server_client_error;
#[path = "app_server_client_events.rs"]
mod app_server_client_events;

pub(crate) use app_server_client_error::TypedRequestError;
use app_server_client_error::service_error_to_typed;
use app_server_client_events::{EventForwarder, PendingServerRequests};

pub(crate) const DEFAULT_RUNHAVEN_APP_SERVER_CHANNEL_CAPACITY: usize = 32;

const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);

enum ClientCommand {
    Request {
        request: ClientRequest,
        response_tx: oneshot::Sender<Result<Value, RunHavenServiceError>>,
    },
    Shutdown {
        response_tx: oneshot::Sender<io::Result<()>>,
    },
    ResolveServerRequest {
        request_id: RequestId,
        response_tx: oneshot::Sender<io::Result<()>>,
    },
    RejectServerRequest {
        request_id: RequestId,
        response_tx: oneshot::Sender<io::Result<()>>,
    },
}

#[derive(Clone)]
pub(crate) struct AppServerRequestHandle {
    command_tx: mpsc::Sender<ClientCommand>,
    event_forwarder: EventForwarder,
}

pub(crate) enum AppServerClient {
    InProcess(RunHavenInProcessClient),
}

pub(crate) struct RunHavenInProcessClient {
    command_tx: mpsc::Sender<ClientCommand>,
    event_rx: mpsc::Receiver<AppServerEvent>,
    event_forwarder: EventForwarder,
    worker_handle: tokio::task::JoinHandle<()>,
}

pub(crate) struct PendingTypedRequest<T> {
    request_id: RequestId,
    method: String,
    response_rx: oneshot::Receiver<Result<Value, RunHavenServiceError>>,
    _marker: PhantomData<T>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct CancelledRequest {
    pub(crate) request_id: RequestId,
    pub(crate) method: String,
}

impl AppServerClient {
    pub(crate) fn start_in_process(service: RunHavenTuiService) -> Self {
        Self::InProcess(RunHavenInProcessClient::start(
            service,
            DEFAULT_RUNHAVEN_APP_SERVER_CHANNEL_CAPACITY,
        ))
    }

    pub(crate) async fn request_typed<T>(
        &self,
        request: ClientRequest,
    ) -> Result<T, TypedRequestError>
    where
        T: DeserializeOwned + Send + 'static,
    {
        match self {
            Self::InProcess(client) => client.request_typed(request).await,
        }
    }

    pub(crate) async fn next_event(&mut self) -> Option<AppServerEvent> {
        match self {
            Self::InProcess(client) => client.next_event().await,
        }
    }

    pub(crate) async fn shutdown(self) -> io::Result<()> {
        match self {
            Self::InProcess(client) => client.shutdown().await,
        }
    }

    pub(crate) async fn resolve_server_request(
        &self,
        request_id: RequestId,
        result: Value,
    ) -> io::Result<()> {
        match self {
            Self::InProcess(client) => client.resolve_server_request(request_id, result).await,
        }
    }

    pub(crate) async fn reject_server_request(
        &self,
        request_id: RequestId,
        message: String,
    ) -> io::Result<()> {
        match self {
            Self::InProcess(client) => client.reject_server_request(request_id, message).await,
        }
    }

    pub(crate) fn request_handle(&self) -> AppServerRequestHandle {
        match self {
            Self::InProcess(client) => client.request_handle(),
        }
    }
}

impl RunHavenInProcessClient {
    pub(crate) fn start(service: RunHavenTuiService, channel_capacity: usize) -> Self {
        let channel_capacity = channel_capacity.max(1);
        let (command_tx, mut command_rx) = mpsc::channel::<ClientCommand>(channel_capacity);
        let (event_tx, event_rx) = mpsc::channel::<AppServerEvent>(channel_capacity);
        let event_forwarder = EventForwarder {
            event_tx,
            skipped_events: Arc::new(tokio::sync::Mutex::new(0)),
            pending_server_requests: PendingServerRequests::default(),
        };
        let worker_event_forwarder = event_forwarder.clone();

        let worker_handle = tokio::spawn(async move {
            while let Some(command) = command_rx.recv().await {
                match command {
                    ClientCommand::Request {
                        request,
                        response_tx,
                    } => {
                        let service = service.clone();
                        tokio::spawn(async move {
                            let result = service.handle_request(request).await;
                            let _ = response_tx.send(result);
                        });
                    }
                    ClientCommand::Shutdown { response_tx } => {
                        let _ = response_tx.send(Ok(()));
                        break;
                    }
                    ClientCommand::ResolveServerRequest {
                        request_id,
                        response_tx,
                    } => {
                        let result = worker_event_forwarder
                            .pending_server_requests
                            .complete(request_id)
                            .await;
                        let _ = response_tx.send(result);
                    }
                    ClientCommand::RejectServerRequest {
                        request_id,
                        response_tx,
                    } => {
                        let result = worker_event_forwarder
                            .pending_server_requests
                            .complete(request_id)
                            .await;
                        let _ = response_tx.send(result);
                    }
                }
            }
        });

        Self {
            command_tx,
            event_rx,
            event_forwarder,
            worker_handle,
        }
    }

    pub(crate) async fn request_typed<T>(
        &self,
        request: ClientRequest,
    ) -> Result<T, TypedRequestError>
    where
        T: DeserializeOwned + Send + 'static,
    {
        self.request_handle().request_typed(request).await
    }

    pub(crate) async fn next_event(&mut self) -> Option<AppServerEvent> {
        self.event_rx.recv().await
    }

    pub(crate) async fn shutdown(self) -> io::Result<()> {
        let Self {
            command_tx,
            event_rx,
            event_forwarder: _,
            worker_handle,
        } = self;
        drop(event_rx);

        let (response_tx, response_rx) = oneshot::channel();
        if command_tx
            .send(ClientCommand::Shutdown { response_tx })
            .await
            .is_ok()
            && let Ok(shutdown_result) = timeout(SHUTDOWN_TIMEOUT, response_rx).await
        {
            shutdown_result.map_err(|_| {
                io::Error::new(io::ErrorKind::BrokenPipe, "shutdown channel closed")
            })??;
        }

        let mut worker_handle = worker_handle;
        if timeout(SHUTDOWN_TIMEOUT, &mut worker_handle).await.is_err() {
            worker_handle.abort();
            let _ = worker_handle.await;
        }
        Ok(())
    }

    pub(crate) async fn resolve_server_request(
        &self,
        request_id: RequestId,
        result: Value,
    ) -> io::Result<()> {
        self.request_handle()
            .resolve_server_request(request_id, result)
            .await
    }

    pub(crate) async fn reject_server_request(
        &self,
        request_id: RequestId,
        message: String,
    ) -> io::Result<()> {
        self.request_handle()
            .reject_server_request(request_id, message)
            .await
    }

    pub(crate) fn request_handle(&self) -> AppServerRequestHandle {
        AppServerRequestHandle {
            command_tx: self.command_tx.clone(),
            event_forwarder: self.event_forwarder.clone(),
        }
    }
}

impl AppServerRequestHandle {
    pub(crate) async fn request_typed<T>(
        &self,
        request: ClientRequest,
    ) -> Result<T, TypedRequestError>
    where
        T: DeserializeOwned + Send + 'static,
    {
        self.begin_request_typed(request).await?.response().await
    }

    pub(crate) async fn begin_request_typed<T>(
        &self,
        request: ClientRequest,
    ) -> Result<PendingTypedRequest<T>, TypedRequestError>
    where
        T: DeserializeOwned + Send + 'static,
    {
        let request_id = request.request_id();
        let method = request.method().to_string();
        let (response_tx, response_rx) = oneshot::channel();
        self.command_tx
            .send(ClientCommand::Request {
                request,
                response_tx,
            })
            .await
            .map_err(|_| TypedRequestError::Transport {
                method: method.clone(),
                message: "RunHaven TUI backend worker is closed".to_string(),
            })?;

        Ok(PendingTypedRequest {
            request_id,
            method,
            response_rx,
            _marker: PhantomData,
        })
    }

    pub(crate) async fn emit_notification(
        &self,
        notification: ServerNotification,
    ) -> io::Result<()> {
        self.event_forwarder
            .forward(AppServerEvent::ServerNotification(notification))
            .await
    }

    pub(crate) async fn emit_server_request(&self, request: ServerRequest) -> io::Result<()> {
        self.event_forwarder
            .forward(AppServerEvent::ServerRequest(request))
            .await
    }

    pub(crate) async fn resolve_server_request(
        &self,
        request_id: RequestId,
        _result: Value,
    ) -> io::Result<()> {
        let (response_tx, response_rx) = oneshot::channel();
        self.command_tx
            .send(ClientCommand::ResolveServerRequest {
                request_id,
                response_tx,
            })
            .await
            .map_err(closed_worker_channel)?;
        response_rx.await.map_err(closed_worker_channel)?
    }

    pub(crate) async fn reject_server_request(
        &self,
        request_id: RequestId,
        _message: String,
    ) -> io::Result<()> {
        let (response_tx, response_rx) = oneshot::channel();
        self.command_tx
            .send(ClientCommand::RejectServerRequest {
                request_id,
                response_tx,
            })
            .await
            .map_err(closed_worker_channel)?;
        response_rx.await.map_err(closed_worker_channel)?
    }
}

impl<T> PendingTypedRequest<T>
where
    T: DeserializeOwned,
{
    pub(crate) async fn response(self) -> Result<T, TypedRequestError> {
        let method = self.method;
        let value = self
            .response_rx
            .await
            .map_err(|_| TypedRequestError::Transport {
                method: method.clone(),
                message: "RunHaven TUI backend response channel is closed".to_string(),
            })?
            .map_err(|error| service_error_to_typed(method.clone(), error))?;

        serde_json::from_value(value)
            .map_err(|source| TypedRequestError::Deserialize { method, source })
    }

    pub(crate) fn cancel(self) -> CancelledRequest {
        CancelledRequest {
            request_id: self.request_id,
            method: self.method,
        }
    }
}

fn closed_worker_channel<T>(_: T) -> io::Error {
    io::Error::new(
        io::ErrorKind::BrokenPipe,
        "RunHaven TUI backend worker channel is closed",
    )
}

#[cfg(test)]
#[path = "app_server_client_tests.rs"]
mod tests;
