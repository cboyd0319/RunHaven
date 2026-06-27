use std::error::Error;
use std::fmt;
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
use super::protocol::UnsupportedFamily;
use super::service::RunHavenServiceError;
use super::service::RunHavenTuiService;

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

#[derive(Debug)]
pub(crate) enum TypedRequestError {
    Transport {
        method: String,
        message: String,
    },
    Unsupported {
        method: String,
        family: UnsupportedFamily,
    },
    Validation {
        method: String,
        message: String,
    },
    Backend {
        method: String,
        message: String,
    },
    Deserialize {
        method: String,
        source: serde_json::Error,
    },
}

impl fmt::Display for TypedRequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transport { method, message } => {
                write!(f, "{method} transport error: {message}")
            }
            Self::Unsupported { method, .. } => {
                write!(f, "{method} is not supported by the RunHaven TUI backend")
            }
            Self::Validation { method, message } => {
                write!(f, "{method} validation error: {message}")
            }
            Self::Backend { method, message } => {
                write!(f, "{method} backend error: {message}")
            }
            Self::Deserialize { method, source } => {
                write!(f, "{method} response decode error: {source}")
            }
        }
    }
}

impl Error for TypedRequestError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Deserialize { source, .. } => Some(source),
            Self::Transport { .. }
            | Self::Unsupported { .. }
            | Self::Validation { .. }
            | Self::Backend { .. } => None,
        }
    }
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

#[derive(Clone)]
struct EventForwarder {
    event_tx: mpsc::Sender<AppServerEvent>,
    skipped_events: Arc<tokio::sync::Mutex<usize>>,
    pending_server_requests: PendingServerRequests,
}

