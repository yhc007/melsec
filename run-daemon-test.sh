#!/bin/bash

# MELSEC PLC Daemon 테스트 실행 스크립트
# 10초간 실행 후 자동 종료

cd "$(dirname "$0")"

echo "=========================================="
echo "  MELSEC PLC Daemon 테스트 실행"
echo "=========================================="
echo ""

# 빌드 확인
if [ ! -f "target/release/melsec-plc-daemon" ]; then
    echo "빌드 중..."
    cargo build --release --bin melsec-plc-daemon
    echo ""
fi

echo "설정:"
echo "  PLC: 192.168.21.112:5010"
echo "  Kafka: localhost:9092"
echo "  Topic: melsec-plc-data"
echo "  읽기 주소: D1000~D1009"
echo "  읽기 간격: 1초"
echo ""
echo "10초간 실행 후 자동 종료됩니다..."
echo "수동 종료: Ctrl+C"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# 환경 변수 설정 및 실행
PLC_IP=192.168.21.112 \
PLC_PORT=5010 \
KAFKA_BROKERS=localhost:9092 \
KAFKA_TOPIC=melsec-plc-data \
READ_INTERVAL_MS=1000 \
START_ADDRESS=1000 \
READ_COUNT=10 \
RUST_LOG=info \
timeout 10 ./target/release/melsec-plc-daemon

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "테스트 완료!"
echo ""
echo "systemd 서비스로 설치하려면:"
echo "  sudo ./install-daemon.sh"
echo ""
