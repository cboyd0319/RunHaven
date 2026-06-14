# Contributing

This repo optimizes for safe defaults and beginner usability.

Users are trusting this project with personal machines, personal files, and
developer credentials. Security-sensitive behavior must fail closed when it
cannot verify a boundary. Do not hide risk behind friendly wording.

## Local Checks

```bash
python3 -m compileall src tests
PYTHONPATH=src python3 -m unittest discover -s tests
python3 scripts/check_pins.py
```

Developer tools are installed from the transitive lock:

```bash
python3.14 -m venv .venv
source .venv/bin/activate
python -m pip install pip==26.1.2
python -m pip install -r requirements-dev.txt
python -m pip install --no-deps -e .
python -m ruff check .
python -m mypy src
```

## Security Review Expectations

- Show the exact `container` command with `mca plan` before changing runtime
  behavior.
- Keep dependencies current stable and hard-pinned. Updating a package means
  changing the exact version or digest in source control.
- Add or update tests for every change to command construction.
- Keep host secrets out of generated commands unless the user explicitly passes
  a variable name with `--env`.
- Do not add broad mounts for convenience. Add a narrow mount or a documented
  explicit option.
- Do not claim full isolation for a mode unless a focused runtime check proves
  the claimed boundary.
