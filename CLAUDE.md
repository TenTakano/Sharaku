# Sharaku - 開発ガイド

ローカル画像管理・閲覧デスクトップアプリ（Tauri 2 + Svelte 5）

## 技術スタック

- フロントエンド: Svelte 5 + TypeScript + Vite
- バックエンド: Rust + Tauri 2
- DB: SQLite（アプリ内蔵）
- ツールバージョン: `.tool-versions`（Node.js, pnpm）、`rust-toolchain.toml`（Rust）を参照

## 開発コマンド

### アプリ起動（開発モード）

```bash
pnpm run tauri dev
```

**注意**: `pnpm run dev` ではなく `pnpm run tauri dev` を使うこと。
`pnpm run dev` はViteのみ起動し、Tauriアプリは立ち上がらない。

`pnpm run tauri dev` は内部で以下を順に実行する:

1. Vite開発サーバー起動（ポート1420）
2. Rustコンパイル（初回は数分かかる）
3. Tauriウィンドウ起動

### その他のコマンド

| コマンド                                            | 説明                         |
| --------------------------------------------------- | ---------------------------- |
| `pnpm run tauri dev`                                | 開発サーバー + アプリ起動    |
| `pnpm run build`                                    | フロントエンドビルド         |
| `pnpm run tauri build`                              | リリースビルド               |
| `pnpm run check`                                    | Svelte/TypeScript 型チェック |
| `pnpm run lint`                                     | ESLint                       |
| `pnpm run format`                                   | Prettier フォーマット        |
| `cargo check --manifest-path src-tauri/Cargo.toml`  | Rustコンパイルチェック       |
| `cargo test --manifest-path src-tauri/Cargo.toml`   | Rustテスト                   |
| `cargo clippy --manifest-path src-tauri/Cargo.toml` | Rust lint                    |

## UI 確認

UI を変更した場合は `/verify-ui` スキルで表示を確認できる（`pnpm run tauri dev` でアプリ起動中に使用）。

## プロジェクト構成

```
src/                  # フロントエンド（Svelte 5）
  lib/
    components/       # Svelteコンポーネント
    stores/           # Svelte stores
    utils/            # 共通ユーティリティ関数
    thumbnailCache.ts # サムネイルキャッシュ
    types.ts          # TypeScript型定義
  App.svelte          # ルートコンポーネント
  main.ts             # エントリポイント
src-tauri/            # バックエンド（Rust + Tauri 2）
  src/                # Rustソースコード
    commands/         # Tauriコマンド
    tests/            # Rustテスト
  migrations/         # SQLiteマイグレーション
  Cargo.toml
  tauri.conf.json     # Tauri設定
docs/                 # 機能仕様書
```

## 機能仕様書

現在の機能仕様は [docs/README.md](docs/README.md) を起点に整理されている。
