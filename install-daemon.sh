#!/bin/bash

set -e

echo "==========================================" 
echo "  MELSEC PLC Daemon 설치"
echo "==========================================" 
echo ""

# 루트 권한 확인
if [ "$EUID" -ne 0 ]; then 
    echo "❌ 이 스크립트는 root 권한으로 실행해야 합니다."
    echo "   sudo ./install-daemon.sh 로 실행하세요."
    exit 1
fi

# 현재 디렉토리 확인
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "📦 1. 데몬 바이너리 확인 중..."
if [ ! -f "target/release/melsec-plc-daemon" ]; then
    echo "   target/release/melsec-plc-daemon 파일이 없습니다."
    echo "   cargo build --release --bin melsec-plc-daemon를 먼저 실행하세요."
    exit 1
fi
echo "✓ 바이너리 확인 완료"
echo ""

echo "📂 2. 디렉토리 생성 중..."
mkdir -p /etc/melsec-plc
mkdir -p /var/lib/melsec-plc
mkdir -p /var/log/melsec-plc
chown -R root1:root1 /var/lib/melsec-plc
chown -R root1:root1 /var/log/melsec-plc
echo "✓ 디렉토리 생성 완료"
echo ""

echo "📋 3. 바이너리 복사 중..."
cp target/release/melsec-plc-daemon /usr/local/bin/
chmod +x /usr/local/bin/melsec-plc-daemon
echo "✓ 바이너리 복사 완료: /usr/local/bin/melsec-plc-daemon"
echo ""

echo "⚙️  4. 설정 파일 복사 중..."
if [ -f "/etc/melsec-plc/daemon.env" ]; then
    echo "   기존 설정 파일이 있습니다. 백업합니다..."
    cp /etc/melsec-plc/daemon.env /etc/melsec-plc/daemon.env.backup.$(date +%Y%m%d_%H%M%S)
fi
cp config/daemon.env /etc/melsec-plc/
chmod 644 /etc/melsec-plc/daemon.env
echo "✓ 설정 파일 복사 완료: /etc/melsec-plc/daemon.env"
echo ""

echo "🔧 5. systemd 서비스 설치 중..."
cp systemd/melsec-plc-daemon.service /etc/systemd/system/
chmod 644 /etc/systemd/system/melsec-plc-daemon.service
echo "✓ 서비스 파일 복사 완료: /etc/systemd/system/melsec-plc-daemon.service"
echo ""

echo "🔄 6. systemd 데몬 리로드..."
systemctl daemon-reload
echo "✓ 리로드 완료"
echo ""

echo "🚀 7. 서비스 활성화 및 시작..."
systemctl enable melsec-plc-daemon.service
systemctl start melsec-plc-daemon.service
echo "✓ 서비스 시작 완료"
echo ""

# 상태 확인
sleep 2
echo "📊 서비스 상태:"
systemctl status melsec-plc-daemon.service --no-pager || true
echo ""

echo "==========================================" 
echo "  ✅ 설치 완료!"
echo "==========================================" 
echo ""
echo "사용 방법:"
echo "  상태 확인:  sudo systemctl status melsec-plc-daemon"
echo "  로그 확인:  sudo journalctl -u melsec-plc-daemon -f"
echo "  재시작:     sudo systemctl restart melsec-plc-daemon"
echo "  중지:       sudo systemctl stop melsec-plc-daemon"
echo "  시작:       sudo systemctl start melsec-plc-daemon"
echo ""
echo "설정 파일:   /etc/melsec-plc/daemon.env"
echo "로그 위치:   journalctl -u melsec-plc-daemon"
echo ""
