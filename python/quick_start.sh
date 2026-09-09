#!/bin/bash

echo "=========================================="
echo "  Python PLC 프로그램 빠른 시작"
echo "=========================================="
echo ""

# 가상환경 활성화
if [ ! -d "venv" ]; then
    echo "❌ 가상환경이 없습니다. 먼저 ./setup.sh를 실행하세요."
    exit 1
fi

source venv/bin/activate

# 메뉴 표시
while true; do
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  사용 가능한 프로그램"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  1) D1000 데이터 읽기 (1회)"
    echo "  2) D1000 데이터 연속 읽기 (실시간)"
    echo "  3) D1000에 데이터 쓰기"
    echo "  4) GUI 모니터링 실행"
    echo "  5) 환경 설정"
    echo "  0) 종료"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    
    read -p "선택 (0-5): " choice
    
    case $choice in
        1)
            echo ""
            python plc_reader.py
            ;;
        2)
            echo ""
            echo "※ Ctrl+C로 종료할 수 있습니다."
            python plc_reader.py --continuous
            ;;
        3)
            echo ""
            read -p "주소 (예: 1000): " addr
            read -p "값 (예: 1234): " value
            python plc_writer.py $addr $value
            ;;
        4)
            echo ""
            echo "GUI 실행 중..."
            python plc_gui.py
            ;;
        5)
            echo ""
            echo "현재 설정:"
            echo "  PLC_IP   = ${PLC_IP:-192.168.21.112 (기본값)}"
            echo "  PLC_PORT = ${PLC_PORT:-5010 (기본값)}"
            echo ""
            read -p "IP 주소 (Enter=현재값 유지): " new_ip
            read -p "Port (Enter=현재값 유지): " new_port
            
            if [ ! -z "$new_ip" ]; then
                export PLC_IP=$new_ip
                echo "✓ PLC_IP = $new_ip"
            fi
            
            if [ ! -z "$new_port" ]; then
                export PLC_PORT=$new_port
                echo "✓ PLC_PORT = $new_port"
            fi
            ;;
        0)
            echo ""
            echo "종료합니다."
            exit 0
            ;;
        *)
            echo ""
            echo "❌ 잘못된 선택입니다."
            ;;
    esac
    
    read -p "계속하려면 Enter를 누르세요..."
done
