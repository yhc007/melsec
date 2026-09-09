#!/bin/bash
# PLC TUI 프로그램 빠른 테스트 스크립트

echo "=========================================="
echo "PLC TUI 프로그램 테스트"
echo "=========================================="
echo ""
echo "📋 설정 정보:"
echo "  - IP: 192.168.21.112"
echo "  - Port: 5010"
echo "  - 읽기 주소: D7000, D7002, D7004, D7006, D7008,"
echo "              D7010, D7012, D7014, D7016"
echo ""
echo "🚀 실행 파일:"
echo "  /home/root1/Work/RS_PLC/target/release/melsec-plc-tui"
echo ""
echo "⌨️  주요 키 명령어:"
echo "  C - PLC 연결"
echo "  R - 데이터 읽기"
echo "  A - 자동 읽기 On/Off"
echo "  Q - 종료"
echo ""
echo "📊 데이터 확인 방법:"
echo "  1. TUI 화면 하단 테이블에서 실시간 확인"
echo "  2. 로그 파일: tail -f tui_debug.log"
echo ""
echo "=========================================="
echo ""
read -p "프로그램을 실행하시겠습니까? (y/N) " answer
if [ "$answer" = "y" ] || [ "$answer" = "Y" ]; then
    cd /home/root1/Work/RS_PLC
    ./target/release/melsec-plc-tui
fi
