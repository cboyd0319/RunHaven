from __future__ import annotations

import json
from collections.abc import Sequence
from typing import Any

from .auth_broker import (
    AUTH_BROKER_RUNTIME,
    AUTH_BROKER_STATUS,
    auth_broker_profiles,
    get_auth_broker_profile,
)
from .cache_paths import auth_broker_log_path, egress_policy_log_path
from .egress import EgressPolicy, is_ip_literal, normalize_host
from .profiles import PROFILES, get_profile
from .provider_endpoints import ProviderEndpoint, match_provider_endpoints


def egress_log(*, limit: int, json_output: bool) -> int:
    if limit < 0:
        raise ValueError("--limit must be 0 or greater")
    entries = read_egress_policy_log(limit=limit)
    if json_output:
        print(json.dumps(entries, indent=2, sort_keys=True))
        return 0
    if not entries:
        print("No RunHaven provider egress policy log entries found.")
        return 0
    for entry in entries:
        host = entry.get("host", "<unknown>")
        port = entry.get("port", "?")
        decision = entry.get("decision", "unknown")
        reason = entry.get("reason", "unknown")
        count = entry.get("count", 1)
        profile = entry.get("profile", "unknown")
        run_id = entry.get("run_id", "-")
        matched_rule = entry.get("matched_rule") or "-"
        print(
            f"{entry.get('timestamp', '<unknown>')}  {profile}  {decision}  "
            f"{host}:{port}  count={count}  reason={reason}  rule={matched_rule}  "
            f"run={run_id}"
        )
    return 0


def read_egress_policy_log(*, limit: int) -> list[dict[str, Any]]:
    log_path = egress_policy_log_path()
    if not log_path.exists():
        return []
    entries: list[dict[str, Any]] = []
    for line in log_path.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        try:
            payload = json.loads(line)
        except json.JSONDecodeError:
            continue
        if isinstance(payload, dict):
            entries.append(payload)
    if limit == 0:
        return entries
    return entries[-limit:]


def auth_log(*, limit: int, json_output: bool) -> int:
    if limit < 0:
        raise ValueError("--limit must be 0 or greater")
    entries = read_auth_broker_log(limit=limit)
    if json_output:
        print(json.dumps(entries, indent=2, sort_keys=True))
        return 0
    if not entries:
        print("No RunHaven auth broker log entries found.")
        return 0
    for entry in entries:
        broker = entry.get("broker", "unknown")
        decision = entry.get("decision", "unknown")
        reason = entry.get("reason", "unknown")
        method = entry.get("method", "-")
        path = entry.get("path", "-")
        count = entry.get("count", 1)
        profile = entry.get("profile", "unknown")
        run_id = entry.get("run_id", "-")
        upstream_status = entry.get("upstream_status")
        status = upstream_status if upstream_status is not None else "-"
        print(
            f"{entry.get('timestamp', '<unknown>')}  {profile}  {broker}  "
            f"{decision}  {method} {path}  status={status}  count={count}  "
            f"reason={reason}  run={run_id}"
        )
    return 0


def read_auth_broker_log(*, limit: int) -> list[dict[str, Any]]:
    log_path = auth_broker_log_path()
    if not log_path.exists():
        return []
    entries: list[dict[str, Any]] = []
    for line in log_path.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        try:
            payload = json.loads(line)
        except json.JSONDecodeError:
            continue
        if isinstance(payload, dict):
            entries.append(payload)
    if limit == 0:
        return entries
    return entries[-limit:]


def auth_status(*, json_output: bool) -> int:
    profiles = auth_broker_profiles()
    payload = {
        "status": AUTH_BROKER_STATUS,
        "runtime": AUTH_BROKER_RUNTIME,
        "credential_stores_inspected": False,
        "environment_values_inspected": False,
        "secrets_printed": False,
        "profiles": [profile.to_json() for profile in profiles],
    }
    if json_output:
        print(json.dumps(payload, indent=2, sort_keys=True))
        return 0

    print(f"Auth broker: {AUTH_BROKER_STATUS}")
    print(f"Runtime: {AUTH_BROKER_RUNTIME}")
    print("Credential stores inspected: no")
    print("Environment values inspected: no")
    print("Secrets printed: no")
    print("Profiles:")
    width = max(len(profile.name) for profile in profiles)
    for profile in profiles:
        print(f"  {profile.name:<{width}}  {profile.status}")
    print("Current safe paths:")
    print("  - authenticate inside the isolated agent state volume when interactive")
    print("  - use the Codex API-key broker for headless Codex API-key runs")
    print("  - pass one token with --env NAME only when explicitly needed")
    print("  - use --network provider to constrain provider egress separately")
    return 0


