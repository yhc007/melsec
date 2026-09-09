#!/bin/bash

echo "=========================================="
echo "  Python PLC 프로그램 설치"
echo "=========================================="
echo ""

# Python 버전 확인
if ! command -v python3 &> /dev/null; then
    echo "✗ Python3가 설치되어 있지 않습니다."
    echo "  설치: sudo apt install python3 python3-pip python3-venv"
    exit 1
fi

echo "✓ Python3 버전: $(python3 --version)"
echo ""

# 가상환경 생성
if [ ! -d "venv" ]; then
    echo "가상환경 생성 중..."
    python3 -m venv venv
    echo "✓ 가상환경 생성 완료"
else
    echo "✓ 가상환경이 이미 존재합니다."
fi

echo ""

# 가상환경 활성화
source venv/bin/activate

# pip 업그레이드
echo "pip 업그레이드 중..."
pip install --upgrade pip -q

# 의존성 설치
echo "의존성 패키지 설치 중..."
pip install -r requirements.txt -q

echo ""
echo "=========================================="
echo "  설치 완료!"
echo "=========================================="
echo ""
echo "사용 방법:"
echo ""
echo "1. 가상환경 활성화:"
echo "   source venv/bin/activate"
echo ""
echo "2. PLC 데이터 읽기 (D1000부터):"
echo "   python plc_reader.py"
echo ""
echo "3. PLC 데이터 연속 읽기:"
echo "   python plc_reader.py --continuous"
echo ""
echo "4. PLC 데이터 쓰기:"
echo "   python plc_writer.py 1000 1234"
echo ""
echo "5. GUI 실행:"
echo "   python plc_gui.py"
echo ""
echo "6. IP/Port 설정:"
echo "   export PLC_IP=192.168.1.100"
echo "   export PLC_PORT=6000"
echo ""