#[derive(Clone, Default)]
struct PendingServerRequests {
    pending: Arc<tokio::sync::Mutex<std::collections::HashSet<RequestId>>>,
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

impl EventForwarder {
    async fn forward(&self, event: AppServerEvent) -> io::Result<()> {
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

    async fn complete(&self, request_id: RequestId) -> io::Result<()> {
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

fn closed_worker_channel<T>(_: T) -> io::Error {
    io::Error::new(
        io::ErrorKind::BrokenPipe,
        "RunHaven TUI backend worker channel is closed",
    )
}

fn service_error_to_typed(method: String, error: RunHavenServiceError) -> TypedRequestError {
    match error {
        RunHavenServiceError::Unsupported { family, .. } => {
            TypedRequestError::Unsupported { method, family }
        }
        RunHavenServiceError::Validation { message, .. } => {
            TypedRequestError::Validation { method, message }
        }
        RunHavenServiceError::Backend { message, .. } => {
            TypedRequestError::Backend { method, message }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::runhaven::protocol::UnsupportedMethod;
    use crate::tui::runhaven::protocol::ValidateWorkspaceResponse;
    use runhaven_core::ui_contracts::AgentCatalogData;
    use serde::Deserialize;

    fn test_client(capacity: usize) -> RunHavenInProcessClient {
        RunHavenInProcessClient::start(RunHavenTuiService::new(), capacity)
    }

    #[tokio::test]
    async fn request_typed_returns_agent_catalog() {
        let client = test_client(4);

        let catalog: AgentCatalogData = client
            .request_typed(ClientRequest::RunHavenAgentList { request_id: 1 })
            .await
            .expect("agent catalog");

        assert!(
            catalog.agents.iter().any(|agent| agent.name == "codex"),
            "expected codex profile in catalog"
        );

        client.shutdown().await.expect("shutdown");
    }

    #[tokio::test]
    async fn request_handle_can_cancel_pending_request() {
        let client = test_client(4);
        let handle = client.request_handle();
        let pending = handle
            .begin_request_typed::<AgentCatalogData>(ClientRequest::RunHavenAgentList {
                request_id: 7,
            })
            .await
            .expect("pending request");

        let cancelled = pending.cancel();
        assert_eq!(
            cancelled,
            CancelledRequest {
                request_id: 7,
                method: "runhaven/agent/list".to_string(),
            }
        );

        let catalog: AgentCatalogData = handle
            .request_typed(ClientRequest::RunHavenAgentList { request_id: 8 })
            .await
            .expect("worker remains usable after cancellation");
        assert!(!catalog.agents.is_empty());

        client.shutdown().await.expect("shutdown");
    }

    #[tokio::test]
    async fn next_event_reads_server_notifications_and_requests() {
        let mut client = test_client(4);
        let handle = client.request_handle();

        handle
            .emit_notification(ServerNotification::TranscriptDelta {
                text: "hello".to_string(),
            })
            .await
            .expect("emit transcript");
        handle
            .emit_server_request(ServerRequest::Confirm {
                request_id: 9,
                prompt: "Launch?".to_string(),
            })
            .await
            .expect("emit request");

        assert_eq!(
            client.next_event().await,
            Some(AppServerEvent::ServerNotification(
                ServerNotification::TranscriptDelta {
                    text: "hello".to_string()
                }
            ))
        );
        assert_eq!(
            client.next_event().await,
            Some(AppServerEvent::ServerRequest(ServerRequest::Confirm {
                request_id: 9,
                prompt: "Launch?".to_string(),
            }))
        );

        client.shutdown().await.expect("shutdown");
    }

    #[tokio::test]
    async fn shutdown_closes_backend_without_pending_events() {
        let client = test_client(1);
        client.shutdown().await.expect("shutdown");
    }

    #[tokio::test]
    async fn wrapper_client_supports_request_events_and_server_request_resolution() {
        let mut client = AppServerClient::start_in_process(RunHavenTuiService::new());
        let handle = client.request_handle();

        let catalog: AgentCatalogData = client
            .request_typed(ClientRequest::RunHavenAgentList { request_id: 20 })
            .await
            .expect("agent catalog");
        assert!(!catalog.agents.is_empty());

        handle
            .emit_server_request(ServerRequest::Confirm {
                request_id: 21,
                prompt: "Launch?".to_string(),
            })
            .await
            .expect("emit server request");
        assert_eq!(
            client.next_event().await,
            Some(AppServerEvent::ServerRequest(ServerRequest::Confirm {
                request_id: 21,
                prompt: "Launch?".to_string(),
            }))
        );

        client
            .resolve_server_request(21, Value::Bool(true))
            .await
            .expect("resolve server request");
        let duplicate = client
            .reject_server_request(21, "already resolved".to_string())
            .await
            .expect_err("resolved request should not reject again");
        assert_eq!(duplicate.kind(), io::ErrorKind::NotFound);

        client.shutdown().await.expect("shutdown");
    }

    #[tokio::test]
    async fn lossless_notifications_are_delivered_in_order() {
        let mut client = test_client(8);
        let handle = client.request_handle();

        for notification in [
            ServerNotification::TranscriptDelta {
                text: "streamed text".to_string(),
            },
            ServerNotification::TurnCompleted {
                turn_id: "turn-1".to_string(),
            },
            ServerNotification::LaunchPrepared {
                plan_id: "plan-1".to_string(),
            },
        ] {
            handle
                .emit_notification(notification)
                .await
                .expect("emit notification");
        }

        assert_eq!(
            client.next_event().await,
            Some(AppServerEvent::ServerNotification(
                ServerNotification::TranscriptDelta {
                    text: "streamed text".to_string()
                }
            ))
        );
        assert_eq!(
            client.next_event().await,
            Some(AppServerEvent::ServerNotification(
                ServerNotification::TurnCompleted {
                    turn_id: "turn-1".to_string()
                }
            ))
        );
        assert_eq!(
            client.next_event().await,
            Some(AppServerEvent::ServerNotification(
                ServerNotification::LaunchPrepared {
                    plan_id: "plan-1".to_string()
                }
            ))
        );

        client.shutdown().await.expect("shutdown");
    }

    #[tokio::test]
    async fn best_effort_noise_drops_and_flushes_lag_before_lossless_event() {
        let mut client = test_client(1);
        let handle = client.request_handle();

        handle
            .emit_notification(ServerNotification::Progress {
                message: "queued".to_string(),
            })
            .await
            .expect("queue first progress");
        handle
            .emit_notification(ServerNotification::Progress {
                message: "dropped one".to_string(),
            })
            .await
            .expect("drop second progress");
        handle
            .emit_notification(ServerNotification::LogDelta {
                text: "dropped two".to_string(),
            })
            .await
            .expect("drop log noise");

        assert_eq!(
            client.next_event().await,
            Some(AppServerEvent::ServerNotification(
                ServerNotification::Progress {
                    message: "queued".to_string()
                }
            ))
        );

        let send_handle = handle.clone();
        let send_lossless = tokio::spawn(async move {
            send_handle
                .emit_notification(ServerNotification::TurnCompleted {
                    turn_id: "turn-2".to_string(),
                })
                .await
        });

        assert_eq!(
            client.next_event().await,
            Some(AppServerEvent::Lagged { skipped: 2 })
        );
        assert_eq!(
            client.next_event().await,
            Some(AppServerEvent::ServerNotification(
                ServerNotification::TurnCompleted {
                    turn_id: "turn-2".to_string()
                }
            ))
        );
        send_lossless
            .await
            .expect("join lossless sender")
            .expect("lossless sender");

        client.shutdown().await.expect("shutdown");
    }

    #[tokio::test]
    async fn unsupported_method_matrix_fails_closed() {
        let client = test_client(4);

        for method in UnsupportedMethod::ALL {
            let error = client
                .request_typed::<Value>(ClientRequest::Unsupported {
                    request_id: 11,
                    method: *method,
                })
                .await
                .expect_err("unsupported method should fail");

            match error {
                TypedRequestError::Unsupported {
                    method: actual,
                    family,
                } => {
                    assert_eq!(actual, method.method());
                    assert_eq!(family, method.family());
                }
                other => panic!("expected unsupported error for {method:?}, got {other:?}"),
            }
        }

        client.shutdown().await.expect("shutdown");
    }

    #[tokio::test]
    async fn request_typed_surfaces_validation_errors() {
        let root = tempfile::tempdir().expect("root");
        let missing = root.path().join("missing");
        let client = test_client(4);

        let error = client
            .request_typed::<ValidateWorkspaceResponse>(ClientRequest::RunHavenValidateWorkspace {
                request_id: 12,
                workspace: missing,
            })
            .await
            .expect_err("missing workspace should fail validation");

        assert!(matches!(error, TypedRequestError::Validation { .. }));

        client.shutdown().await.expect("shutdown");
    }

    #[tokio::test]
    async fn request_typed_surfaces_backend_failures() {
        let client = test_client(4);

        let error = client
            .request_typed::<Value>(ClientRequest::BackendFailureForTest {
                request_id: 13,
                message: "backend unavailable".to_string(),
            })
            .await
            .expect_err("test backend failure should fail");

        assert!(matches!(
            error,
            TypedRequestError::Backend { message, .. } if message == "backend unavailable"
        ));

        client.shutdown().await.expect("shutdown");
    }

    #[tokio::test]
    async fn request_typed_surfaces_deserialize_errors() {
        #[derive(Debug, Deserialize)]
        #[allow(dead_code)]
        struct WrongResponse {
            missing_field: String,
        }

        let client = test_client(4);

        let error = client
            .request_typed::<WrongResponse>(ClientRequest::RunHavenAgentList { request_id: 14 })
            .await
            .expect_err("wrong response type should fail decode");

        match error {
            TypedRequestError::Deserialize { method, source } => {
                assert_eq!(method, "runhaven/agent/list");
                assert!(source.to_string().contains("missing_field"));
            }
            other => panic!("expected deserialize error, got {other:?}"),
        }

        client.shutdown().await.expect("shutdown");
    }
}
