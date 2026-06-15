from __future__ import annotations

import argparse
import os
import subprocess
import sys
from collections.abc import Iterator, Sequence
from contextlib import contextmanager
from dataclasses import dataclass
from pathlib import Path
from tempfile import TemporaryDirectory

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_API_KEY_ENV = "RUNHAVEN_CODEX_BROKER_SMOKE_API_KEY"
MARKER = "RUNHAVEN_BROKER_SMOKE_OK"
OUTPUT_FILE = ".runhaven-codex-broker-smoke-output.txt"


class SmokeFailure(RuntimeError):
    pass


@dataclass(frozen=True)
class CommandResult:
    args: tuple[str, ...]
    returncode: int
    stdout: str
    stderr: str


def main(argv: Sequence[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    try:
        run_smoke(args)
    except SmokeFailure as exc:
        print(f"FAIL {exc}", file=sys.stderr)
        return 1
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description=(
            "Run a real Codex non-interactive smoke through the RunHaven API-key broker. "
            "Set the disposable key in the named host environment variable first."
        )
    )
    parser.add_argument(
        "--api-key-env",
        default=DEFAULT_API_KEY_ENV,
        help="host environment variable containing a disposable OpenAI API key",
    )
    parser.add_argument(
        "--require-api-key",
        action="store_true",
        help="fail instead of skipping when the key environment variable is absent",
    )
    parser.add_argument("--workspace", type=Path, help="workspace to mount; defaults to a temp dir")
    parser.add_argument("--model", help="optional Codex model override")
    parser.add_argument("--timeout", type=int, default=180)
    return parser


def run_smoke(args: argparse.Namespace) -> None:
    if args.timeout < 1:
        raise SmokeFailure("--timeout must be greater than zero")
    if not os.environ.get(args.api_key_env):
        message = f"{args.api_key_env} is not set; skipping live Codex broker smoke"
        if args.require_api_key:
            raise SmokeFailure(message)
        print(f"SKIP {message}")
        return

    with smoke_workspace(args.workspace) as workspace:
        workspace.mkdir(parents=True, exist_ok=True)
        output_path = workspace / OUTPUT_FILE
        if output_path.exists():
            output_path.unlink()
        command = build_runhaven_command(args, workspace)
        result = run_command(command, args.timeout)
        combined_output = "\n".join(
            part.strip() for part in (result.stdout, result.stderr) if part.strip()
        )
        marker_output = output_path.read_text(encoding="utf-8") if output_path.exists() else ""
        try:
            if result.returncode != 0:
                raise SmokeFailure(f"run command failed: {summarize(result)}")
            if MARKER not in combined_output and MARKER not in marker_output:
                raise SmokeFailure(f"expected marker {MARKER!r} in Codex output")
            print("PASS Codex broker smoke completed through RunHaven provider mode")
        finally:
            if output_path.exists():
                output_path.unlink()


@contextmanager
def smoke_workspace(path: Path | None) -> Iterator[Path]:
    if path is not None:
        yield path.expanduser().resolve()
        return
    with TemporaryDirectory(prefix="runhaven-codex-broker-smoke.") as directory:
        yield Path(directory).resolve()


def build_runhaven_command(args: argparse.Namespace, workspace: Path) -> tuple[str, ...]:
    command: list[str] = [
        sys.executable,
        "-m",
        "runhaven",
        "run",
        "codex",
        "--workspace",
        str(workspace),
        "--network",
        "provider",
        "--codex-api-key-broker-env",
        args.api_key_env,
        "--tty",
        "never",
        "--no-interactive",
        "--",
        "codex",
        "exec",
        "--skip-git-repo-check",
        "--sandbox",
        "read-only",
        "--ask-for-approval",
        "never",
        "--output-last-message",
        f"/workspace/{OUTPUT_FILE}",
    ]
    if args.model:
        command.extend(("--model", args.model))
    command.append(f"Reply with exactly: {MARKER}")
    return tuple(command)


def run_command(command: tuple[str, ...], timeout: int) -> CommandResult:
    env = os.environ.copy()
    existing_pythonpath = env.get("PYTHONPATH")
    src_path = str(ROOT / "src")
    env["PYTHONPATH"] = (
        src_path if not existing_pythonpath else f"{src_path}{os.pathsep}{existing_pythonpath}"
    )
    completed = subprocess.run(
        command,
        check=False,
        capture_output=True,
        text=True,
        timeout=timeout,
        env=env,
    )
    return CommandResult(
        args=command,
        returncode=completed.returncode,
        stdout=completed.stdout,
        stderr=completed.stderr,
    )


def summarize(result: CommandResult) -> str:
    output = "\n".join(part.strip() for part in (result.stdout, result.stderr) if part.strip())
    if len(output) > 500:
        output = f"{output[:500]}..."
    return f"exit {result.returncode} from {result.args[0]!r}: {output}"


if __name__ == "__main__":
    raise SystemExit(main())
