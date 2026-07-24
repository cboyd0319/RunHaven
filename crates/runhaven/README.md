# runhaven

Binary entrypoints for the RunHaven CLI and pin checker. Shared runtime behavior
belongs in `runhaven-core`; CLI presentation belongs in `runhaven-cli`.

Focused verification: `scripts/cargo.sh test -p runhaven --locked`.
