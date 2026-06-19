//! ローカルストレージ。実行ファイルと同一フォルダに保存（完全ポータブル）。
//! クラウド同期はユーザーがこのフォルダをOneDrive/Dropbox等に置くことで実現。
//! 破損検知と自動バックアップを備える。
use crate::model::Note;
use crate::settings::Settings;
use std::fs;
use std::path::PathBuf;

fn data_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn notes_path() -> PathBuf {
    data_dir().join("notes.json")
}
fn backup_path() -> PathBuf {
    data_dir().join("notes.backup.json")
}
fn settings_path() -> PathBuf {
    data_dir().join("settings.json")
}

/// notes.json の最終更新時刻（外部変更の検知に使用）
pub fn notes_mtime() -> Option<std::time::SystemTime> {
    fs::metadata(notes_path())
        .ok()
        .and_then(|m| m.modified().ok())
}

/// メモ読込。破損時はバックアップから自動復元。
pub fn load_notes() -> (Vec<Note>, Option<String>) {
    let p = notes_path();
    let raw = match fs::read_to_string(&p) {
        Ok(s) => s,
        Err(_) => return (Vec::new(), None),
    };
    match serde_json::from_str::<Vec<Note>>(&raw) {
        Ok(v) => (v, None),
        Err(e) => {
            if let Ok(bs) = fs::read_to_string(backup_path()) {
                if let Ok(v) = serde_json::from_str::<Vec<Note>>(&bs) {
                    let _ = fs::copy(&p, data_dir().join("notes.corrupt.json"));
                    return (
                        v,
                        Some("notes.json が破損していたためバックアップから復元しました。".to_string()),
                    );
                }
            }
            (
                Vec::new(),
                Some(format!("notes.json を読み込めませんでした（新規作成します）: {e}")),
            )
        }
    }
}

/// メモ保存。直前の正常データをバックアップへ退避してから原子的に書き込む。
pub fn save_notes(notes: &[Note]) {
    let p = notes_path();
    let json = match serde_json::to_string_pretty(notes) {
        Ok(j) => j,
        Err(_) => return,
    };
    let tmp = data_dir().join("notes.json.tmp");
    if fs::write(&tmp, &json).is_err() {
        return;
    }
    if p.exists() {
        let _ = fs::copy(&p, backup_path());
    }
    let _ = fs::rename(&tmp, &p);
}

pub fn load_settings() -> Settings {
    match fs::read_to_string(settings_path()) {
        Ok(s) => serde_json::from_str::<Settings>(&s).unwrap_or_default(),
        Err(_) => Settings::default(),
    }
}

pub fn save_settings(settings: &Settings) {
    if let Ok(json) = serde_json::to_string_pretty(settings) {
        let _ = fs::write(settings_path(), json);
    }
}
