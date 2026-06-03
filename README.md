# tt — time tracker for contractors

A minimal command-line tool for logging work sessions, tracking earnings across multiple clients and currencies, and generating monthly reports.

## Install

```bash
cargo install timetracker
```

Or from source:

```bash
cargo install --path .
```

Both put `tt` on your PATH via `~/.cargo/bin/`.

## Quick start

```bash
# Add your first client (currency inferred from symbol)
tt client add acme --rate '$150'
tt client add eurotech --rate '€90'
tt client add ukco --rate '£75'

# Clock in (uses the default client when only one exists)
tt in
tt in -n "starting auth refactor"

# Add notes as you go — no quotes needed
tt note fixed the login endpoint
tt note PR up for review

# Check running time
tt status

# Clock out
tt out
tt out -n "finished for the day"

# End-of-month report (defaults to last month)
tt report
tt report -o may-2026.md
```

## Commands

### Clock in / out

```
tt in [CLIENT] [-n NOTE]    start a session (omit CLIENT to use default)
tt out [-n NOTE]            end the current session
tt status                   show live running time and earnings
```

### Notes

```
tt note <text>              append a timestamped note to the active session
```

Each note is stored with a timestamp. You can add as many as you like during a session. No quoting needed — `tt note fixed the login bug` works as-is.

### Retroactive sessions

```
tt add [CLIENT] --start <time> --end <time> [-n NOTE]
```

`<time>` is either `HH:MM` (today) or `YYYY-MM-DD HH:MM`:

```bash
tt add --start "09:00" --end "12:30"
tt add --start "2026-05-13 09:00" --end "2026-05-13 17:30" -n "full day"
tt add ukco --start "14:00" --end "16:00" -n "client call"
```

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

When sessions span multiple currencies, separate totals are shown per currency.

### Monthly report

```
tt report                           last month, all clients → stdout
tt report --month 2026-05           specific month
tt report -c acme                   one client only
tt report -o may-2026.md            write to file
tt report --month 2026-05 -c acme -o acme-may.md
```

Generates a Markdown document per client with rate, period, total hours and earnings, and a collated list of all notes from the month — ready to attach to an invoice or paste into an email.

### Clients

```
tt client add <name> --rate <rate>          add or update a client
tt client add <name> --rate <rate> --currency <code>   explicit currency code
tt client list                              show all clients and rates
tt client default <name>                   set the default for clock-in
tt client remove <name>                    remove a client
```

#### Currency

The currency is inferred from the symbol prefix of `--rate`:

| Symbol | Currency |
|--------|----------|
| `$`    | USD      |
| `€`    | EUR      |
| `£`    | GBP      |
| `¥`    | JPY      |
| `₹`    | INR      |
| `C$`   | CAD      |
| `A$`   | AUD      |
| `NZ$`  | NZD      |
| `HK$`  | HKD      |
| `S$`   | SGD      |
| `MX$`  | MXN      |

For any other currency pass `--currency <ISO code>` explicitly, e.g. `--currency CHF`.

### Raw data

```
tt edit         open the data file in $EDITOR (falls back to vi)
```

## Data

Sessions are stored as JSON at:

```
~/.local/share/timetracker/data.json
```

Respects `$XDG_DATA_HOME` if set. The file is human-readable and straightforward to back up or inspect directly.

## License

MIT — see [LICENSE](LICENSE).
