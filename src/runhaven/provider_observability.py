from __future__ import annotations

import json
import sys
from collections.abc import Sequence
from datetime import UTC, datetime

from .auth_broker import BrokerDecision
from .cache_paths import auth_broker_log_path, egress_policy_log_path
from .egress import ProxyDecision, is_ip_literal
from .plans import AgentRunPlan
from .provider_endpoints import match_provider_endpoints


def utc_timestamp() -> str:
    return datetime.now(UTC).isoformat().replace("+00:00", "Z")


def write_provider_policy_log(
    plan: AgentRunPlan,
    decisions: Sequence[ProxyDecision],
    *,
    run_id: str,
) -> None:
    if not decisions:
        return
    log_path = egress_policy_log_path()
    log_path.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
    timestamp = utc_timestamp()
    with log_path.open("a", encoding="utf-8") as log_file:
        for decision in decisions:
            payload = {
                "timestamp": timestamp,
                "run_id": run_id,
                "profile": plan.profile_name,
                "workspace": str(plan.workspace),
                "network": plan.network_mode,
                "host": decision.host,
                "port": decision.port,
                "decision": decision.decision,
                "reason": decision.reason,
                "matched_rule": decision.matched_rule,
                "count": decision.count,
            }
            log_file.write(json.dumps(payload, sort_keys=True) + "\n")


def write_auth_broker_log(
    plan: AgentRunPlan,
    decisions: Sequence[BrokerDecision],
    *,
    run_id: str,
    return_code: int,
) -> None:
    log_path = auth_broker_log_path()
    log_path.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
    timestamp = utc_timestamp()
    entries = decisions or (
        BrokerDecision(
            method="-",
            path="-",
            decision="denied",
            reason="run-complete",
            upstream_status=None,
            count=0,
        ),
    )
    with log_path.open("a", encoding="utf-8") as log_file:
        for decision in entries:
            payload = {
                "timestamp": timestamp,
                "run_id": run_id,
                "profile": plan.profile_name,
                "workspace": str(plan.workspace),
                "network": plan.network_mode,
                "broker": "codex-api-key",
                "method": decision.method,
                "path": decision.path,
                "decision": "no-requests" if not decisions else decision.decision,
                "reason": decision.reason,
                "upstream_status": decision.upstream_status,
                "count": decision.count,
                "return_code": return_code,
            }
            log_file.write(json.dumps(payload, sort_keys=True) + "\n")


def print_provider_blocked_host_review(
    plan: AgentRunPlan,
    decisions: Sequence[ProxyDecision],
    *,
    run_id: str,
) -> None:
    denials = tuple(decision for decision in decisions if decision.decision == "denied")
    if not denials:
        return
    total = sum(decision.count for decision in denials)
    plural = "request" if total == 1 else "requests"
    print(
        f"RunHaven provider proxy blocked {total} CONNECT {plural} "
        f"across {len(denials)} target(s).",
        file=sys.stderr,
    )
    print(f"Run id: {run_id}", file=sys.stderr)
    print("Review:", file=sys.stderr)
    for decision in denials:
        target = f"{decision.host}:{decision.port}"
        matched_rule = decision.matched_rule or "-"
        print(
            f"  - {target}  count={decision.count}  reason={decision.reason}  "
            f"rule={matched_rule}",
            file=sys.stderr,
        )
        print(
            f"    Next action: {provider_denial_next_action(plan, decision)}",
            file=sys.stderr,
        )
    print("Recent policy log: runhaven egress log --limit 20", file=sys.stderr)


def provider_denial_next_action(plan: AgentRunPlan, decision: ProxyDecision) -> str:
    host = decision.host
    if is_ip_literal(host):
        return "IP literal targets cannot be allowed; use a reviewed provider hostname."
    if decision.reason == "port-not-allowed":
        return "provider mode only allows HTTPS CONNECT on port 443."
    if decision.reason == "unsafe-resolved-address":
        return "do not add an override; the allowed host resolved to a non-public address."
    if decision.reason == "dns-resolution-failed":
        return "check DNS or provider availability before changing the allowlist."
    explanation = f"runhaven why host {host} --agent {plan.profile_name}"
    if match_provider_endpoints(host, profile=plan.profile_name):
        return (
            f"{explanation}; rerun with --provider-host {host} only if the documented "
            "purpose matches."
        )
    return f"{explanation}; add --provider-host {host} only after source review."
