# Sharaku - 開発ガイド

ローカル画像管理・閲覧デスクトップアプリ（Tauri 2 + Svelte 5）

## 技術スタック

- フロントエンド: Svelte 5 + TypeScript + Vite
- バックエンド: Rust + Tauri 2
- DB: SQLite（アプリ内蔵）
- ツールバージョン: `.tool-versions`（Node.js）、`rust-toolchain.toml`（Rust）を参照

## 開発コマンド

### アプリ起動（開発モード）

```bash
npm run tauri dev
```

**注意**: `npm run dev` ではなく `npm run tauri dev` を使うこと。
`npm run dev` はViteのみ起動し、Tauriアプリは立ち上がらない。

`npm run tauri dev` は内部で以下を順に実行する:

1. Vite開発サーバー起動（ポート1420）
2. Rustコンパイル（初回は数分かかる）
3. Tauriウィンドウ起動

### その他のコマンド

| コマンド                                            | 説明                         |
| --------------------------------------------------- | ---------------------------- |
| `npm run tauri dev`                                 | 開発サーバー + アプリ起動    |
| `npm run build`                                     | フロントエンドビルド         |
| `npm run tauri build`                               | リリースビルド               |
| `npm run check`                                     | Svelte/TypeScript 型チェック |
| `npm run lint`                                      | ESLint                       |
| `npm run format`                                    | Prettier フォーマット        |
| `cargo check --manifest-path src-tauri/Cargo.toml`  | Rustコンパイルチェック       |
| `cargo test --manifest-path src-tauri/Cargo.toml`   | Rustテスト                   |
| `cargo clippy --manifest-path src-tauri/Cargo.toml` | Rust lint                    |

## UI 確認

UI を変更した場合は `/verify-ui` スキルで表示を確認できる（`npm run tauri dev` でアプリ起動中に使用）。

## プロジェクト構成

```
src/                  # フロントエンド（Svelte 5）
  lib/
    components/       # Svelteコンポーネント
    stores/           # Svelte stores
    types.ts          # TypeScript型定義
  App.svelte          # ルートコンポーネント
  main.ts             # エントリポイント
src-tauri/            # バックエンド（Rust + Tauri 2）
  src/                # Rustソースコード
  migrations/         # SQLiteマイグレーション
  tests/              # Rustテスト
  Cargo.toml
  tauri.conf.json     # Tauri設定
```
