//! 設定（フォント・ウィンドウ）。settings.json に保存。
//! SimpleTask と settings.json を共有しても壊れないよう、未知フィールドは serde が無視する。
use serde::{Deserialize, Serialize};

/// テーマ設定
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    /// OSのダーク/ライト設定に追従
    #[default]
    System,
    Dark,
    Light,
}

impl ThemeMode {
    pub fn label(self) -> &'static str {
        match self {
            ThemeMode::System => "OSに追従",
            ThemeMode::Dark => "ダーク",
            ThemeMode::Light => "ライト",
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Settings {
    /// フォントファミリー（表示名キー。font_catalog 参照）
    pub font_family: String,
    pub font_size: f32,
    /// テーマ（OS追従 / ダーク / ライト）
    #[serde(default)]
    pub theme: ThemeMode,
    /// 常に最前面に表示
    #[serde(default)]
    pub always_on_top: bool,
    #[serde(default)]
    pub window_pos: Option<[f32; 2]>,
    #[serde(default)]
    pub window_size: Option<[f32; 2]>,
    /// 編集ウィンドウのサイズ
    #[serde(default)]
    pub editor_size: Option<[f32; 2]>,
    /// 編集ウィンドウの位置
    #[serde(default)]
    pub editor_pos: Option<[f32; 2]>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            font_family: "Yu Gothic".to_string(),
            font_size: 16.0,
            theme: ThemeMode::System,
            always_on_top: false,
            window_pos: None,
            window_size: None,
            editor_size: None,
            editor_pos: None,
        }
    }
}

/// 利用可能なフォント候補（表示名 -> システムフォントパス）。日本語・ラテン両対応。
pub fn font_catalog() -> Vec<(&'static str, &'static str)> {
    vec![
        ("Yu Gothic", r"C:\Windows\Fonts\YuGothR.ttc"),
        ("Meiryo", r"C:\Windows\Fonts\meiryo.ttc"),
        ("MS Gothic", r"C:\Windows\Fonts\msgothic.ttc"),
        ("MS Mincho", r"C:\Windows\Fonts\msmincho.ttc"),
        ("BIZ UDGothic", r"C:\Windows\Fonts\BIZ-UDGothicR.ttc"),
        ("BIZ UDMincho", r"C:\Windows\Fonts\BIZ-UDMinchoM.ttc"),
    ]
}

pub fn available_fonts() -> Vec<(&'static str, &'static str)> {
    font_catalog()
        .into_iter()
        .filter(|(_, path)| std::path::Path::new(path).exists())
        .collect()
}

pub fn font_path_for(name: &str) -> Option<&'static str> {
    font_catalog()
        .into_iter()
        .find(|(n, _)| *n == name)
        .map(|(_, p)| p)
}
