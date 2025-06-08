use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TranslationEntry {
    pub id: String,
    pub original_text: String,
    pub translated_text: String,
    pub detected_language: String,
    pub target_language: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct TranslationHistory {
    pub entries: Vec<TranslationEntry>,
}

impl TranslationHistory {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, entry: TranslationEntry) {
        // Insert at the beginning to keep newest entries first
        self.entries.insert(0, entry);

        // Keep only the last 100 entries to prevent the file from growing too large
        if self.entries.len() > 100 {
            self.entries.truncate(100);
        }
    }
}

fn get_history_file_path() -> Result<PathBuf> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;

    let config_dir = home_dir.join(".gptranslate");

    // Create the directory if it doesn't exist
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }

    Ok(config_dir.join("history.json"))
}

pub fn load_history() -> Result<TranslationHistory> {
    let history_path = get_history_file_path()?;

    if !history_path.exists() {
        return Ok(TranslationHistory::new());
    }

    let contents = fs::read_to_string(history_path)?;
    let history: TranslationHistory =
        serde_json::from_str(&contents).unwrap_or_else(|_| TranslationHistory::new());

    Ok(history)
}

pub fn save_history(history: &TranslationHistory) -> Result<()> {
    let history_path = get_history_file_path()?;
    let contents = serde_json::to_string_pretty(history)?;
    fs::write(history_path, contents)?;
    Ok(())
}

pub fn add_translation_to_history(
    original_text: String,
    translated_text: String,
    detected_language: String,
    target_language: String,
) -> Result<()> {
    let mut history = load_history()?;

    let entry = TranslationEntry {
        id: uuid::Uuid::new_v4().to_string(),
        original_text,
        translated_text,
        detected_language,
        target_language,
        timestamp: Utc::now(),
    };

    history.add_entry(entry);
    save_history(&history)?;

    Ok(())
}

pub fn get_translation_history() -> Result<TranslationHistory> {
    load_history()
}

pub fn clear_translation_history() -> Result<()> {
    let history = TranslationHistory::new();
    save_history(&history)?;
    Ok(())
}
