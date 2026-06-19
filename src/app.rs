//! GUI 本体（egui/eframe）。
//! メイン: メモ一覧（タイトルのみ。日時はホバーで表示）。
//! 編集: 別ウィンドウ（独立OSビューポート）で行う。
use crate::model::Note;
use crate::settings::{available_fonts, font_path_for, Settings, ThemeMode};
use crate::storage;
use chrono::{DateTime, Local};
use eframe::egui;
use egui::{FontId, RichText};
use std::time::{Duration, Instant, SystemTime};

/// 自動保存のデバウンス間隔（クラウド同期フォルダでの書き込み頻発を抑える）
const AUTOSAVE_DEBOUNCE: Duration = Duration::from_millis(800);

pub struct App {
    notes: Vec<Note>,
    settings: Settings,
    selected: usize,
    /// 編集中メモのID（Some=編集ウィンドウを開く）
    editing: Option<String>,
    /// 編集ウィンドウの本文へフォーカスを送る
    focus_editor: bool,
    show_settings: bool,
    current_font: String,
    /// 適用済みのテーマ / 最前面状態（変更検知用）
    current_theme: ThemeMode,
    current_on_top: bool,
    win_pos: Option<[f32; 2]>,
    win_size: Option<[f32; 2]>,
    dirty: bool,
    /// 直近に保存した時刻（自動保存デバウンス用）
    last_saved: Instant,
    /// 認識している notes.json の更新時刻（外部変更検知用）
    last_disk_mtime: Option<SystemTime>,
    /// 前フレームのフォーカス状態（フォーカス復帰検知用）
    was_focused: bool,
    /// 外部変更と未保存編集が衝突した（確認ダイアログ表示）
    external_conflict: bool,
    /// 現在のモニタサイズ（位置クランプ用）
    monitor_size: Option<[f32; 2]>,
    /// 起動時のウィンドウ位置クランプ済みか
    geometry_checked: bool,
}

/// 位置をモニタの可視範囲に収める
fn clamp_pos(pos: [f32; 2], size: [f32; 2], mon: [f32; 2]) -> [f32; 2] {
    let maxx = (mon[0] - size[0]).max(0.0);
    let maxy = (mon[1] - size[1]).max(0.0);
    [pos[0].clamp(0.0, maxx), pos[1].clamp(0.0, maxy)]
}

fn theme_pref(m: ThemeMode) -> egui::ThemePreference {
    match m {
        ThemeMode::System => egui::ThemePreference::System,
        ThemeMode::Dark => egui::ThemePreference::Dark,
        ThemeMode::Light => egui::ThemePreference::Light,
    }
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (notes, _msg) = storage::load_notes();
        let last_disk_mtime = storage::notes_mtime();
        let settings = storage::load_settings();
        apply_font(&cc.egui_ctx, &settings.font_family);
        cc.egui_ctx.set_theme(theme_pref(settings.theme));
        let current_font = settings.font_family.clone();
        let current_theme = settings.theme;
        let current_on_top = settings.always_on_top;
        Self {
            notes,
            settings,
            selected: 0,
            editing: None,
            focus_editor: false,
            show_settings: false,
            current_font,
            current_theme,
            current_on_top,
            win_pos: None,
            win_size: None,
            dirty: false,
            last_saved: Instant::now(),
            last_disk_mtime,
            was_focused: false,
            external_conflict: false,
            monitor_size: None,
            geometry_checked: false,
        }
    }

    fn new_note(&mut self) {
        let order = self.notes.len() as i64;
        let note = Note::new(order);
        let id = note.id.clone();
        self.notes.insert(0, note);
        self.selected = 0;
        self.renumber();
        self.editing = Some(id);
        self.focus_editor = true;
        self.dirty = true;
    }

    fn delete_selected(&mut self) {
        if self.selected < self.notes.len() {
            let removed_id = self.notes[self.selected].id.clone();
            self.notes.remove(self.selected);
            self.renumber();
            if self.selected >= self.notes.len() {
                self.selected = self.notes.len().saturating_sub(1);
            }
            if self.editing.as_deref() == Some(removed_id.as_str()) {
                self.editing = None;
            }
            self.dirty = true;
        }
    }

    fn renumber(&mut self) {
        for (i, n) in self.notes.iter_mut().enumerate() {
            n.manual_order = i as i64;
        }
    }

    fn persist(&mut self) {
        self.settings.window_pos = self.win_pos;
        self.settings.window_size = self.win_size;
        storage::save_notes(&self.notes);
        storage::save_settings(&self.settings);
        // 自分の書き込み後の更新時刻を記録（外部変更と区別するため）
        self.last_disk_mtime = storage::notes_mtime();
    }

    /// ディスクから再読込して現在の状態を置き換える
    fn reload_from_disk(&mut self) {
        let (notes, _msg) = storage::load_notes();
        self.notes = notes;
        if self.selected >= self.notes.len() {
            self.selected = self.notes.len().saturating_sub(1);
        }
        self.last_disk_mtime = storage::notes_mtime();
        self.dirty = false;
    }
}

