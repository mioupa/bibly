# bibly

簡単な書籍管理デスクトップアプリ（Tauri + Vue）。ISBNから国立国会図書館（NDL）APIやGoogle Books API、楽天ブックスAPIを使って自動入力する機能を備えています。設定画面で使用するAPIを選択できます。

## 主要ファイル
- フロントエンド（Vue 3）
  - UIエントリ: [src/App.vue](src/App.vue)
  - 追加フォーム: [src/components/AddBookForm.vue](src/components/AddBookForm.vue)
  - 型定義: [src/types.ts](src/types.ts)
- バックエンド（Rust / Tauri）
  - Tauri設定 & 依存: [src-tauri/Cargo.toml](src-tauri/Cargo.toml)
  - アプリ起動: [src-tauri/src/main.rs](src-tauri/src/main.rs)
  - NDL検索コマンド: [`commands::fetch_book_info_from_ndl`](src-tauri/src/commands/mod.rs) （Tauriコマンド）
  - Google Books検索コマンド: [`commands::fetch_book_info_from_google_books`](src-tauri/src/commands/mod.rs)
  - 楽天ブックス検索コマンド: [`commands::fetch_book_info_from_rakuten`](src-tauri/src/commands/mod.rs)

## 必要環境
- Node.js（推奨: LTS）
- pnpm
- Rust toolchain（stable）および cargo

## 開発（ローカル起動）
1. ルートで依存インストール:
   ```sh
   pnpm install
   ```
2. 開発モードで起動（フロント + Tauriバックエンド）:
   ```sh
   pnpm tauri dev
   ```
   （環境により `pnpm dev` + 別ターミナルで `pnpm tauri dev` の手順が必要な場合あり）

## ビルド（配布用）
1. フロントビルド:
   ```sh
   pnpm build
   ```
2. Tauriネイティブバイナリ生成:
   ```sh
   cd src-tauri
   cargo build --release
   ```
   または Tauri のビルドスクリプトを使ってパッケージ化してください。

## ISBN 自動入力について
- UI: [src/components/AddBookForm.vue](src/components/AddBookForm.vue) の「自動入力 (ISBN)」タブから利用可能。
- 実際の検索は Tauri コマンド [`commands::fetch_book_info_from_ndl`](src-tauri/src/commands/mod.rs) または [`commands::fetch_book_info_from_google_books`](src-tauri/src/commands/mod.rs)、[`commands::fetch_book_info_from_rakuten`](src-tauri/src/commands/mod.rs) に委譲されます。設定で選択したAPIに応じてバックエンドで API を呼び出し、タイトル・著者・出版社を返します。
- バックエンドの依存は [src-tauri/Cargo.toml](src-tauri/Cargo.toml) を確認してください（例: reqwest, quick-xml）。

## 注意点 / トラブルシューティング
- Rust の依存を追加・変更した場合は `cd src-tauri && cargo clean && cargo build` を行ってください。
- Windows 環境でビルドエラーが出る場合、`src-tauri/Cargo.toml` のcrate設定や features を確認してください。
- Tauri のバージョンに依存する設定があるため、問題があれば `src-tauri/tauri.conf.json` と `src-tauri/Cargo.toml` を参照してください。
