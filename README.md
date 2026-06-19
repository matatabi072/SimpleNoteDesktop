# SimpleNote Desktop

軽量・軽快・環境非依存なローカルメモ帳アプリ。Rust + [egui/eframe](https://github.com/emilk/egui) 製の **単一実行ファイル（約3.2MB）** で、インストール不要・ランタイム不要で動作します。姉妹アプリ **SimpleTask Desktop** とデータ構造を共通化しています。

> A lightweight, portable, dependency-free desktop memo app written in Rust (egui). Single ~3.2MB executable, no installer, no runtime required.

---

## ✨ 特徴

- **完全ポータブル / インストール不要** — 実行ファイルとデータ（`notes.json` / `settings.json`）が同一フォルダで完結。
- **環境非依存** — OpenGL(glow) 描画のネイティブGUI。WebView2 等のランタイム不要。
- **クラウド同期対応** — 保存フォルダを OneDrive / Dropbox / Google Drive 等に置くだけ。**外部変更を検知して自動再読込**（未保存時は確認）。
- **安定** — JSON破損を検知し、自動バックアップ（`notes.backup.json`）から復元。原子的保存（tmp→rename）。
- **二重起動防止** — 既に起動中なら既存ウィンドウを前面化。

## 🧩 機能

| 区分 | 内容 |
|------|------|
| メモ一覧 | タイトル（先頭行）を表示。**更新日時はマウスホバーで表示** |
| 編集 | **別ウィンドウ**で自由記述（独立OSウィンドウ・位置/サイズを記憶） |
| 操作 | ツールバー（新規作成 / 開く / 削除 / 設定）。**ウィンドウ幅に応じて多段折り返し** |
| 自動保存 | 入力は自動保存（書き込み頻度を抑えるデバウンス付き）。空メモは閉じる時に自動破棄 |
| テーマ | OSに追従 / ダーク / ライト を選択 |
| 表示 | 常に最前面に表示（オン/オフ） |
| カスタマイズ | フォントファミリー・サイズ（即時反映） |
| ウィンドウ | メイン・編集ウィンドウとも位置/サイズを記憶。可視範囲へ自動クランプ |

## 🚀 入手・起動

### ビルド済みバイナリ
[Releases](../../releases) から `simplenote.exe` をダウンロードし、任意のフォルダに置いてダブルクリック。データは同じフォルダに作成されます。

### ソースからビルド
```bash
cargo build --release   # -> target/release/simplenote.exe
cargo test              # データ層の単体テスト
```

## 🖱 操作方法

| 操作 | 方法 |
|------|------|
| 新規メモ | 「📄 新規作成」→ 編集ウィンドウが開く |
| 編集 | 一覧をダブルクリック、または選択して「✏ 開く」 |
| 更新日時の確認 | 一覧項目にマウスを合わせる（ツールチップ） |
| 保存して閉じる | 編集ウィンドウの「保存」（入力中も自動保存済み） |
| 削除 | 一覧で選択して「🗑 削除」 |
| 設定 | 「⚙ 設定」（フォント / サイズ / テーマ / 常に最前面） |

## ☁ クラウド同期について

実行ファイルと同階層の `notes.json` を、クラウド同期フォルダに置いて運用します。別端末で更新された場合、ウィンドウへ戻った時（フォーカス復帰時）に変更を検知し、未保存がなければ自動で再読込、未保存があれば再読込/保持を確認します。

> 注: 現状の競合解決は「全体再読込 or 自分優先」の粒度です。`updatedAt` によるレコード単位マージや削除同期は将来対応予定です。

## 🗂 データ構造（`notes.json`）

SimpleTask Desktop の `tasks.json` と**同一スキーマ**。統合時にメモをそのままタスクとして認識できます。

```json
{
  "id": "uuid",
  "googleTaskId": null,
  "taskContent": "メモの内容",
  "isCompleted": false,
  "scheduledDateTime": null,
  "priority": "none",
  "manualOrder": 0,
  "updatedAt": "2026-06-19T00:00:00Z"
}
```

## 🏗 アーキテクチャ

| ファイル | 役割 |
|----------|------|
| `src/model.rs` | データモデル（SimpleTask 互換スキーマ） |
| `src/storage.rs` | ローカル保存・破損検知・バックアップ・更新時刻取得 |
| `src/settings.rs` | 設定（フォント・テーマ・最前面・ウィンドウ） |
| `src/app.rs` | GUI（egui） |
| `src/main.rs` | エントリ・二重起動防止・ウィンドウ初期化 |

## 🛠 技術スタック
- 言語: Rust（GNU toolchain）
- GUI: egui / eframe 0.29（glow / OpenGL）
- データ: serde_json / 日時: chrono / 識別子: uuid

## 📄 ライセンス
[MIT License](LICENSE)
