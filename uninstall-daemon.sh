#!/bin/bash

set -e

# 사용법: sudo ./uninstall-daemon.sh [1|2|all]
#   1    - 1호기(melsec-plc-daemon)만 제거
#   2    - 2호기(melsec-plc-daemon-2)만 제거
#   all  - 설치된 모든 인스턴스 제거 (기본값)
#
# 바이너리와 /etc/melsec-plc 는 두 인스턴스가 공유하므로,
# 남아 있는 인스턴스가 하나라도 있으면 제거하지 않는다.

TARGET="${1:-all}"

BIN_PATH="/usr/local/bin/melsec-plc-daemon"
CONF_DIR="/etc/melsec-plc"
DATA_DIR="/var/lib/melsec-plc"
LOG_DIR="/var/log/melsec-plc"
UNIT_DIR="/etc/systemd/system"

ALL_INSTANCES="1 2"

# 인스턴스 번호 -> 서비스명
instance_service() {
    case "$1" in
        1) echo "melsec-plc-daemon.service" ;;
        2) echo "melsec-plc-daemon-2.service" ;;
    esac
}

# 인스턴스 번호 -> 설정 파일
instance_env() {
    case "$1" in
        1) echo "$CONF_DIR/daemon.env" ;;
        2) echo "$CONF_DIR/daemon-2.env" ;;
    esac
}

# 유닛 파일이 있으면 설치된 것으로 본다
instance_installed() {
    [ -f "$UNIT_DIR/$(instance_service "$1")" ]
}

echo "=========================================="
echo "  MELSEC PLC Daemon 제거"
echo "=========================================="
echo ""

# 루트 권한 확인
if [ "$EUID" -ne 0 ]; then
    echo "❌ 이 스크립트는 root 권한으로 실행해야 합니다."
    echo "   sudo ./uninstall-daemon.sh [1|2|all] 로 실행하세요."
    exit 1
fi

# 인자 검증
case "$TARGET" in
    1|2|all) ;;
    *)
        echo "❌ 알 수 없는 인자: $TARGET"
        echo "   사용법: sudo ./uninstall-daemon.sh [1|2|all]"
        exit 1
        ;;
esac

# 제거 대상 결정
if [ "$TARGET" = "all" ]; then
    TARGETS="$ALL_INSTANCES"
else
    TARGETS="$TARGET"
fi

# 실제로 설치된 것만 남긴다
INSTALLED=""
for i in $TARGETS; do
    if instance_installed "$i"; then
        INSTALLED="$INSTALLED $i"
    fi
done

if [ -z "$INSTALLED" ]; then
    echo "제거 대상 인스턴스가 설치되어 있지 않습니다. (대상: $TARGET)"
    echo ""
    for i in $ALL_INSTANCES; do
        if instance_installed "$i"; then
            echo "   참고: ${i}호기($(instance_service "$i"))는 설치되어 있습니다."
        fi
    done
    exit 0
fi

echo "제거 대상:"
for i in $INSTALLED; do
    echo "   ${i}호기 - $(instance_service "$i")"
done
echo ""

# 1~3. 인스턴스별 중지 / 비활성화 / 유닛 제거
for i in $INSTALLED; do
    SERVICE="$(instance_service "$i")"

    echo "⏹️  ${i}호기 서비스 중지 중..."
    if systemctl is-active --quiet "$SERVICE"; then
        systemctl stop "$SERVICE"
        echo "✓ 서비스 중지 완료"
    else
        echo "   (서비스가 실행 중이 아닙니다)"
    fi

    echo "🔓 ${i}호기 서비스 비활성화 중..."
    if systemctl is-enabled --quiet "$SERVICE" 2>/dev/null; then
        systemctl disable "$SERVICE"
        echo "✓ 서비스 비활성화 완료"
    else
        echo "   (서비스가 활성화되어 있지 않습니다)"
    fi

    if [ -f "$UNIT_DIR/$SERVICE" ]; then
        rm -f "$UNIT_DIR/$SERVICE"
        echo "✓ 서비스 파일 제거: $UNIT_DIR/$SERVICE"
    fi
    echo ""
