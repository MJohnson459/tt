use std::collections::HashMap;

use anyhow::{anyhow, bail};
use chrono::{Datelike, Duration, Local, NaiveDate};
use colored::Colorize;

use crate::data::{Session, Store};

fn fmt_hours(h: f64) -> String {
    let total_m = (h * 60.0).round() as i64;
    let hours = total_m / 60;
    let mins = total_m % 60;
    if hours > 0 {
        format!("{}h {:02}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}

fn parse_rate(s: &str) -> anyhow::Result<(f64, Option<String>)> {
    // Longest symbol first to avoid "$" matching before "C$" etc.
    const SYMBOLS: &[(&str, &str)] = &[
        ("NZ$", "NZD"),
        ("HK$", "HKD"),
        ("MX$", "MXN"),
        ("C$", "CAD"),
        ("A$", "AUD"),
        ("S$", "SGD"),
        ("£", "GBP"),
        ("€", "EUR"),
        ("¥", "JPY"),
        ("₹", "INR"),
        ("$", "USD"),
    ];

    for (sym, code) in SYMBOLS {
        if s.starts_with(sym) {
            let rest = s[sym.len()..].trim();
            let rate = rest.parse::<f64>().map_err(|_| {
                anyhow!("invalid rate '{}': expected a number after '{}'", s, sym)
            })?;
            return Ok((rate, Some(code.to_string())));
        }
    }

    let rate = s.trim().parse::<f64>().map_err(|_| {
        anyhow!(
            "invalid rate '{}': expected a number or a symbol-prefixed amount (e.g. £75, €90, $100)",
            s
        )
    })?;
    Ok((rate, None))
}

fn currency_symbol(code: &str) -> &str {
    match code {
        "USD" => "$",
        "CAD" => "C$",
        "AUD" => "A$",
        "NZD" => "NZ$",
        "HKD" => "HK$",
        "SGD" => "S$",
        "MXN" => "MX$",
        "EUR" => "€",
        "GBP" => "£",
        "JPY" => "¥",
        "CNY" => "¥",
        "INR" => "₹",
        _ => "",
    }
}

fn fmt_money(amount: f64, currency: &str) -> String {
    let sym = currency_symbol(currency);
    if sym.is_empty() {
        format!("{} {:.2}", currency, amount)
    } else {
        format!("{}{:.2}", sym, amount)
    }
}

pub fn clock_in(
    store: &mut Store,
    client: Option<String>,
    note: Option<String>,
) -> anyhow::Result<()> {
    if let Some(active) = store.active_session() {
        bail!(
            "Already clocked in to '{}' since {}",
            active.client,
            active.start.format("%H:%M")
        );
    }

    let client_name = client
        .or_else(|| store.default_client.clone())
        .ok_or_else(|| {
            anyhow!(
                "No client specified and no default set.\n  Add a client: tt client add <name> --rate <rate>"
            )
        })?;

    let info = store.clients.get(&client_name).ok_or_else(|| {
        anyhow!(
            "Unknown client '{}'.\n  Add it with: tt client add {} --rate <rate>",
            client_name,
            client_name
        )
    })?.clone();

    let id = store.new_id();
    let start = Local::now();
    let session = Session {
        id,
        client: client_name.clone(),
        start,
        end: None,
        note: note.clone(),
        rate: info.rate,
        currency: info.currency.clone(),
    };

    println!(
        "{} Clocked in to {}  ({}/hr)  {}",
        "●".green().bold(),
        client_name.bold(),
        fmt_money(info.rate, &info.currency),
        start.format("%H:%M on %a %b %-d")
    );
    if let Some(n) = &note {
        println!("  Note: {}", n.dimmed());
    }

    store.sessions.push(session);
    Ok(())
}

pub fn clock_out(store: &mut Store, note: Option<String>) -> anyhow::Result<()> {
    let session = store
        .active_session_mut()
        .ok_or_else(|| anyhow!("Not currently clocked in."))?;

    let end = Local::now();
    session.end = Some(end);
    if note.is_some() {
        session.note = note;
    }

    let hours = session.duration_hours();
    let earnings = session.earnings();
    let client = session.client.clone();
    let currency = session.currency.clone();

    println!(
        "{} Clocked out of {}  {}  {}",
        "○".dimmed(),
        client.bold(),
        fmt_hours(hours).bold(),
        fmt_money(earnings, &currency).green().bold()
    );
    Ok(())
}

pub fn status(store: &Store) {
    match store.active_session() {
        Some(s) => {
            let hours = s.duration_hours();
            println!("{} Clocked in", "●".green().bold());
            println!("  Client:  {}", s.client.bold());
            println!("  Since:   {}", s.start.format("%H:%M on %a %b %-d"));
            println!(
                "  Running: {}  =  {}",
                fmt_hours(hours),
                fmt_money(s.earnings(), &s.currency).green()
            );
            if let Some(note) = &s.note {
                println!("  Note:    {}", note.dimmed());
            }
        }
        None => println!("{} Not clocked in", "○".dimmed()),
    }
}

pub fn note(store: &mut Store, text: String) -> anyhow::Result<()> {
    let session = store
        .active_session_mut()
        .ok_or_else(|| anyhow!("No active session — clock in first."))?;

    session.note = Some(match session.note.take() {
        Some(existing) => format!("{}; {}", existing, text),
        None => text,
    });

    println!("  Note: {}", session.note.as_deref().unwrap().dimmed());
    Ok(())
}

fn week_start() -> chrono::DateTime<Local> {
    let today = Local::now().date_naive();
    let days_since_monday = today.weekday().num_days_from_monday();
    let monday = today - Duration::days(days_since_monday as i64);
    monday
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(Local)
        .unwrap()
}

fn month_start() -> chrono::DateTime<Local> {
    let now = Local::now();
    NaiveDate::from_ymd_opt(now.year(), now.month(), 1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(Local)
        .unwrap()
}

fn filter_sessions<'a>(
    sessions: &'a [Session],
    client: Option<&str>,
    week: bool,
    month: bool,
) -> Vec<&'a Session> {
    let cutoff = if week {
        Some(week_start())
    } else if month {
        Some(month_start())
    } else {
        None
    };

    sessions
        .iter()
        .filter(|s| {
            if let Some(c) = client {
                if s.client != c {
                    return false;
                }
            }
            if let Some(t) = cutoff {
                if s.start < t {
                    return false;
                }
            }
            true
        })
        .collect()
}

pub fn log(store: &Store, client: Option<String>, week: bool, month: bool) {
    let mut sessions = filter_sessions(&store.sessions, client.as_deref(), week, month);
    sessions.sort_by(|a, b| a.start.cmp(&b.start));

    if sessions.is_empty() {
        println!("No sessions found.");
        return;
    }

    let cw = sessions
        .iter()
        .map(|s| s.client.len())
        .max()
        .unwrap_or(6)
        .max(6);

    println!(
        "{:>4}  {:<cw$}  {:<12}  {:>5}  {:>5}  {:>9}  {:>10}  {}",
        "#".dimmed(),
        "Client".dimmed(),
        "Date".dimmed(),
        "Start".dimmed(),
        "End".dimmed(),
        "Duration".dimmed(),
        "Earnings".dimmed(),
        "Note".dimmed(),
    );
    println!("{}", "─".repeat(66 + cw.saturating_sub(6)).dimmed());

    for s in &sessions {
        let end_str = s
            .end
            .map(|e| e.format("%H:%M").to_string())
            .unwrap_or_else(|| "─────".to_string());

        let note_str = s.note.as_deref().unwrap_or("");
        let note_display = if note_str.len() > 30 {
            format!("{}…", &note_str[..29])
        } else {
            note_str.to_string()
        };

        let earnings_str = if s.is_active() {
            format!("~{}", fmt_money(s.earnings(), &s.currency))
        } else {
            fmt_money(s.earnings(), &s.currency)
        };

        let active_marker = if s.is_active() { " ●" } else { "" };

        println!(
            "{:>4}  {:<cw$}  {:<12}  {:>5}  {:>5}  {:>9}  {:>10}  {}{}",
            s.id,
            s.client,
            s.start.format("%a %b %-d").to_string(),
            s.start.format("%H:%M").to_string(),
            end_str,
            fmt_hours(s.duration_hours()),
            earnings_str,
            note_display,
            active_marker,
        );
    }
}

pub fn summary(store: &Store, week: bool, month: bool) {
    let sessions = filter_sessions(&store.sessions, None, week, month);

    let label = if week {
        let today = Local::now().date_naive();
        let days_since_monday = today.weekday().num_days_from_monday();
        let monday = today - Duration::days(days_since_monday as i64);
        format!("Week of {}", monday.format("%b %-d, %Y"))
    } else if month {
        Local::now().format("Month of %B %Y").to_string()
    } else {
        "All time".to_string()
    };

    println!("{}", label.bold());

    if sessions.is_empty() {
        println!("No sessions.");
        return;
    }

    // client -> (hours, earnings, currency)
    let mut by_client: HashMap<String, (f64, f64, String)> = HashMap::new();
    for s in &sessions {
        let e = by_client
            .entry(s.client.clone())
            .or_insert((0.0, 0.0, s.currency.clone()));
        e.0 += s.duration_hours();
        e.1 += s.earnings();
    }

    let cw = by_client
        .keys()
        .map(|k| k.len())
        .max()
        .unwrap_or(6)
        .max(6);
    let rule = "─".repeat(44 + cw.saturating_sub(6));

    println!("{}", rule.dimmed());
    println!(
        "{:<cw$}  {:>8}  {:>12}  {}",
        "Client".dimmed(),
        "Hours".dimmed(),
        "Earnings".dimmed(),
        "Rate".dimmed(),
    );
    println!("{}", rule.dimmed());

    let mut clients: Vec<_> = by_client.iter().collect();
    clients.sort_by_key(|(k, _)| k.as_str());

    // currency -> (hours, earnings)
    let mut by_currency: HashMap<String, (f64, f64)> = HashMap::new();

    for (client, (hours, earnings, currency)) in &clients {
        let info = store.clients.get(client.as_str());
        let rate_str = match info {
            Some(i) => format!("@ {}/hr", fmt_money(i.rate, &i.currency)),
            None => String::new(),
        };
        println!(
            "{:<cw$}  {:>7.2}h  {:>12}  {}",
            client,
            hours,
            fmt_money(*earnings, currency).green(),
            rate_str,
        );
        let e = by_currency.entry(currency.clone()).or_default();
        e.0 += hours;
        e.1 += earnings;
    }

    println!("{}", rule.dimmed());

    let mut currencies: Vec<_> = by_currency.iter().collect();
    currencies.sort_by_key(|(k, _)| k.as_str());

    for (currency, (hours, earnings)) in &currencies {
        let label = if currencies.len() == 1 {
            "Total".to_string()
        } else {
            format!("Total {}", currency)
        };
        println!(
            "{:<cw$}  {:>7.2}h  {:>12}",
            label.bold(),
            hours,
            fmt_money(*earnings, currency).green().bold(),
        );
    }
}

pub fn client_add(
    store: &mut Store,
    name: String,
    rate_input: &str,
    currency_override: Option<String>,
) -> anyhow::Result<()> {
    let (rate, symbol_currency) = parse_rate(rate_input)?;
    if rate <= 0.0 {
        bail!("Rate must be greater than 0");
    }
    let currency = currency_override
        .map(|c| c.to_uppercase())
        .or(symbol_currency)
        .unwrap_or_else(|| "USD".to_string());
    let existed = store.clients.contains_key(&name);
    let rate_str = fmt_money(rate, &currency);
    store
        .clients
        .insert(name.clone(), crate::data::ClientInfo { rate, currency });
    if !existed && store.default_client.is_none() {
        store.default_client = Some(name.clone());
        println!(
            "{} Added '{}' at {}/hr  (set as default)",
            "✓".green(),
            name,
            rate_str
        );
    } else if existed {
        println!(
            "{} Updated '{}' rate to {}/hr",
            "✓".green(),
            name,
            rate_str
        );
    } else {
        println!("{} Added '{}' at {}/hr", "✓".green(), name, rate_str);
    }
    Ok(())
}

pub fn client_list(store: &Store) {
    if store.clients.is_empty() {
        println!("No clients. Add one with: tt client add <name> --rate <rate>");
        return;
    }

    let mut clients: Vec<_> = store.clients.iter().collect();
    clients.sort_by_key(|(k, _)| k.as_str());

    println!(
        "{:<20}  {:>12}  {}",
        "Client".dimmed(),
        "Rate".dimmed(),
        "Currency".dimmed()
    );
    println!("{}", "─".repeat(42).dimmed());

    for (name, info) in clients {
        let is_default = store.default_client.as_deref() == Some(name.as_str());
        let marker = if is_default {
            " (default)".dimmed().to_string()
        } else {
            String::new()
        };
        println!(
            "{:<20}  {:>10}/hr  {}{}",
            name,
            fmt_money(info.rate, &info.currency),
            info.currency,
            marker
        );
    }
}

pub fn client_remove(store: &mut Store, name: String) -> anyhow::Result<()> {
    if !store.clients.contains_key(&name) {
        bail!("Client '{}' not found", name);
    }
    if store.active_session().map(|s| s.client.as_str()) == Some(name.as_str()) {
        bail!(
            "Cannot remove '{}' while clocked in — clock out first",
            name
        );
    }
    store.clients.remove(&name);
    if store.default_client.as_deref() == Some(&name) {
        store.default_client = store.clients.keys().next().cloned();
    }
    println!("{} Removed '{}'", "✓".green(), name);
    Ok(())
}

pub fn client_default(store: &mut Store, name: String) -> anyhow::Result<()> {
    if !store.clients.contains_key(&name) {
        bail!(
            "Client '{}' not found. Add it first: tt client add {} --rate <rate>",
            name,
            name
        );
    }
    store.default_client = Some(name.clone());
    println!("{} Default client set to '{}'", "✓".green(), name);
    Ok(())
}
