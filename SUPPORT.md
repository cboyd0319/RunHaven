# RunHaven Support

RunHaven can create one text report for any app, setup, session, sign-in,
history, or installation problem. The file stays private on your Mac until you
attach it. The report is optional. RunHaven never uploads or submits it.

## Create a report in the desktop app

1. Choose **Help > Get Support…**, or open **Help** and choose **Get Support…**.
2. Read the macOS confirmation, then choose **Create report**.
3. Review the file selected in Finder.
4. Drag the file into the GitHub support form that RunHaven opened.
5. Review the issue and attachments, then submit it when you are ready.

GitHub uploads an attachment when you select it, before you submit the issue.
Do not attach the report until you have reviewed it.

If the desktop app does not open, use Terminal:

```bash
runhaven support
```

The command prints the report path and the support-form address. It does not
open another app, upload the file, or submit an issue.

## What the report includes

The report contains only allowlisted diagnostic facts:

- the RunHaven and macOS versions
- Apple Container installation, version, and service results
- the status of each bundled assistant image
- recent session status, exit code, cleanup result, and allowed or denied event
  counts
- recent normalized RunHaven action names, times, outcomes, durations, and exit
  codes

RunHaven rebuilds these facts from known record fields and known app-event
patterns. It does not copy a raw log into the report. Unknown or unstructured
log lines are counted and omitted.

## What the report excludes

The report does not include:

- raw application logs or terminal output
- prompts, messages, or assistant responses
- project names, paths, files, contents, branches, diffs, or changes
- session names, run identifiers, state-volume names, or host user names
- environment values, credentials, tokens, or account identifiers
- network hosts, ports, request paths, request bodies, or browser data

The report is capped at 256 KiB. RunHaven reads only bounded recent metadata and
uses time limits for Apple Container checks, so a broken service cannot leave
report creation running indefinitely.

## Where the report is saved

RunHaven saves a single `.txt` file under:

```text
~/Library/Application Support/RunHaven/Support Reports/
```

The folder is readable only by your macOS account (`0700`). Each report is
readable only by your account (`0600`). RunHaven uses a new name for every
report and does not create a ZIP, tar archive, or hidden metadata bundle.

Deleting a report in Finder removes it. RunHaven does not retain a second copy.

## Security or privacy problems

Do not post a report or sensitive details in a public issue if RunHaven may have
exposed credentials, accessed an unexpected host location, escaped its
container boundary, or run an unexpected host command. Use
[GitHub private vulnerability reporting](https://github.com/cboyd0319/RunHaven/security/advisories/new).

## Contributor-only debug log

RunHaven also keeps a rotating local debug log while the desktop app runs. That
log is not a support report and may contain more operational detail. Do not ask
users to attach it. Contributors can inspect it locally after reproducing a
problem and run `scripts/scan-dev-log.sh` before using it in private debugging.
