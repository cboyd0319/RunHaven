from __future__ import annotations

import json
import os
import subprocess
import sys
import threading
import uuid
from collections.abc import Callable, Sequence
from dataclasses import dataclass

from .active_records import (
    active_run_terminal_status,
    remove_active_run_record,
    write_active_run_record,
)
from .auth_broker import (
    CODEX_BROKER_PLACEHOLDER_ENV,
    CODEX_BROKER_PLACEHOLDER_VALUE,
    CODEX_BROKER_PROVIDER_ID,
    BrokerDecision,
    CodexApiKeyBrokerProxy,
)
from .egress import EgressPolicy, ProxyDecision, ThreadedAllowlistProxy
from .git_metadata import capture_git_snapshot, summarize_git_change
from .plans import AgentRunPlan
from .provider_observability import (
    print_provider_blocked_host_review,
    utc_timestamp,
    write_auth_broker_log,
    write_provider_policy_log,
)
from .run_history import write_run_record


@dataclass(frozen=True)
class InternalNetworkInfo:
    ipv4_gateway: str
    ipv4_subnet: str


ContainerCaller = Callable[[tuple[str, ...]], int]
NetworkDeleter = Callable[[str], int]
NetworkInspector = Callable[[str], InternalNetworkInfo]
PreflightRunner = Callable[[tuple[str, ...]], None]
ProviderProxyFactory = Callable[[EgressPolicy, InternalNetworkInfo], ThreadedAllowlistProxy]
CodexBrokerFactory = Callable[[str, InternalNetworkInfo], CodexApiKeyBrokerProxy]
ThreadFactory = Callable[..., threading.Thread]


@dataclass(frozen=True)
class ProviderRuntimeDependencies:
    run_preflight: PreflightRunner
    inspect_internal_network: NetworkInspector
    create_provider_proxy: ProviderProxyFactory
    create_codex_api_key_broker: CodexBrokerFactory
    thread_factory: ThreadFactory
    call_container: ContainerCaller
    delete_container_network: NetworkDeleter


def run_provider_agent(plan: AgentRunPlan, *, deps: ProviderRuntimeDependencies) -> int:
    if not plan.network_name:
        raise ValueError("provider network plan is missing an internal network")
    if not plan.provider_allowed_hosts:
        raise ValueError("provider network plan is missing provider hosts")

    codex_api_key = require_codex_api_key_broker_secret(plan)
    provider_network_created = False
    proxy: ThreadedAllowlistProxy | None = None
    proxy_thread: threading.Thread | None = None
    codex_broker: CodexApiKeyBrokerProxy | None = None
    codex_broker_thread: threading.Thread | None = None
    run_id = plan.run_id or uuid.uuid4().hex
    started_at: str | None = None
    finished_at: str | None = None
    return_code: int | None = None
    provider_decisions: tuple[ProxyDecision, ...] = ()
    auth_decisions: tuple[BrokerDecision, ...] | None = None
    git: dict[str, object] | None = None
    terminal_status: str | None = None
    active_run_recorded = False
    cleanup: dict[str, object] = {
        "provider_network": "not-created",
        "provider_network_name": plan.network_name,
    }
    try:
        for command in plan.preflight:
            deps.run_preflight(command)
            if command[:4] == ("container", "network", "create", "--internal"):
                provider_network_created = (
                    provider_network_created or command[-1] == plan.network_name
                )

        network_info = deps.inspect_internal_network(plan.network_name)
        policy = EgressPolicy(plan.provider_allowed_hosts)
        proxy = deps.create_provider_proxy(policy, network_info)
        worker = deps.thread_factory(target=proxy.serve_forever, daemon=True)
        worker.start()
        proxy_thread = worker
        proxy_url = f"http://{network_info.ipv4_gateway}:{proxy.server_address[1]}"

        if codex_api_key is None:
            command = with_provider_proxy_environment(plan, proxy_url)
        else:
            codex_broker = deps.create_codex_api_key_broker(codex_api_key, network_info)
            broker_worker = deps.thread_factory(
                target=codex_broker.serve_forever,
                daemon=True,
            )
            broker_worker.start()
            codex_broker_thread = broker_worker
            broker_url = (
                f"http://{network_info.ipv4_gateway}:"
                f"{codex_broker.server_address[1]}/v1"
            )
            command = with_provider_proxy_environment(
                plan,
                proxy_url,
                no_proxy_hosts=(network_info.ipv4_gateway,),
            )
            command = with_codex_api_key_broker_config(command, plan, broker_url)

        git_before = capture_git_snapshot(plan.workspace)
        started_at = utc_timestamp()
        print(f"Run id: {run_id}", file=sys.stderr)
        write_active_run_record(plan, run_id=run_id, started_at=started_at)
        active_run_recorded = True
        try:
            return_code = deps.call_container(command)
        finally:
            terminal_status = active_run_terminal_status(run_id)
        finished_at = utc_timestamp()
        git = summarize_git_change(git_before, capture_git_snapshot(plan.workspace))
        provider_decisions = tuple(proxy.policy_decisions())
        write_provider_policy_log(plan, provider_decisions, run_id=run_id)
        if codex_broker is not None:
            auth_decisions = tuple(codex_broker.broker_decisions())
            write_auth_broker_log(
                plan,
                auth_decisions,
                run_id=run_id,
                return_code=return_code,
            )
        print_provider_blocked_host_review(plan, provider_decisions, run_id=run_id)
        return return_code
    finally:
        if codex_broker is not None:
            if codex_broker_thread is not None:
                codex_broker.shutdown()
            codex_broker.server_close()
        if codex_broker_thread is not None:
            codex_broker_thread.join(timeout=5)
        if proxy is not None:
            if proxy_thread is not None:
                proxy.shutdown()
            proxy.server_close()
        if proxy_thread is not None:
            proxy_thread.join(timeout=5)
        if provider_network_created:
            cleanup = cleanup_provider_network(
                plan,
                delete_network=deps.delete_container_network,
            )
        if return_code is not None and started_at is not None and finished_at is not None:
            write_run_record(
                plan,
                run_id=run_id,
                started_at=started_at,
                finished_at=finished_at,
                return_code=return_code,
                status=terminal_status,
                provider_decisions=provider_decisions,
                auth_decisions=auth_decisions,
                cleanup=cleanup,
                git=git or {"available": False, "reason": "git-snapshot-missing"},
            )
        if active_run_recorded:
            remove_active_run_record(run_id)


