//! データモデル。SimpleTask Desktop とスキーマを共通化し、
//! 将来の統合時にメモをそのままタスクとして認識できる構造とする。
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 重要度（SimpleTask 互換。メモは既定で none）
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    High,
    Medium,
    Low,
    None,
}

fn default_priority() -> Priority {
    Priority::None
}

/// メモ1件。フィールド名は SimpleTask の Task と完全一致（notes.json / tasks.json 共通）。
#[derive(Serialize, Deserialize, Clone)]
pub struct Note {
    pub id: String,
    #[serde(rename = "googleTaskId", default)]
    pub google_task_id: Option<String>,
    /// メモ本文（タスクとしては内容欄）
    #[serde(rename = "taskContent")]
    pub task_content: String,
    #[serde(rename = "isCompleted", default)]
    pub is_completed: bool,
    /// メモは期限なし（null）。SimpleTask 統合時に利用。
    #[serde(rename = "scheduledDateTime", default)]
    pub scheduled_date_time: Option<chrono::NaiveDateTime>,
    #[serde(default = "default_priority")]
    pub priority: Priority,
    #[serde(rename = "manualOrder", default)]
    pub manual_order: i64,
    #[serde(rename = "updatedAt", default)]
    pub updated_at: String,
}

impl Note {
    pub fn new(order: i64) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            google_task_id: None,
            task_content: String::new(),
            is_completed: false,
            scheduled_date_time: None,
            priority: Priority::None,
            manual_order: order,
            updated_at: Utc::now().to_rfc3339(),
        }
    }

    /// 変更時に呼び、競合解決用タイムスタンプを更新する。
    pub fn touch(&mut self) {
        self.updated_at = Utc::now().to_rfc3339();
    }

    /// 一覧表示用のタイトル（先頭行）
    pub fn title(&self) -> String {
        let first = self
            .task_content
            .lines()
            .map(|l| l.trim())
            .find(|l| !l.is_empty())
            .unwrap_or("");
        if first.is_empty() {
            "(無題)".to_string()
        } else {
            first.chars().take(28).collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shares_simpletask_schema() {
        let mut n = Note::new(0);
        n.task_content = "買い物リスト\n牛乳".to_string();
        let json = serde_json::to_string(&n).unwrap();
        // SimpleTask と同じフィールド名で直列化される
        assert!(json.contains("\"taskContent\":"));
        assert!(json.contains("\"googleTaskId\":null"));
        assert!(json.contains("\"scheduledDateTime\":null"));
        assert!(json.contains("\"priority\":\"none\""));
        assert!(json.contains("\"manualOrder\":0"));
        assert_eq!(n.title(), "買い物リスト");
    }

    #[test]
    fn reads_simpletask_task() {
        // SimpleTask が書いた tasks.json 形式も読める
        let sample = r#"[{
            "id":"x","googleTaskId":null,"taskContent":"会議メモ",
            "isCompleted":false,"scheduledDateTime":"2026-06-20T15:00:00",
            "priority":"high","manualOrder":2,"updatedAt":"2026-06-19T00:00:00Z"
        }]"#;
        let v: Vec<Note> = serde_json::from_str(sample).unwrap();
        assert_eq!(v[0].task_content, "会議メモ");
    }
}
