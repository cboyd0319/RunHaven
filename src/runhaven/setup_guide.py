from __future__ import annotations

from .doctor import Check
from .profiles import get_profile


def print_checks(checks: tuple[Check, ...]) -> None:
    for check in checks:
        status = "ok" if check.ok else "fail"
        print(f"{status:4} {check.name}: {check.detail}")
        if not check.ok and check.remedy:
            print(f"     fix: {check.remedy}")


def print_setup_guide(agent: str, checks: tuple[Check, ...]) -> int:
    profile = get_profile(agent)
    ready = all(check.ok for check in checks)
    print("RunHaven setup")
    print()
    print("1. Host prerequisites")
    print_checks(checks)
    print()
    if not ready:
        print("Next steps")
        for check in checks:
            if not check.ok and check.remedy:
                print(f"- {check.name}: {check.remedy}")
        print("- After fixing the items above, run `runhaven setup` again.")
        return 1

    print(f"Selected agent: {agent} - {profile.description}")
    print()
    print("2. Build the agent image")
    print(f"   runhaven image build {agent}")
    print()
    print("3. Preview the container boundary")
    print(f"   runhaven plan {agent}")
    print()
    print("4. Run from your project directory")
    print(f"   runhaven run {agent}")
    print()
    print("Safety defaults")
    print("- One selected project is mounted at /workspace.")
    print("- No host home, raw SSH keys, or cloud credential folders are mounted by default.")
    print_setup_workspace_and_credentials()
    print_setup_network_choices(agent)
    return 0


def print_setup_workspace_and_credentials() -> None:
    print()
    print("Workspace and credentials")
    print(
        "- Run from the smallest project directory you want the agent to see; "
        "that directory is mounted at /workspace."
    )
    print(
        "- Do not run from your home directory, a cloud sync root, or a "
        "credential folder unless you intentionally allow that broader scope."
    )
    print(
        "- RunHaven does not mount raw SSH keys, browser profiles, cloud "
        "credential folders, or provider login caches by default."
    )
    print("- Use `--ssh` for SSH agent forwarding instead of mounting key files.")
    print(
        "- Use `--env NAME` only for a reviewed variable that the agent really "
        "needs."
    )
    print("- Use `runhaven plan` to confirm the mounted host path.")


def print_setup_network_choices(agent: str) -> None:
    print()
    print("Network choices")
    print(
        f"- Local-only: use `runhaven run {agent} --network internal` for tests, "
        "builds, and commands that do not need internet."
    )
    print(
        f"- Provider-only: use `runhaven run {agent} --network provider` to allow "
        "reviewed provider hosts through the proxy. Login, telemetry, package "
        "registries, or feature paths may need extra reviewed hosts."
    )
    print(
        f"- Package install: use default internet mode with `runhaven run {agent}` "
        "when package managers or dependency updates need broad registry and "
        "CDN access."
    )
    print(
        f"- Unrestricted internet: default `runhaven run {agent}` leaves egress "
        "unrestricted inside Apple `container` and your host network."
    )
    print("- Use `runhaven plan` before changing network modes.")