/// RFC3339 → ローカル時刻 "YYYY/MM/DD HH:MM"
fn fmt_updated(s: &str) -> String {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Local).format("%Y/%m/%d %H:%M").to_string())
        .unwrap_or_else(|_| "—".to_string())
}

fn apply_font(ctx: &egui::Context, family_name: &str) {
    let mut fonts = egui::FontDefinitions::default();
    if let Some(path) = font_path_for(family_name) {
        if let Ok(bytes) = std::fs::read(path) {
            fonts
                .font_data
                .insert("jp".to_owned(), egui::FontData::from_owned(bytes));
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "jp".to_owned());
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .insert(0, "jp".to_owned());
        }
    }
    ctx.set_fonts(fonts);
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ウィンドウ位置・サイズ・モニタ・フォーカスの観測
        let focused = ctx.input(|i| {
            let vp = i.viewport();
            if let Some(o) = vp.outer_rect {
                self.win_pos = Some([o.min.x, o.min.y]);
            }
            if let Some(inner) = vp.inner_rect {
                self.win_size = Some([inner.width(), inner.height()]);
            }
            if let Some(mon) = vp.monitor_size {
                self.monitor_size = Some([mon.x, mon.y]);
            }
            vp.focused.unwrap_or(true)
        });

        // 起動時: ウィンドウ位置を可視範囲へクランプ（モニタ構成変更対策）
        if !self.geometry_checked {
            if let (Some(mon), Some(pos), Some(sz)) =
                (self.monitor_size, self.win_pos, self.win_size)
            {
                let c = clamp_pos(pos, sz, mon);
                if (c[0] - pos[0]).abs() > 1.0 || (c[1] - pos[1]).abs() > 1.0 {
                    ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(
                        c[0], c[1],
                    )));
                }
                self.geometry_checked = true;
            }
        }

        // フォーカス復帰時: notes.json の外部変更を検知
        if focused && !self.was_focused {
            let disk_mtime = storage::notes_mtime();
            if disk_mtime.is_some() && disk_mtime != self.last_disk_mtime {
                if self.dirty {
                    // 未保存の編集あり → 確認ダイアログ
                    self.external_conflict = true;
                } else {
                    // 安全に再読込
                    self.reload_from_disk();
                }
            }
        }
        self.was_focused = focused;

        // テーマ変更の反映（変更時のみ）
        if self.settings.theme != self.current_theme {
            ctx.set_theme(theme_pref(self.settings.theme));
            self.current_theme = self.settings.theme;
        }
        // 常に最前面の反映（変更時のみ）
        if self.settings.always_on_top != self.current_on_top {
            let level = if self.settings.always_on_top {
                egui::WindowLevel::AlwaysOnTop
            } else {
                egui::WindowLevel::Normal
            };
            ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(level));
            self.current_on_top = self.settings.always_on_top;
        }

        // フォントサイズを毎フレーム反映
        let size = self.settings.font_size;
        ctx.style_mut(|s| {
            use egui::FontFamily::Proportional;
            use egui::TextStyle::*;
            s.text_styles = [
                (Heading, FontId::new(size * 1.3, Proportional)),
                (Body, FontId::new(size, Proportional)),
                (Button, FontId::new(size, Proportional)),
                (Monospace, FontId::new(size, egui::FontFamily::Monospace)),
                (Small, FontId::new(size * 0.85, Proportional)),
            ]
            .into();
        });

        // ===== ツールバー（ウィンドウ幅に合わせて多段に折り返す） =====
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.add_space(3.0);
            ui.horizontal_wrapped(|ui| {
                let has = !self.notes.is_empty();
                if ui.button("📄 新規作成").clicked() {
                    self.new_note();
                }
                if ui.add_enabled(has, egui::Button::new("✏ 開く")).clicked() {
                    if let Some(n) = self.notes.get(self.selected) {
                        self.editing = Some(n.id.clone());
                        self.focus_editor = true;
                    }
                }
                if ui.add_enabled(has, egui::Button::new("🗑 削除")).clicked() {
                    self.delete_selected();
                }
                if ui.button("⚙ 設定").clicked() {
                    self.show_settings = !self.show_settings;
                }
            });
            ui.add_space(3.0);
        });

        // ===== メモ一覧（日時はホバーで表示。クリックで編集ウィンドウを開く） =====
        let mut open_id: Option<String> = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.notes.is_empty() {
                ui.add_space(24.0);
                ui.vertical_centered(|ui| {
                    ui.weak("メモがありません。「📄 新規作成」で追加してください。");
                });
                return;
            }
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for i in 0..self.notes.len() {
                        let selected = i == self.selected;
                        let title = self.notes[i].title();
                        let updated = fmt_updated(&self.notes[i].updated_at);
                        let resp = ui
                            .add_sized(
                                [ui.available_width(), 24.0],
                                egui::SelectableLabel::new(selected, RichText::new(title)),
                            )
                            .on_hover_text(format!("更新日時: {updated}"));
                        if resp.clicked() {
                            self.selected = i;
                        }
                        if resp.double_clicked() {
                            self.selected = i;
                            open_id = Some(self.notes[i].id.clone());
                        }
                    }
                });
        });
        if let Some(id) = open_id {
            self.editing = Some(id);
            self.focus_editor = true;
        }

        // ===== 編集ウィンドウ（別ビューポート） =====
        if let Some(id) = self.editing.clone() {
            if let Some(pos) = self.notes.iter().position(|n| n.id == id) {
                let mut close = false;
                let title = self.notes[pos].title();
                let editor_size = self.settings.editor_size.unwrap_or([520.0, 420.0]);
                let mut builder = egui::ViewportBuilder::default()
                    .with_title(format!("メモの編集 — {title}"))
                    .with_inner_size(editor_size)
                    .with_min_inner_size([260.0, 200.0]);
                if let Some(p) = self.settings.editor_pos {
                    // 可視範囲へクランプ（保存位置がモニタ外のとき対策）
                    let p = match self.monitor_size {
                        Some(mon) => clamp_pos(p, editor_size, mon),
                        None => p,
                    };
                    builder = builder.with_position(p);
                }
                ctx.show_viewport_immediate(
                    egui::ViewportId::from_hash_of("note_editor"),
                    builder,
                    |ctx, _class| {
                        // 編集ウィンドウのサイズ・位置を記憶
                        ctx.input(|i| {
                            let vp = i.viewport();
                            if let Some(inner) = vp.inner_rect {
                                self.settings.editor_size =
                                    Some([inner.width(), inner.height()]);
                            }
                            if let Some(outer) = vp.outer_rect {
                                self.settings.editor_pos = Some([outer.min.x, outer.min.y]);
                            }
                        });

                        egui::TopBottomPanel::bottom("editor_bottom").show(ctx, |ui| {
                            ui.add_space(2.0);
                            ui.horizontal(|ui| {
                                let chars = self.notes[pos].task_content.chars().count();
                                ui.weak(format!("{chars} 文字"));
                                ui.weak("｜");
                                ui.weak(format!(
                                    "更新: {}",
                                    fmt_updated(&self.notes[pos].updated_at)
                                ));
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui.button("保存").clicked() {
                                            close = true; // 保存して閉じる（変更は自動保存済み）
                                        }
                                    },
                                );
                            });
                            ui.add_space(2.0);
                        });

                        egui::CentralPanel::default().show(ctx, |ui| {
                            let note = &mut self.notes[pos];
                            let resp = ui.add_sized(
                                ui.available_size(),
                                egui::TextEdit::multiline(&mut note.task_content)
                                    .hint_text("ここにメモを入力…")
                                    .frame(false),
                            );
                            if resp.changed() {
                                note.touch();
                                self.dirty = true;
                            }
                            if self.focus_editor {
                                resp.request_focus();
                                self.focus_editor = false;
                            }
                        });

                        if ctx.input(|i| i.viewport().close_requested()) {
                            close = true;
                        }
                    },
                );
                if close {
                    // 空メモ（空白のみ）は破棄する
                    if let Some(p) = self.notes.iter().position(|n| n.id == id) {
                        if self.notes[p].task_content.trim().is_empty() {
                            self.notes.remove(p);
                            self.renumber();
                            if self.selected >= self.notes.len() {
                                self.selected = self.notes.len().saturating_sub(1);
                            }
                        }
                    }
                    self.editing = None;
                    // 閉じる時は即保存
                    self.persist();
                    self.dirty = false;
                    self.last_saved = Instant::now();
                }
            } else {
                self.editing = None;
            }
        }

        // ===== 外部変更の競合ダイアログ（未保存編集あり） =====
        if self.external_conflict {
            let mut reload = false;
            let mut keep = false;
            egui::Window::new("外部で変更されました")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("notes.json が外部（クラウド同期など）で変更されました。");
                    ui.label("このアプリには未保存の変更があります。");
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        if ui.button("再読込（自分の変更を破棄）").clicked() {
                            reload = true;
                        }
                        if ui.button("自分の変更を保持").clicked() {
                            keep = true;
                        }
                    });
                });
            if reload {
                self.reload_from_disk();
                self.editing = None;
                self.external_conflict = false;
            } else if keep {
                // 自分の変更を維持。以後この外部変更では再確認しない。
                self.last_disk_mtime = storage::notes_mtime();
                self.external_conflict = false;
            }
        }

        // ===== 設定ウィンドウ =====
        if self.show_settings {
            let mut open = self.show_settings;
            egui::Window::new("⚙ 設定")
                .open(&mut open)
                .resizable(false)
                .show(ctx, |ui| {
                    egui::Grid::new("settings_grid")
                        .num_columns(2)
                        .spacing([12.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("フォント");
                            egui::ComboBox::from_id_salt("font_family")
                                .selected_text(self.settings.font_family.clone())
                                .show_ui(ui, |ui| {
                                    for (name, _) in available_fonts() {
                                        if ui
                                            .selectable_value(
                                                &mut self.settings.font_family,
                                                name.to_string(),
                                                name,
                                            )
                                            .changed()
                                        {
                                            self.dirty = true;
                                        }
                                    }
                                });
                            ui.end_row();

                            ui.label("文字サイズ");
                            if ui
                                .add(egui::Slider::new(&mut self.settings.font_size, 10.0..=32.0))
                                .changed()
                            {
                                self.dirty = true;
                            }
                            ui.end_row();

                            ui.label("テーマ");
                            egui::ComboBox::from_id_salt("theme")
                                .selected_text(self.settings.theme.label())
                                .show_ui(ui, |ui| {
                                    for t in [ThemeMode::System, ThemeMode::Dark, ThemeMode::Light] {
                                        if ui
                                            .selectable_value(&mut self.settings.theme, t, t.label())
                                            .changed()
                                        {
                                            self.dirty = true;
                                        }
                                    }
                                });
                            ui.end_row();

                            ui.label("表示");
                            if ui
                                .checkbox(&mut self.settings.always_on_top, "常に最前面に表示")
                                .changed()
                            {
                                self.dirty = true;
                            }
                            ui.end_row();
                        });
                });
            self.show_settings = open;
        }

        // ===== フォント変更の反映 =====
        if self.settings.font_family != self.current_font {
            apply_font(ctx, &self.settings.font_family);
            self.current_font = self.settings.font_family.clone();
        }

        // ===== 終了直前 / 変更時の保存（デバウンス付き自動保存） =====
        if ctx.input(|i| i.viewport().close_requested()) {
            // 終了時は確実に保存
            self.persist();
            self.dirty = false;
        } else if self.dirty && !self.external_conflict {
            // 競合確認中はユーザーの選択まで保存しない（外部変更の上書き防止）
            let elapsed = self.last_saved.elapsed();
            if elapsed >= AUTOSAVE_DEBOUNCE {
                self.persist();
                self.dirty = false;
                self.last_saved = Instant::now();
            } else {
                // 入力が止んでも保存されるよう、残り時間後に再描画を要求
                ctx.request_repaint_after(AUTOSAVE_DEBOUNCE - elapsed);
            }
        }
    }
}
