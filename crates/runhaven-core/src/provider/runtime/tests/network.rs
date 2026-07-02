//! Network-owned provider-runtime tests: internal-network inspect parsing and
//! the provider-network teardown choke point (`cleanup_provider_network`).

use super::broker_plan;
use crate::provider::runtime::network::{
    ContainerCli, ContainerOutcome, cleanup_provider_network_with_cli, parse_internal_network_info,
};

const NETWORK_INSPECT_HOSTONLY: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/apple_container/network-inspect-hostonly.json"
));

#[test]
fn parses_current_apple_network_inspect_shape() {
    let info =
        parse_internal_network_info("runhaven-volume-prep-internal", NETWORK_INSPECT_HOSTONLY)
            .expect("network inspect");

    assert_eq!(info.ipv4_gateway, "192.168.130.1");
    assert_eq!(info.ipv4_subnet, "192.168.130.0/24");
}

#[test]
fn rejects_non_host_only_network_inspect_shape() {
    let error = parse_internal_network_info(
        "runhaven-default",
        br#"[{"configuration":{"mode":"nat"},"status":{"ipv4Gateway":"192.168.64.1","ipv4Subnet":"192.168.64.0/24"}}]"#,
    )
    .expect_err("nat network");

    assert!(error.to_string().contains("not host-only"));
}

#[test]
fn rejects_network_inspect_missing_ipv4_fields() {
    let error = parse_internal_network_info(
        "runhaven-missing-ipv4",
        br#"[{"configuration":{"mode":"hostOnly"},"status":{}}]"#,
    )
    .expect_err("missing ipv4");

    assert!(error.to_string().contains("missing IPv4 gateway or subnet"));
}

/// Faithful stand-in for the Apple `container` teardown surface: a provider
/// network cannot be deleted while a container still references it, and
/// `rm --force` on an absent container reports "not found". Modeling that
/// coupling keeps the teardown-order fix a real red/green: without the
/// force-remove the network delete fails exactly as the live tauri:dev smoke
/// showed (`invalidState: cannot delete subnet ... with referring containers`).
struct RuntimeSim {
    container_present: bool,
    calls: Vec<Vec<String>>,
}

impl RuntimeSim {
    fn new(container_present: bool) -> Self {
        Self {
            container_present,
            calls: Vec::new(),
        }
    }
}

impl ContainerCli for RuntimeSim {
    fn run(&mut self, args: &[&str]) -> std::io::Result<ContainerOutcome> {
        self.calls
            .push(args.iter().map(|arg| arg.to_string()).collect());
        match args {
            ["rm", "--force", _name] => {
                if self.container_present {
                    self.container_present = false;
                    Ok(ContainerOutcome {
                        code: 0,
                        stderr: Vec::new(),
                    })
                } else {
                    Ok(ContainerOutcome {
                        code: 1,
                        stderr: br#"Error: internalError: "failed to delete container" (cause: "notFound: "container with ID runhaven-test-run not found"")"#.to_vec(),
                    })
                }
            }
            ["network", "delete", _name] => {
                if self.container_present {
                    Ok(ContainerOutcome {
                        code: 1,
                        stderr: br#"Error: invalidState: "cannot delete subnet runhaven-test-internal with referring containers: runhaven-test-run""#.to_vec(),
                    })
                } else {
                    Ok(ContainerOutcome {
                        code: 0,
                        stderr: Vec::new(),
                    })
                }
            }
            other => panic!("unexpected container subcommand: {other:?}"),
        }
    }
}

/// A container the runtime refuses to force-remove (service error, not
/// "not found"), that keeps referencing the network. Proves teardown still
/// attempts the delete and surfaces the anomaly instead of swallowing it.
struct StubbornSim {
    calls: Vec<Vec<String>>,
}

impl ContainerCli for StubbornSim {
    fn run(&mut self, args: &[&str]) -> std::io::Result<ContainerOutcome> {
        self.calls
            .push(args.iter().map(|arg| arg.to_string()).collect());
        match args {
            ["rm", "--force", _name] => Ok(ContainerOutcome {
                code: 1,
                stderr: br#"Error: internalError: "container service unavailable""#.to_vec(),
            }),
            ["network", "delete", _name] => Ok(ContainerOutcome {
                code: 1,
                stderr: br#"Error: invalidState: "cannot delete subnet runhaven-test-internal with referring containers: runhaven-test-run""#.to_vec(),
            }),
            other => panic!("unexpected container subcommand: {other:?}"),
        }
    }
}

#[test]
fn cleanup_force_removes_lingering_run_container_before_deleting_network() {
    // Leak scenario: the PTY client was killed mid-flow, so the `--rm` guest
    // container lingered and still references the provider network.
    let plan = broker_plan("claude", "claude");
    let mut cli = RuntimeSim::new(true);

    let cleanup = cleanup_provider_network_with_cli(&plan, &mut cli).expect("cleanup");

    // Order matters: force-remove the run container, THEN delete the network.
    assert_eq!(cli.calls.len(), 2);
    assert_eq!(cli.calls[0], ["rm", "--force", "runhaven-test-run"]);
    assert_eq!(
        cli.calls[1],
        ["network", "delete", "runhaven-test-internal"]
    );
    // With the container gone the delete now succeeds: no orphan, no leaked net.
    assert_eq!(cleanup["run_container"], "removed");
    assert_eq!(cleanup["provider_network"], "deleted");
    assert_eq!(cleanup["delete_return_code"], 0);
}

#[test]
fn cleanup_on_clean_run_treats_already_removed_container_as_success() {
    // Clean path: `container run --rm` already removed the guest on exit. The
    // force-remove must be a benign no-op, not a spurious teardown failure.
    let plan = broker_plan("claude", "claude");
    let mut cli = RuntimeSim::new(false);

    let cleanup = cleanup_provider_network_with_cli(&plan, &mut cli).expect("cleanup");

    assert_eq!(cli.calls.len(), 2);
    assert_eq!(cli.calls[0], ["rm", "--force", "runhaven-test-run"]);
    assert_eq!(
        cli.calls[1],
        ["network", "delete", "runhaven-test-internal"]
    );
    assert_eq!(cleanup["run_container"], "already-gone");
    assert_eq!(cleanup["provider_network"], "deleted");
    assert_eq!(cleanup["delete_return_code"], 0);
}

#[test]
fn cleanup_surfaces_delete_failure_when_container_cannot_be_removed() {
    // An unexpected remove failure must not abort teardown, and a network that
    // stays referenced is surfaced as delete-failed, never silently swallowed.
    let plan = broker_plan("claude", "claude");
    let mut cli = StubbornSim { calls: Vec::new() };

    let cleanup =
        cleanup_provider_network_with_cli(&plan, &mut cli).expect("structured teardown outcome");

    assert_eq!(
        cli.calls.len(),
        2,
        "delete is still attempted after a failed remove"
    );
    assert_eq!(cleanup["run_container"], "remove-failed");
    assert_eq!(cleanup["provider_network"], "delete-failed");
    assert_ne!(cleanup["delete_return_code"], 0);
}