def auth_explain(agent: str, *, json_output: bool) -> int:
    profile = get_profile(agent)
    auth_profile = get_auth_broker_profile(profile.name)
    payload = {
        **auth_profile.to_json(),
        "runtime": AUTH_BROKER_RUNTIME,
        "credential_stores_inspected": False,
        "environment_values_inspected": False,
        "secrets_printed": False,
        "provider_hosts": profile.provider_hosts,
    }
    if json_output:
        print(json.dumps(payload, indent=2, sort_keys=True))
        return 0

    print(f"Profile: {profile.name}")
    print(f"Auth broker: {auth_profile.status}")
    print(f"Runtime: {AUTH_BROKER_RUNTIME}")
    print("Credential stores inspected: no")
    print("Environment values inspected: no")
    print("Secrets printed: no")
    print("Supported auth surfaces:")
    for item in auth_profile.supported_auth:
        print(f"  - {item}")
    print("Host keeps:")
    for item in auth_profile.host_keeps:
        print(f"  - {item}")
    print("Guest receives:")
    for item in auth_profile.guest_receives:
        print(f"  - {item}")
    if profile.provider_hosts:
        print(f"Provider hosts: {', '.join(profile.provider_hosts)}")
    else:
        print("Provider hosts: none bundled")
    print(f"Current safe path: {auth_profile.current_safe_path}")
    if auth_profile.notes:
        print("Notes:")
        for note in auth_profile.notes:
            print(f"  - {note}")
    return 0


def why_host(host: str, *, port: int, agent: str | None) -> int:
    if port < 1 or port > 65535:
        raise ValueError("--port must be between 1 and 65535")
    normalized = normalize_host(host)
    print(f"Host: {normalized}")
    print(f"Port: {port}")
    if is_ip_literal(normalized):
        print("Provider mode: denied")
        print("Reason: IP literal targets cannot be allowed in provider mode.")
        print("Next action: use a reviewed fully qualified provider hostname instead.")
        return 0
    if "." not in normalized:
        print("Provider mode: denied")
        print("Reason: provider hosts must be fully qualified, not single-label names.")
        print("Next action: use a specific hostname such as api.example.com.")
        return 0

    if agent is not None:
        profile = get_profile(agent)
        print(f"Provider profile: {profile.name}")
        if not profile.provider_hosts:
            print("Provider mode: no bundled provider hosts are defined for this profile.")
            print("Next action: use --provider-host only after reviewing a fully qualified host.")
            return 0
        policy = EgressPolicy(profile.provider_hosts)
        matched_rule = policy.match_rule(normalized, port)
        if matched_rule is not None:
            print("Provider mode: allowed by bundled provider profile")
            print(f"Matched rule: {matched_rule}")
            print("DNS safety: checked at runtime before the proxy opens the connection.")
            return 0
        print("Provider mode: not allowed by bundled provider profile")
        print(f"Bundled hosts: {', '.join(profile.provider_hosts)}")
        print_endpoint_matches(match_provider_endpoints(normalized, profile=profile.name))
        print(f"Next action: review before rerunning with --provider-host {normalized}.")
        return 0

    matches: list[str] = []
    for profile in PROFILES.values():
        if not profile.provider_hosts:
            continue
        policy = EgressPolicy(profile.provider_hosts)
        matched_rule = policy.match_rule(normalized, port)
        if matched_rule is not None:
            matches.append(f"{profile.name} ({matched_rule})")
    if matches:
        print("Provider mode: allowed by bundled profile(s)")
        print(f"Matches: {', '.join(matches)}")
    else:
        print("Provider mode: not allowed by any bundled provider profile")
        print_endpoint_matches(match_provider_endpoints(normalized))
        print(f"Next action: review before rerunning with --provider-host {normalized}.")
    print("DNS safety: checked at runtime before the proxy opens the connection.")
    return 0


def print_endpoint_matches(matches: Sequence[ProviderEndpoint]) -> None:
    if not matches:
        return
    print("Known endpoint record(s):")
    for endpoint in matches:
        print(f"  - {endpoint.profile}: {endpoint.status}; {endpoint.purpose}")
        if endpoint.note:
            print(f"    Note: {endpoint.note}")
