#!/bin/bash

set -e

echo "=========================================="
echo "  MELSEC PLC Daemon 제거"
echo "=========================================="
echo ""

# 루트 권한 확인
if [ "$EUID" -ne 0 ]; then 
    echo "❌ 이 스크립트는 root 권한으로 실행해야 합니다."
    echo "   sudo ./uninstall-daemon.sh 로 실행하세요."
    exit 1
fi

echo "⏹️  1. 서비스 중지 중..."
if systemctl is-active --quiet melsec-plc-daemon.service; then
    systemctl stop melsec-plc-daemon.service
    echo "✓ 서비스 중지 완료"
else
    echo "   (서비스가 실행 중이 아닙니다)"
fi
echo ""

echo "🔓 2. 서비스 비활성화 중..."
if systemctl is-enabled --quiet melsec-plc-daemon.service 2>/dev/null; then
    systemctl disable melsec-plc-daemon.service
    echo "✓ 서비스 비활성화 완료"
else
    echo "   (서비스가 활성화되어 있지 않습니다)"
fi
echo ""

echo "🗑️  3. 파일 제거 중..."

# 바이너리 제거
if [ -f "/usr/local/bin/melsec-plc-daemon" ]; then
    rm -f /usr/local/bin/melsec-plc-daemon
    echo "✓ 바이너리 제거: /usr/local/bin/melsec-plc-daemon"
fi

# systemd 서비스 파일 제거
if [ -f "/etc/systemd/system/melsec-plc-daemon.service" ]; then
    rm -f /etc/systemd/system/melsec-plc-daemon.service
    echo "✓ 서비스 파일 제거: /etc/systemd/system/melsec-plc-daemon.service"
fi

echo ""

# systemd 리로드
echo "🔄 4. systemd 데몬 리로드..."
systemctl daemon-reload
systemctl reset-failed
echo "✓ 리로드 완료"
echo ""

# 설정 파일 제거 확인
echo "❓ 설정 파일도 제거하시겠습니까?"
echo "   /etc/melsec-plc/daemon.env"
echo ""
read -p "제거하시겠습니까? (y/N): " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]; then
    if [ -d "/etc/melsec-plc" ]; then
        # 백업 생성
        if [ -f "/etc/melsec-plc/daemon.env" ]; then
            BACKUP_FILE="/tmp/melsec-plc-daemon.env.backup.$(date +%Y%m%d_%H%M%S)"
            cp /etc/melsec-plc/daemon.env "$BACKUP_FILE"
            echo "   백업 생성: $BACKUP_FILE"
        fi
        rm -rf /etc/melsec-plc
        echo "✓ 설정 디렉토리 제거: /etc/melsec-plc"
    fi
else
    echo "   설정 파일을 유지합니다."
fi
echo ""

# 데이터 디렉토리 제거 확인
echo "❓ 데이터 디렉토리도 제거하시겠습니까?"
echo "   /var/lib/melsec-plc"
echo "   /var/log/melsec-plc"
echo ""
read -p "제거하시겠습니까? (y/N): " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]; then
    if [ -d "/var/lib/melsec-plc" ]; then
        rm -rf /var/lib/melsec-plc
        echo "✓ 데이터 디렉토리 제거: /var/lib/melsec-plc"
    fi
    if [ -d "/var/log/melsec-plc" ]; then
        rm -rf /var/log/melsec-plc
        echo "✓ 로그 디렉토리 제거: /var/log/melsec-plc"
    fi
else
    echo "   데이터 디렉토리를 유지합니다."
fi
echo ""

echo "=========================================="
echo "  ✅ 제거 완료!"
echo "=========================================="
echo ""
