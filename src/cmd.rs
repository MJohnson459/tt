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

fn fmt_money(amount: f64) -> String {
    format!("${:.2}", amount)
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

    let rate = *store.clients.get(&client_name).ok_or_else(|| {
        anyhow!(
            "Unknown client '{}'.\n  Add it with: tt client add {} --rate <rate>",
            client_name,
            client_name
        )
    })?;

    let id = store.new_id();
    let start = Local::now();
    let session = Session {
        id,
        client: client_name.clone(),
        start,
        end: None,
        note: note.clone(),
        rate,
    };

    println!(
        "{} Clocked in to {}  (${}/hr)  {}",
        "●".green().bold(),
        client_name.bold(),
        rate,
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

    println!(
        "{} Clocked out of {}  {}  {}",
        "○".dimmed(),
        client.bold(),
        fmt_hours(hours).bold(),
        fmt_money(earnings).green().bold()
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
                fmt_money(s.earnings()).green()
            );
            if let Some(note) = &s.note {
                println!("  Note:    {}", note.dimmed());
            }
        }
        None => println!("{} Not clocked in", "○".dimmed()),
    }
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
            format!("~{}", fmt_money(s.earnings()))
        } else {
            fmt_money(s.earnings())
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

    let mut by_client: HashMap<String, (f64, f64)> = HashMap::new();
    for s in &sessions {
        let e = by_client.entry(s.client.clone()).or_default();
        e.0 += s.duration_hours();
        e.1 += s.earnings();
    }

    let cw = by_client
        .keys()
        .map(|k| k.len())
        .max()
        .unwrap_or(6)
        .max(6);
    let rule = "─".repeat(42 + cw.saturating_sub(6));

    println!("{}", rule.dimmed());
    println!(
        "{:<cw$}  {:>8}  {:>10}  {}",
        "Client".dimmed(),
        "Hours".dimmed(),
        "Earnings".dimmed(),
        "Rate".dimmed(),
    );
    println!("{}", rule.dimmed());

    let mut clients: Vec<_> = by_client.iter().collect();
    clients.sort_by_key(|(k, _)| k.as_str());

    let mut total_hours = 0.0f64;
    let mut total_earnings = 0.0f64;

    for (client, (hours, earnings)) in &clients {
        let rate = store.clients.get(client.as_str()).copied().unwrap_or(0.0);
        println!(
            "{:<cw$}  {:>7.2}h  {:>10}  @ ${}/hr",
            client,
            hours,
            fmt_money(*earnings).green(),
            rate,
        );
        total_hours += hours;
        total_earnings += earnings;
    }

    if clients.len() > 1 {
        println!("{}", rule.dimmed());
        println!(
            "{:<cw$}  {:>7.2}h  {:>10}",
            "Total".bold(),
            total_hours,
            fmt_money(total_earnings).green().bold(),
        );
    }
}

pub fn client_add(store: &mut Store, name: String, rate: f64) -> anyhow::Result<()> {
    if rate <= 0.0 {
        bail!("Rate must be greater than 0");
    }
    let existed = store.clients.contains_key(&name);
    store.clients.insert(name.clone(), rate);
    if !existed && store.default_client.is_none() {
        store.default_client = Some(name.clone());
        println!(
            "{} Added '{}' at ${}/hr  (set as default)",
            "✓".green(),
            name,
            rate
        );
    } else if existed {
        println!("{} Updated '{}' rate to ${}/hr", "✓".green(), name, rate);
    } else {
        println!("{} Added '{}' at ${}/hr", "✓".green(), name, rate);
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

    println!("{:<20}  {:>10}", "Client".dimmed(), "Rate".dimmed());
    println!("{}", "─".repeat(34).dimmed());

    for (name, rate) in clients {
        let is_default = store.default_client.as_deref() == Some(name.as_str());
        let marker = if is_default {
            " (default)".dimmed().to_string()
        } else {
            String::new()
        };
        println!("{:<20}  {:>8}$/hr{}", name, rate, marker);
    }
}

pub fn client_remove(store: &mut Store, name: String) -> anyhow::Result<()> {
    if !store.clients.contains_key(&name) {
        bail!("Client '{}' not found", name);
    }
    if store.active_session().map(|s| &s.client) == Some(&name) {
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
