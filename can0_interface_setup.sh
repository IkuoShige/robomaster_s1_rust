#!/bin/bash
# CAN0インターフェースセットアップスクリプト

echo "CAN0インターフェースを設定中..."

# 既存のcan0インターフェースがあれば停止
sudo ip link set down can0 2>/dev/null

# CANインターフェースを設定（ビットレート1Mbps）
sudo ip link set can0 type can bitrate 1000000

# 送信キューサイズを設定（インターフェースをアップする前に）
sudo ip link set can0 txqueuelen 10000

# インターフェースを有効化
sudo ip link set up can0

echo "CAN0インターフェースの設定完了"
echo "インターフェース情報:"
ip link show can0
echo ""
echo "CANの詳細情報:"
ip -details link show can0