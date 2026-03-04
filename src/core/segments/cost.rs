use super::{Segment, SegmentData};
use crate::config::{InputData, SegmentId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
struct MonthlyCostData {
    month: String,
    sessions: HashMap<String, f64>,
}

fn get_cost_file_path() -> PathBuf {
    if let Some(home) = dirs::home_dir() {
        home.join(".claude").join("ccline").join("monthly_cost.json")
    } else {
        PathBuf::from(".claude/ccline/monthly_cost.json")
    }
}

fn current_month() -> String {
    chrono::Local::now().format("%Y-%m").to_string()
}

fn load_monthly_data() -> Option<MonthlyCostData> {
    let path = get_cost_file_path();
    let content = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

fn save_monthly_data(data: &MonthlyCostData) {
    let path = get_cost_file_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(data) {
        let _ = std::fs::write(&path, json);
    }
}

/// Track this session's cost in the monthly data file.
/// Called on every render regardless of whether the cost segment is displayed.
/// Returns the monthly total.
pub fn track_monthly_cost(transcript_path: &str, session_cost: f64) -> Option<f64> {
    let month = current_month();

    let mut data = load_monthly_data().unwrap_or_else(|| MonthlyCostData {
        month: month.clone(),
        sessions: HashMap::new(),
    });

    // Reset if month changed
    if data.month != month {
        data = MonthlyCostData {
            month,
            sessions: HashMap::new(),
        };
    }

    // Overwrite this session's cost (no double-counting)
    data.sessions
        .insert(transcript_path.to_string(), session_cost);

    let total: f64 = data.sessions.values().sum();

    save_monthly_data(&data);

    Some(total)
}

/// Read the monthly total without writing. Used by the cost segment for display.
fn read_monthly_total() -> Option<f64> {
    let data = load_monthly_data()?;
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