def require_codex_api_key_broker_secret(plan: AgentRunPlan) -> str | None:
    if plan.codex_api_key_broker_env is None:
        return None
    value = os.environ.get(plan.codex_api_key_broker_env)
    if value is None or not value.strip():
        raise ValueError(
            f"{plan.codex_api_key_broker_env} is not set on the host; export it before "
            "using --codex-api-key-broker-env"
        )
    return value


def validate_runtime_auth_broker_environment(plan: AgentRunPlan) -> None:
    require_codex_api_key_broker_secret(plan)


def with_provider_proxy_environment(
    plan: AgentRunPlan,
    proxy_url: str,
    *,
    no_proxy_hosts: Sequence[str] = (),
) -> tuple[str, ...]:
    image_index = plan.command.index(plan.image)
    no_proxy = ",".join(("localhost", "127.0.0.1", "::1", *no_proxy_hosts))
    proxy_environment = (
        ("HTTPS_PROXY", proxy_url),
        ("HTTP_PROXY", proxy_url),
        ("ALL_PROXY", proxy_url),
        ("https_proxy", proxy_url),
        ("http_proxy", proxy_url),
        ("all_proxy", proxy_url),
        ("NO_PROXY", no_proxy),
        ("no_proxy", no_proxy),
    )
    injected: list[str] = []
    for name, value in proxy_environment:
        injected.extend(("--env", f"{name}={value}"))
    return (*plan.command[:image_index], *injected, *plan.command[image_index:])


def with_codex_api_key_broker_config(
    command: tuple[str, ...],
    plan: AgentRunPlan,
    broker_base_url: str,
) -> tuple[str, ...]:
    image_index = command.index(plan.image)
    if image_index + 1 >= len(command) or command[image_index + 1] != "codex":
        raise ValueError("Codex API key broker requires the agent command to start with codex")
    broker_environment = (
        "--env",
        f"{CODEX_BROKER_PLACEHOLDER_ENV}={CODEX_BROKER_PLACEHOLDER_VALUE}",
    )
    command_with_env = (*command[:image_index], *broker_environment, *command[image_index:])
    codex_index = image_index + len(broker_environment) + 1
    config = (
        "-c",
        f'model_provider="{CODEX_BROKER_PROVIDER_ID}"',
        "-c",
        f'model_providers.{CODEX_BROKER_PROVIDER_ID}.name='
        '"RunHaven OpenAI API-key broker"',
        "-c",
        f'model_providers.{CODEX_BROKER_PROVIDER_ID}.base_url="{broker_base_url}"',
        "-c",
        f'model_providers.{CODEX_BROKER_PROVIDER_ID}.env_key='
        f'"{CODEX_BROKER_PLACEHOLDER_ENV}"',
        "-c",
        f'model_providers.{CODEX_BROKER_PROVIDER_ID}.wire_api="responses"',
    )
    return (
        *command_with_env[: codex_index + 1],
        *config,
        *command_with_env[codex_index + 1 :],
    )


