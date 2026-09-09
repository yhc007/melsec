#!/bin/bash
# PLC 연결 진단 스크립트

echo "=========================================="
echo "🔍 PLC 연결 진단 도구"
echo "=========================================="
echo ""

PLC_IP="192.168.21.112"
PLC_PORT="5010"

echo "📋 설정 정보:"
echo "  IP: $PLC_IP"
echo "  Port: $PLC_PORT"
echo ""

# 1. 네트워크 연결 확인
echo "1️⃣ 네트워크 연결 확인..."
if ping -c 2 -W 2 $PLC_IP > /dev/null 2>&1; then
    echo "  ✅ PING 성공 - PLC와 네트워크 연결됨"
else
    echo "  ❌ PING 실패 - PLC에 도달할 수 없음"
    echo "     → IP 주소를 확인하세요: $PLC_IP"
fi
echo ""

# 2. 포트 연결 확인
echo "2️⃣ 포트 연결 확인..."
if timeout 3 bash -c "echo > /dev/tcp/$PLC_IP/$PLC_PORT" 2>/dev/null; then
    echo "  ✅ 포트 $PLC_PORT 열림 - PLC가 응답 가능"
else
    echo "  ❌ 포트 $PLC_PORT 연결 실패"
    echo "     → 포트 번호를 확인하세요"
    echo "     → PLC의 이더넷 통신 설정을 확인하세요"
fi
echo ""

# 3. 로그 파일 확인
echo "3️⃣ 최근 로그 확인..."
if [ -f "tui_debug.log" ]; then
    echo "  📄 최근 오류 메시지:"
    grep -E "(오류|Error|타임아웃|timeout)" tui_debug.log | tail -5 | while read line; do
        echo "     $line"
    done
else
    echo "  ⚠️ 로그 파일이 없습니다"
fi
echo ""

# 4. 권장 사항
echo "=========================================="
echo "💡 문제 해결 방법:"
echo "=========================================="
echo ""
echo "타임아웃이 발생하는 경우:"
echo "  1. PLC의 IP 주소가 $PLC_IP 인지 확인"
echo "  2. PLC의 통신 포트가 $PLC_PORT 인지 확인"
echo "  3. PLC의 이더넷 통신 설정 확인"
echo "     - MC 프로토콜 활성화"
echo "     - 통신 포트 번호"
echo "     - 네트워크 번호"
echo "  4. 방화벽 설정 확인"
echo "  5. PLC 전원 및 네트워크 케이블 확인"
echo ""
echo "타임아웃 설정:"
echo "  - 현재: 10초"
echo "  - 증가 필요 시 src/main_tui.rs 168번 라인 수정"
echo ""
echo "=========================================="
