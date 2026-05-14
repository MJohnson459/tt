# tt — time tracker for contractors

A minimal command-line tool for logging work sessions and tracking earnings across multiple clients.

## Install

```bash
cargo install --path .
```

This puts `tt` on your PATH via `~/.cargo/bin/`.

## Quick start

```bash
# Add your first client
tt client add acme --rate 150

# Clock in (uses the default client when only one exists)
tt in
tt in -n "starting auth refactor"

# Check running time
tt status

# Clock out
tt out
tt out -n "finished auth, 2 PRs merged"

# Review the day
tt log
tt log --week
tt log --month

# See earnings summary
tt summary --week
tt summary --month
```

## Commands

### Clock in / out

```
tt in [CLIENT] [-n NOTE]    start a session (omit CLIENT to use default)
tt out [-n NOTE]            end the current session
tt status                   show live running time and earnings
```

If you only have one client it's used automatically. With multiple clients you can set a default or pass the name explicitly.

### Logs

```
tt log                      all sessions, oldest first
tt log -c <client>          filter by client
tt log --week               this week only
tt log --month              this month only
```

### Summary

```
tt summary                  all-time totals by client
tt summary --week
tt summary --month
```

### Clients

```
tt client add <name> --rate <rate>    add or update a client ($/hr)
tt client list                        show all clients and rates
tt client default <name>              set the default for clock-in
tt client remove <name>               remove a client
```

## Data

Sessions are stored as JSON at:

```
~/.local/share/timetracker/data.json
```

Respects `$XDG_DATA_HOME` if set. The file is human-readable and easy to back up or inspect.
