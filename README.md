# RoboMaster Rust Control Library

Rust実装によるRoboMaster S1 CAN制御ライブラリ

## 概要

このライブラリは、DJI RoboMaster S1ロボットをCANバス経由で制御するためのRust実装です。

## 機能

- ✅ **CAN通信**: SocketCANを使用したCANバス通信
- ✅ **ロボット制御**: 移動、ジンバル、LED制御
- ✅ **コマンドビルダー**: 安全なコマンド構築API
- ✅ **エラーハンドリング**: 包括的なエラー処理
- ✅ **非同期サポート**: Tokioベースの非同期処理
- ✅ **ジョイスティック入力**: ゲームパッド入力処理（基本実装）
- ⚠️ **センサーデータ**: 基本的なフレームワーク（完全実装は今後）

## インストール

### 前提条件

```bash
# Ubuntuの場合
sudo apt-get install can-utils

# CANインターフェースのセットアップ command or shell-script
sudo ip link set can0 type can bitrate 1000000
sudo ip link set up can0
# or
./can0_interface_setup.sh
```

### ライブラリの使用

`Cargo.toml`に追加:

```toml
[dependencies]
robomaster-rust = "0.1.0"
```

## 基本的な使用方法

### 基本制御

```rust
use robomaster_rust::{RoboMaster, MovementCommand, LedCommand};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // RoboMasterに接続
    let mut robot = RoboMaster::new("can0").await?;
    
    // 初期化
    robot.initialize().await?;
    
    // LEDを緑色に設定
    robot.control_led(LedCommand::green().color()).await?;
    
    // 前進
    let movement = MovementCommand::new().forward(0.5);
    robot.move_robot(movement.into_params()).await?;
    
    sleep(Duration::from_secs(2)).await;
    
    // 停止
    robot.stop().await?;
    
    // シャットダウン
    robot.shutdown().await?;
    
    Ok(())
}
```

### 高度な制御

```rust
use robomaster_rust::{RoboMaster, MovementCommand, LedCommand};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut robot = RoboMaster::new("can0").await?;
    robot.initialize().await?;
    
    // 複合移動 (前進 + 右ストレーフ + 回転)
    let complex_movement = MovementCommand::new()
        .forward(0.3)
        .strafe_right(0.2)
        .rotate_right(0.1);
    
    robot.move_robot(complex_movement.into_params()).await?;
    
    // カスタムRGB LED
    robot.control_led(LedCommand::rgb(255, 128, 0).color()).await?;
    
    // タッチコマンド送信
    robot.send_touch().await?;
    
    robot.shutdown().await?;
    Ok(())
}
```

## サンプルプログラム

```bash
# 基本制御デモ
cargo run --example basic_control

# シミュレートされたジョイスティック制御
cargo run --example joystick_control

# センサーモニタリング
cargo run --example sensor_monitor
```

## アーキテクチャ

### モジュール構成

```
src/
├── lib.rs           # メインライブラリエントリポイント
├── can/             # CAN通信レイヤー
├── command/         # コマンド構築とプロトコル
├── control/         # 高レベル制御API
├── error.rs         # エラー型定義
├── joystick/        # ジョイスティック入力処理
└── crc/             # CRCチェックサム計算
```


## 開発状況

### 完了済み 
- CAN通信インフラストラクチャ
- 基本的なロボット制御
- コマンドビルダーAPI
- LED制御
- 移動制御
- エラーハンドリング
- 基本的な非同期サポート
- サンプルプログラム
- 統合テスト

### 進行中 
- センサーデータ読み取り


## テスト

```bash
# 全テスト実行
cargo test

# ライブラリテストのみ
cargo test --lib

# 統合テスト
cargo test --test integration_tests

# ベンチマーク
cargo bench
```

**注意**: CANインターフェースが利用できない環境では、ハードウェア依存のテストはスキップされます。

## License

MIT license

### 開発環境セットアップ

```bash
# リポジトリのクローン
git clone https://github.com/IkuoShige/robomaster_s1_rust.git
cd robomaster_s1_rust

# 依存関係のインストール
cargo build

# テスト実行
cargo test

# フォーマット
cargo fmt

# Lint
cargo clippy
```