def cleanup_provider_network(
    plan: AgentRunPlan,
    *,
    delete_network: NetworkDeleter = lambda name: delete_container_network(name),
) -> dict[str, object]:
    if plan.network_name is None:
        return {"provider_network": "not-created", "provider_network_name": None}
    result: object = delete_network(plan.network_name)
    status = "deleted" if result == 0 else "delete-failed"
    cleanup: dict[str, object] = {
        "provider_network": status,
        "provider_network_name": plan.network_name,
    }
    if isinstance(result, int):
        cleanup["delete_return_code"] = result
    return cleanup


def create_provider_proxy(
    policy: EgressPolicy,
    network_info: InternalNetworkInfo,
) -> ThreadedAllowlistProxy:
    try:
        return ThreadedAllowlistProxy(
            (network_info.ipv4_gateway, 0),
            policy,
            allowed_client_subnets=(network_info.ipv4_subnet,),
        )
    except OSError:
        return ThreadedAllowlistProxy(
            ("0.0.0.0", 0),
            policy,
            allowed_client_subnets=(network_info.ipv4_subnet,),
        )


def create_codex_api_key_broker(
    api_key: str,
    network_info: InternalNetworkInfo,
) -> CodexApiKeyBrokerProxy:
    try:
        return CodexApiKeyBrokerProxy(
            (network_info.ipv4_gateway, 0),
            api_key=api_key,
            allowed_client_subnets=(network_info.ipv4_subnet,),
        )
    except OSError:
        return CodexApiKeyBrokerProxy(
            ("0.0.0.0", 0),
            api_key=api_key,
            allowed_client_subnets=(network_info.ipv4_subnet,),
        )


def inspect_internal_network(name: str) -> InternalNetworkInfo:
    result = subprocess.run(
        ("container", "network", "inspect", name),
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        raise SystemExit(result.returncode)
    try:
        payload = json.loads(result.stdout)
    except json.JSONDecodeError as exc:
        raise ValueError(f"could not inspect provider network {name!r}") from exc
    if not isinstance(payload, list) or not payload or not isinstance(payload[0], dict):
        raise ValueError(f"could not inspect provider network {name!r}")

    item = payload[0]
    configuration = item.get("configuration")
    if not isinstance(configuration, dict) or configuration.get("mode") != "hostOnly":
        raise ValueError(f"provider network {name!r} is not host-only")
    status = item.get("status")
    if not isinstance(status, dict):
        raise ValueError(f"provider network {name!r} is missing status")
    gateway = status.get("ipv4Gateway")
    subnet = status.get("ipv4Subnet")
    if not isinstance(gateway, str) or not isinstance(subnet, str):
        raise ValueError(f"provider network {name!r} is missing IPv4 gateway or subnet")
    return InternalNetworkInfo(ipv4_gateway=gateway, ipv4_subnet=subnet)


def delete_container_network(name: str) -> int:
    return subprocess.run(("container", "network", "delete", name), check=False).returncode


def ensure_internal_network(name: str) -> None:
    existing = subprocess.run(
        ("container", "network", "inspect", name),
        check=False,
        capture_output=True,
        text=True,
    )
    if existing.returncode == 0:
        mode = inspect_network_mode(existing.stdout)
        if mode == "hostOnly":
            return
        raise ValueError(
            f"existing container network {name!r} is {mode or 'unknown'}, not host-only"
        )

    created = subprocess.run(("container", "network", "create", "--internal", name), check=False)
    if created.returncode != 0:
        raise SystemExit(created.returncode)


def inspect_network_mode(output: str) -> str | None:
    try:
        payload = json.loads(output)
    except json.JSONDecodeError:
        return None
    if not isinstance(payload, list) or not payload:
        return None
    first = payload[0]
    if not isinstance(first, dict):
        return None
    configuration = first.get("configuration")
    if not isinstance(configuration, dict):
        return None
    mode = configuration.get("mode")
    return mode if isinstance(mode, str) else None
