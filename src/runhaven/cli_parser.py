from __future__ import annotations

import argparse
from pathlib import Path

from .active_commands import DEFAULT_LOG_FOLLOW_LINES
from .plans import SUPPORTED_NETWORK_MODES, SUPPORTED_WORKSPACE_SCOPES
from .profiles import PROFILES


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="runhaven",
        description="Run AI coding agents inside Apple container on macOS.",
    )
    subcommands = parser.add_subparsers(dest="command")

    subcommands.add_parser("agents", help="list bundled agent profiles")
    subcommands.add_parser("doctor", help="check local runtime prerequisites")
    setup_parser = subcommands.add_parser(
        "setup",
        help="guide first-run prerequisites and next commands",
    )
    setup_parser.add_argument(
        "--agent",
        choices=sorted(PROFILES),
        default="claude",
        help="agent profile to prepare; defaults to claude",
    )

    agent_args_epilog = (
        "Use -- before flags meant for the agent, for example:\n"
        "  runhaven run claude -- --version"
    )
    plan_parser = subcommands.add_parser(
        "plan",
        help="print the Apple container run plan",
        epilog=agent_args_epilog,
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    add_run_arguments(plan_parser)

    run_parser = subcommands.add_parser(
        "run",
        help="run an agent through Apple container",
        epilog=agent_args_epilog,
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    add_run_arguments(run_parser)
    run_parser.add_argument(
        "--dry-run",
        action="store_true",
        help="print the plan instead of running",
    )
    run_parser.add_argument(
        "--worktree",
        action="store_true",
        help="create a RunHaven-owned git worktree for this run and mount it",
    )

    image_parser = subcommands.add_parser("image", help="manage local agent images")
    image_subcommands = image_parser.add_subparsers(dest="image_command", required=True)
    build_parser_ = image_subcommands.add_parser("build", help="build a bundled agent image")
    add_image_build_arguments(build_parser_)
    rebuild_parser = image_subcommands.add_parser(
        "rebuild",
        help="rebuild a bundled agent image",
    )
    add_image_build_arguments(rebuild_parser)

    network_parser = subcommands.add_parser(
        "network",
        help="inspect or remove RunHaven managed networks",
    )
    network_subcommands = network_parser.add_subparsers(dest="network_command", required=True)
    network_subcommands.add_parser(
        "list",
        help="list RunHaven managed Apple container networks",
    )
    network_prune_parser = network_subcommands.add_parser(
        "prune",
        help="remove RunHaven managed Apple container networks",
    )
    network_prune_parser.add_argument(
        "--yes",
        action="store_true",
        help="delete listed networks",
    )

    state_parser = subcommands.add_parser("state", help="inspect or remove RunHaven state volumes")
    state_subcommands = state_parser.add_subparsers(dest="state_command", required=True)
    state_list_parser = state_subcommands.add_parser(
        "list",
        help="list RunHaven agent home volumes",
    )
    state_list_parser.add_argument("--session", help="only list volumes for this named session")
    prune_parser = state_subcommands.add_parser("prune", help="remove RunHaven agent home volumes")
    prune_parser.add_argument("--session", help="only prune volumes for this named session")
    prune_parser.add_argument("--yes", action="store_true", help="delete listed volumes")
    reset_parser = state_subcommands.add_parser(
        "reset",
        help="delete one planned project/profile/session state volume",
    )
    reset_parser.add_argument("agent", choices=sorted(PROFILES), help="agent profile to reset")
    reset_parser.add_argument(
        "--workspace",
        type=Path,
        default=Path("."),
        help="host project directory whose state volume should be reset",
    )
    reset_parser.add_argument(
        "--workspace-scope",
        choices=SUPPORTED_WORKSPACE_SCOPES,
        default="current",
        help=(
            "current resets the selected directory state; git-root explicitly expands "
            "to the containing git repository root"
        ),
    )
    reset_parser.add_argument("--session", help="named session to reset")
    reset_parser.add_argument(
        "--allow-sensitive-workspace",
        action="store_true",
        help="allow resolving broad or credential-bearing host paths for reset targeting",
    )
    reset_parser.add_argument("--yes", action="store_true", help="delete the planned volume")

    runs_parser = subcommands.add_parser("runs", help="inspect RunHaven run history")
    runs_subcommands = runs_parser.add_subparsers(dest="runs_command", required=True)
    runs_list_parser = runs_subcommands.add_parser("list", help="show recent RunHaven runs")
    runs_list_parser.add_argument(
        "--limit",
        type=int,
        default=20,
        help="maximum entries to show; use 0 for all entries",
    )
    runs_list_parser.add_argument("--json", action="store_true", help="print JSON output")
    runs_show_parser = runs_subcommands.add_parser("show", help="show one RunHaven run record")
    runs_show_parser.add_argument("run_id", help="run id to show")
    runs_show_parser.add_argument("--json", action="store_true", help="print JSON output")
    runs_log_parser = runs_subcommands.add_parser(
        "log",
        help="show one run with related provider and auth events",
    )
    runs_log_parser.add_argument("run_id", help="run id to show")
    runs_log_parser.add_argument("--json", action="store_true", help="print JSON output")
    runs_diff_parser = runs_subcommands.add_parser(
        "diff",
        help="show live git diff for one RunHaven run",
    )
    runs_diff_parser.add_argument("run_id", help="run id to diff")
    runs_recover_parser = runs_subcommands.add_parser(
        "recover",
        help="print manual recovery steps for a RunHaven worktree run",
    )
    runs_recover_parser.add_argument("run_id", help="worktree run id to recover")
    runs_recover_parser.add_argument("--json", action="store_true", help="print JSON output")
    runs_merge_parser = runs_subcommands.add_parser(
        "merge",
        help="merge a RunHaven worktree run back into the source checkout",
    )
    runs_merge_parser.add_argument("run_id", help="worktree run id to merge")
    runs_keep_parser = runs_subcommands.add_parser(
        "keep",
        help="keep a RunHaven worktree run for manual review",
    )
    runs_keep_parser.add_argument("run_id", help="worktree run id to keep")
    runs_discard_parser = runs_subcommands.add_parser(
        "discard",
        help="discard a RunHaven worktree run and delete its branch",
    )
    runs_discard_parser.add_argument("run_id", help="worktree run id to discard")
    runs_active_parser = runs_subcommands.add_parser(
        "active",
        help="show currently active RunHaven runs",
    )
    runs_active_parser.add_argument("--json", action="store_true", help="print JSON output")
    runs_status_parser = runs_subcommands.add_parser(
        "status",
        help="show sanitized status for an active RunHaven run",
    )
    runs_status_parser.add_argument("run_id", help="active run id to inspect")
    runs_status_parser.add_argument("--json", action="store_true", help="print JSON output")
    runs_attach_parser = runs_subcommands.add_parser(
        "attach",
        help="open a shell or command in an active RunHaven run",
        epilog="Use -- before a custom command, for example: runhaven runs attach RUN_ID -- pwd",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    runs_attach_parser.add_argument("run_id", help="active run id to attach to")
    runs_attach_parser.add_argument(
        "--user",
        default="agent",
        help="container user for the attached process; defaults to non-root agent",
    )
    runs_attach_parser.add_argument(
        "--allow-root-user",
        action="store_true",
        help="allow attaching as root inside the active container",
    )
    runs_attach_parser.add_argument(
        "--workdir",
        default="/workspace",
        help="container working directory for the attached process",
    )
    runs_attach_parser.add_argument(
        "--tty",
        choices=("auto", "always", "never"),
        default="auto",
        help="allocate a TTY for the attached process; auto follows the current terminal",
    )
    runs_logs_follow_parser = runs_subcommands.add_parser(
        "logs-follow",
        help="follow logs from an active RunHaven run",
    )
    runs_logs_follow_parser.add_argument("run_id", help="active run id to follow")
    runs_logs_follow_parser.add_argument(
        "--lines",
        type=int,
        default=DEFAULT_LOG_FOLLOW_LINES,
        help="recent log lines to show before following",
    )
    runs_stop_parser = runs_subcommands.add_parser(
        "stop",
        help="stop an active RunHaven run",
    )
    runs_stop_parser.add_argument("run_id", help="active run id to stop")
    runs_kill_parser = runs_subcommands.add_parser(
        "kill",
        help="hard-stop an active RunHaven run",
    )
    runs_kill_parser.add_argument("run_id", help="active run id to kill")
    runs_repair_parser = runs_subcommands.add_parser(
        "repair",
        help="remove a stale active marker after confirming its container is gone",
    )
    runs_repair_parser.add_argument("run_id", nargs="?", help="active run id to repair")
    runs_repair_parser.add_argument(
        "--all",
        action="store_true",
        help="inspect all active markers and remove only confirmed-stale markers",
    )
    runs_repair_parser.add_argument("--json", action="store_true", help="print JSON output")

    egress_parser = subcommands.add_parser("egress", help="inspect provider egress policy logs")
    egress_subcommands = egress_parser.add_subparsers(dest="egress_command", required=True)
    egress_log_parser = egress_subcommands.add_parser(
        "log",
        help="show recent provider proxy policy decisions",
    )
    egress_log_parser.add_argument(
        "--limit",
        type=int,
        default=20,
        help="maximum entries to show; use 0 for all entries",
    )
    egress_log_parser.add_argument("--json", action="store_true", help="print JSON output")

    auth_parser = subcommands.add_parser("auth", help="inspect provider auth broker status")
    auth_subcommands = auth_parser.add_subparsers(dest="auth_command", required=True)
    auth_status_parser = auth_subcommands.add_parser(
        "status",
        help="show auth broker status without reading secrets",
    )
    auth_status_parser.add_argument("--json", action="store_true", help="print JSON output")
    auth_explain_parser = auth_subcommands.add_parser(
        "explain",
        help="explain the auth broker boundary for an agent",
    )
    auth_explain_parser.add_argument("agent", choices=sorted(PROFILES), help="agent profile")
    auth_explain_parser.add_argument("--json", action="store_true", help="print JSON output")
    auth_log_parser = auth_subcommands.add_parser(
        "log",
        help="show recent auth broker decisions without secrets",
    )
    auth_log_parser.add_argument(
        "--limit",
        type=int,
        default=20,
        help="maximum entries to show; use 0 for all entries",
    )
    auth_log_parser.add_argument("--json", action="store_true", help="print JSON output")

    why_parser = subcommands.add_parser("why", help="explain RunHaven safety decisions")
    why_subcommands = why_parser.add_subparsers(dest="why_command", required=True)
    why_host_parser = why_subcommands.add_parser(
        "host",
        help="explain provider-host allowlist behavior",
    )
    why_host_parser.add_argument("host", help="host to explain")
    why_host_parser.add_argument("--port", type=int, default=443, help="provider port to check")
    why_host_parser.add_argument(
        "--agent",
        choices=sorted(PROFILES),
        help="agent profile whose bundled provider hosts should be checked",
    )

    return parser


def add_image_build_arguments(parser: argparse.ArgumentParser) -> None:
    parser.add_argument(
        "agent",
        choices=sorted(PROFILES),
        help="agent image template to build",
    )
    parser.add_argument("--tag", help="override the image tag")
    parser.add_argument("--dry-run", action="store_true", help="print the build command")


def add_run_arguments(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("agent", choices=sorted(PROFILES), help="agent profile to run")
    parser.add_argument(
        "--workspace",
        type=Path,
        default=Path("."),
        help="host project directory to mount at /workspace",
    )
    parser.add_argument(
        "--workspace-scope",
        choices=SUPPORTED_WORKSPACE_SCOPES,
        default="current",
        help=(
            "current mounts the selected directory; git-root explicitly expands "
            "to the containing git repository root"
        ),
    )
    parser.add_argument(
        "--session",
        help="named reusable project session for the agent home volume",
    )
    parser.add_argument("--image", help="override the profile image")
    parser.add_argument("--cpus", default="4", help="virtual CPUs for the container")
    parser.add_argument("--memory", default="4g", help="memory limit for the container")
    parser.add_argument(
        "--tty",
        choices=("auto", "always", "never"),
        default="auto",
        help="allocate a container TTY; auto follows the current terminal",
    )
    parser.add_argument(
        "--no-interactive",
        action="store_true",
        help="do not keep container standard input open",
    )
    parser.add_argument(
        "--network",
        choices=SUPPORTED_NETWORK_MODES,
        default="internet",
        help=(
            "internet uses unrestricted default networking; internal creates a host-only "
            "network; provider routes through a runtime allowlist proxy"
        ),
    )
    parser.add_argument(
        "--provider-host",
        action="append",
        default=[],
        metavar="HOST",
        help="additional fully qualified HTTPS host allowed by --network provider",
    )
    parser.add_argument(
        "--codex-api-key-broker-env",
        metavar="NAME",
        help=(
            "Codex-only: read this host environment variable at run time and broker "
            "OpenAI Responses API requests without placing the raw value in the guest"
        ),
    )
    parser.add_argument(
        "--read-only-workspace",
        action="store_true",
        help="mount the workspace read-only so the agent can inspect but not edit it",
    )
    parser.add_argument(
        "--ssh",
        action="store_true",
        help="forward the host SSH agent socket without mounting raw SSH keys",
    )
    parser.add_argument(
        "--env",
        action="append",
        default=[],
        metavar="NAME",
        help="inherit a single host environment variable by name",
    )
    parser.add_argument(
        "--user",
        default="agent",
        help="container user to run as; bundled images provide the non-root agent user",
    )
    parser.add_argument(
        "--allow-sensitive-workspace",
        action="store_true",
        help="allow mounting broad or credential-bearing host paths",
    )
    parser.add_argument(
        "--allow-root-user",
        action="store_true",
        help="allow running the agent process as root inside the container",
    )
