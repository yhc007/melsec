#!/bin/bash

echo "==================================="
echo "  PLC D1000 주소 테스트 가이드"
echo "==================================="
echo ""
echo "이 프로그램으로 D1000번부터 데이터를 읽어올 수 있습니다:"
echo ""
echo "1. GUI 프로그램 실행 (그래픽 환경 필요):"
echo "   ./target/release/melsec-plc"
echo "   → '시작 주소' 필드에 '1000'이 기본값으로 설정됨"
echo "   → '연결' 버튼 클릭"
echo "   → '읽기' 버튼 클릭하면 D1000~D1009 데이터 표시"
echo ""
echo "2. TUI 프로그램 실행 (터미널 UI):"
echo "   ./target/release/melsec-plc-tui"
echo "   → 자동으로 D1000, D1002, D1004... D1016 주소 모니터링"
echo "   → Tab 키로 메뉴 이동"
echo "   → 'c' 키로 연결"
echo ""
echo "3. 테스트 프로그램 실행:"
echo "   ./target/release/examples/test_d1000_read"
echo "   → D1000부터 10개 워드를 읽어서 표 형식으로 출력"
echo ""
echo "4. 다른 IP나 포트 사용:"
echo "   export PLC_IP=192.168.1.100"
echo "   export PLC_PORT=6000"
echo "   ./target/release/examples/test_d1000_read"
echo ""
echo "==================================="
echo ""

# 현재 설정 확인
echo "현재 기본 설정:"
echo "  IP: ${PLC_IP:-192.168.21.112}"
echo "  Port: ${PLC_PORT:-5010}"
echo ""

# 실행 선택
read -p "테스트 프로그램을 실행하시겠습니까? (y/n): " choice
if [[ "$choice" == "y" || "$choice" == "Y" ]]; then
    ./target/release/examples/test_d1000_read
fi
