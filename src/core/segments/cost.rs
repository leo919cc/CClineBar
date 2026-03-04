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

fn update_and_get_monthly_total(transcript_path: &str, session_cost: f64) -> Option<f64> {
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

        // Try to get monthly total, fallback to session-only display
        let primary = match update_and_get_monthly_total(&input.transcript_path, session_cost) {
            Some(monthly_total) => {
                let monthly_display = if monthly_total < 0.01 {
                    "$0".to_string()
                } else {
                    format!("${:.2}", monthly_total)
                };
                format!("{} / {}", session_display, monthly_display)
            }
            None => session_display,
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
