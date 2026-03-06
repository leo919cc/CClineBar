use super::{Segment, SegmentData};
use crate::config::{InputData, SegmentId};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
struct MonthlyCostData {
    month: String,
    sessions: HashMap<String, f64>,
}

fn get_cost_dir() -> PathBuf {
    if let Some(home) = dirs::home_dir() {
        home.join(".claude").join("ccline")
    } else {
        PathBuf::from(".claude/ccline")
    }
}

fn get_cost_file_path() -> PathBuf {
    get_cost_dir().join("monthly_cost.json")
}

fn get_lock_file_path() -> PathBuf {
    get_cost_dir().join("monthly_cost.lock")
}

fn current_month() -> String {
    chrono::Local::now().format("%Y-%m").to_string()
}

fn load_monthly_data_from_path(path: &PathBuf) -> Option<MonthlyCostData> {
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Atomically write data: write to temp file, fsync, rename over target, fsync parent dir.
fn atomic_save(path: &PathBuf, data: &MonthlyCostData) -> std::io::Result<()> {
    let dir = path.parent().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::Other, "no parent dir")
    })?;
    std::fs::create_dir_all(dir)?;

    let tmp_path = dir.join(".monthly_cost.tmp");
    {
        let mut tmp = std::fs::File::create(&tmp_path)?;
        let json = serde_json::to_string_pretty(data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        tmp.write_all(json.as_bytes())?;
        tmp.sync_all()?;
    }
    std::fs::rename(&tmp_path, path)?;

    // Fsync parent directory to ensure rename metadata is durable
    if let Ok(dir_handle) = std::fs::File::open(dir) {
        let _ = dir_handle.sync_all();
    }

    Ok(())
}

/// Track this session's cost in the monthly data file.
/// Uses file locking to prevent concurrent read-modify-write races.
/// Enforces monotonic max per session so totals never decrease.
/// Returns the monthly total.
pub fn track_monthly_cost(transcript_path: &str, session_cost: f64) -> Option<f64> {
    let cost_path = get_cost_file_path();
    let lock_path = get_lock_file_path();

    // Ensure directory exists for lock file
    if let Some(parent) = lock_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    // Open lock file and acquire exclusive lock
    let lock_file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(&lock_path)
        .ok()?;
    lock_file.lock_exclusive().ok()?;

    // === Critical section: read, update, write ===
    // Month computed inside lock to prevent midnight boundary races
    let month = current_month();

    let mut data = match load_monthly_data_from_path(&cost_path) {
        Some(d) => d,
        None => {
            // File doesn't exist yet — start fresh
            // (Parse errors also land here; acceptable since corrupt data
            // would be overwritten anyway and the lock prevents concurrent damage)
            MonthlyCostData {
                month: month.clone(),
                sessions: HashMap::new(),
            }
        }
    };

    // Reset if month changed
    if data.month != month {
        data = MonthlyCostData {
            month,
            sessions: HashMap::new(),
        };
    }

    // Monotonic max: never decrease a session's cost
    let existing = data.sessions.get(transcript_path).copied().unwrap_or(0.0);
    data.sessions
        .insert(transcript_path.to_string(), session_cost.max(existing));

    let total: f64 = data.sessions.values().sum();

    // Atomic write: temp file + fsync + rename + fsync dir
    // Fail the update if persistence fails — lock released via drop
    atomic_save(&cost_path, &data).ok()?;

    Some(total)
}

/// Read the monthly total without locking.
/// Safe because atomic_save uses rename, so readers always see a complete file.
fn read_monthly_total() -> Option<f64> {
    let data = load_monthly_data_from_path(&get_cost_file_path())?;
    if data.month != current_month() {
        return Some(0.0);
    }
    Some(data.sessions.values().sum())
}

#[derive(Default)]
pub struct CostSegment;

impl CostSegment {
    pub fn new() -> Self {
        Self
    }
}

impl Segment for CostSegment {
    fn collect(&self, input: &InputData) -> Option<SegmentData> {
        let cost_data = input.cost.as_ref()?;
        let session_cost = cost_data.total_cost_usd?;

        let session_display = if session_cost == 0.0 || session_cost < 0.01 {
            "$0".to_string()
        } else {
            format!("${:.2}", session_cost)
        };

        // Read monthly total (already written by track_monthly_cost in main)
        let primary = match read_monthly_total() {
            Some(monthly_total) if monthly_total >= 0.01 => {
                format!("{} / ${:.2}", session_display, monthly_total)
            }
            _ => session_display,
        };

        let mut metadata = HashMap::new();
        metadata.insert("cost".to_string(), session_cost.to_string());

        Some(SegmentData {
            primary,
            secondary: String::new(),
            metadata,
        })
    }

    fn id(&self) -> SegmentId {
        SegmentId::Cost
    }
}
