---
title: Report a problem
description: What to include and where to post when Oracle Brand Guide misbehaves or crashes.
order: 20
verified: 0.7.1
---

Post in the [issue tracker](https://github.com/Marvingxx/oracle-guide/issues), and attach the log file. The log is the most useful thing you can send after a crash.

## What to include

- Your Oracle Brand Guide version and how you installed it (installer, portable, AppImage and so on).
- Your operating system and graphics card.
- What you did, what you expected and what happened.
- The log file `oracle-guide.log`.

## Where the log is

Oracle Brand Guide writes `logs/oracle-guide.log` in its settings folder, beside `ui.json` and `prefs.json`.

| System  | Log folder                                                              |
|---------|-------------------------------------------------------------------------|
| Windows | `%APPDATA%\Oracle Brand Guide\logs\`                                           |
| macOS   | `~/Library/Application Support/Oracle Brand Guide/logs/`                       |
| Linux   | `~/.config/oracle-guide/logs/` (or `$XDG_CONFIG_HOME/oracle-guide/logs/`) |

Each start moves the previous log to `oracle-guide.1.log`, and that one to `oracle-guide.2.log`. The log from a run that crashed survives the next start, so attach `oracle-guide.1.log` if you have restarted since. The file stops growing at 16 MiB.

## More detail in the log

Set the environment variable `RUST_LOG` to `debug` before starting Oracle Brand Guide to log more. The default is `info` for Oracle Brand Guide's own parts and `warn` for everything else.
