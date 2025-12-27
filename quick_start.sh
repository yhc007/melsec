#!/bin/bash
# 빠른 실행 가이드

echo "=== MELSEC PLC GUI 프로그램 실행 ==="
echo ""
echo "방법 1: Xvfb 사용 (권장 - headless 서버용)"
echo "  ./run_gui.sh"
echo ""
echo "방법 2: 직접 실행 (GUI 디스플레이가 있는 경우)"
echo "  ./target/release/melsec-plc"
echo ""
echo "방법 3: Cargo로 실행"
echo "  cargo run --release"
echo ""

if [ -z "$DISPLAY" ]; then
    echo "현재 DISPLAY가 설정되지 않았습니다."
    echo "Xvfb 스크립트를 사용하는 것을 권장합니다: ./run_gui.sh"
else
    echo "현재 DISPLAY: $DISPLAY"
    echo "직접 실행을 시도할 수 있습니다."
fi