done

# systemd 리로드
echo "🔄 systemd 데몬 리로드..."
systemctl daemon-reload
systemctl reset-failed
echo "✓ 리로드 완료"
echo ""

# 제거 후에도 남아 있는 인스턴스 확인
REMAINING=""
for i in $ALL_INSTANCES; do
    if instance_installed "$i"; then
        REMAINING="$REMAINING $i"
    fi
done

# 제거한 인스턴스의 설정 파일 (공유 디렉터리 안에 있으므로 개별 삭제)
for i in $INSTALLED; do
    ENV_FILE="$(instance_env "$i")"
    if [ -f "$ENV_FILE" ]; then
        BACKUP_FILE="/tmp/$(basename "$ENV_FILE").backup.$(date +%Y%m%d_%H%M%S)"
        cp "$ENV_FILE" "$BACKUP_FILE"
        rm -f "$ENV_FILE"
        echo "✓ 설정 파일 제거: $ENV_FILE"
        echo "   백업 생성: $BACKUP_FILE"
    fi
done
echo ""

# 공유 자원은 남은 인스턴스가 없을 때만 제거
if [ -n "$REMAINING" ]; then
    echo "ℹ️  아래 인스턴스가 남아 있어 공유 자원은 유지합니다:"
    for i in $REMAINING; do
        echo "   ${i}호기 - $(instance_service "$i")"
    done
    echo ""
    echo "   유지되는 공유 자원:"
    echo "     $BIN_PATH"
    echo "     $CONF_DIR"
    echo "     $DATA_DIR"
    echo ""
    echo "=========================================="
    echo "  ✅ 제거 완료! (일부 인스턴스 유지)"
    echo "=========================================="
    echo ""
    exit 0
fi

# 여기부터는 남은 인스턴스가 없는 경우
echo "🗑️  공유 자원 제거..."
if [ -f "$BIN_PATH" ]; then
    rm -f "$BIN_PATH"
    echo "✓ 바이너리 제거: $BIN_PATH"
fi
echo ""

# 설정 디렉터리 제거 확인
if [ -d "$CONF_DIR" ]; then
    echo "❓ 설정 디렉터리도 제거하시겠습니까?"
    echo "   $CONF_DIR"
    echo ""
    read -p "제거하시겠습니까? (y/N): " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        # 남아 있는 파일(수동 생성분 등)이 있으면 함께 백업
        for f in "$CONF_DIR"/*; do
            [ -f "$f" ] || continue
            BACKUP_FILE="/tmp/$(basename "$f").backup.$(date +%Y%m%d_%H%M%S)"
            cp "$f" "$BACKUP_FILE"
            echo "   백업 생성: $BACKUP_FILE"
        done
        rm -rf "$CONF_DIR"
        echo "✓ 설정 디렉터리 제거: $CONF_DIR"
    else
        echo "   설정 디렉터리를 유지합니다."
    fi
    echo ""
fi

# 데이터 디렉터리 제거 확인
echo "❓ 데이터 디렉터리도 제거하시겠습니까?"
echo "   $DATA_DIR"
echo "   $LOG_DIR"
echo ""
read -p "제거하시겠습니까? (y/N): " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]; then
    if [ -d "$DATA_DIR" ]; then
        rm -rf "$DATA_DIR"
        echo "✓ 데이터 디렉터리 제거: $DATA_DIR"
    fi
    if [ -d "$LOG_DIR" ]; then
        rm -rf "$LOG_DIR"
        echo "✓ 로그 디렉터리 제거: $LOG_DIR"
    fi
else
    echo "   데이터 디렉터리를 유지합니다."
fi
echo ""

echo "=========================================="
echo "  ✅ 제거 완료!"
echo "=========================================="
echo ""
