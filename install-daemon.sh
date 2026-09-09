#!/bin/bash

set -e

# 사용법: sudo ./install-daemon.sh [1|2|all]
#   1    - 1호기(melsec-plc-daemon)만 설치
#   2    - 2호기(melsec-plc-daemon-2)만 설치
#   all  - 두 인스턴스 모두 설치 (기본값)
#
# 바이너리와 /etc/melsec-plc 는 두 인스턴스가 공유한다.

TARGET="${1:-all}"

BIN_PATH="/usr/local/bin/melsec-plc-daemon"
BIN_SRC="target/release/melsec-plc-daemon"
CONF_DIR="/etc/melsec-plc"
DATA_DIR="/var/lib/melsec-plc"
LOG_DIR="/var/log/melsec-plc"
UNIT_DIR="/etc/systemd/system"
RUN_AS="root1"

ALL_INSTANCES="1 2"

instance_service() {
    case "$1" in
        1) echo "melsec-plc-daemon.service" ;;
        2) echo "melsec-plc-daemon-2.service" ;;
    esac
}

instance_env_name() {
    case "$1" in
        1) echo "daemon.env" ;;
        2) echo "daemon-2.env" ;;
    esac
}

echo "=========================================="
echo "  MELSEC PLC Daemon 설치"
echo "=========================================="
echo ""

# 루트 권한 확인
if [ "$EUID" -ne 0 ]; then
    echo "❌ 이 스크립트는 root 권한으로 실행해야 합니다."
    echo "   sudo ./install-daemon.sh [1|2|all] 로 실행하세요."
    exit 1
fi

# 인자 검증
case "$TARGET" in
    1|2|all) ;;
    *)
        echo "❌ 알 수 없는 인자: $TARGET"
        echo "   사용법: sudo ./install-daemon.sh [1|2|all]"
        exit 1
        ;;
esac

if [ "$TARGET" = "all" ]; then
    TARGETS="$ALL_INSTANCES"
else
    TARGETS="$TARGET"
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "📦 1. 설치 대상 및 원본 파일 확인 중..."
if [ ! -f "$BIN_SRC" ]; then
    echo "   $BIN_SRC 파일이 없습니다."
    echo "   cargo build --release --bin melsec-plc-daemon 를 먼저 실행하세요."
    exit 1
fi

# 대상 인스턴스의 설정/유닛 원본이 모두 있는지 먼저 확인 (부분 설치 방지)
for i in $TARGETS; do
    ENV_SRC="config/$(instance_env_name "$i")"
    UNIT_SRC="systemd/$(instance_service "$i")"
    if [ ! -f "$ENV_SRC" ]; then
        echo "   ${i}호기 설정 원본이 없습니다: $ENV_SRC"
        exit 1
    fi
    if [ ! -f "$UNIT_SRC" ]; then
        echo "   ${i}호기 유닛 원본이 없습니다: $UNIT_SRC"
        exit 1
    fi
done

echo "✓ 확인 완료"
echo "   설치 대상:"
for i in $TARGETS; do
    echo "     ${i}호기 - $(instance_service "$i")"
done
echo ""

echo "📂 2. 디렉터리 생성 중..."
mkdir -p "$CONF_DIR" "$DATA_DIR" "$LOG_DIR"
chown -R "$RUN_AS:$RUN_AS" "$DATA_DIR"
chown -R "$RUN_AS:$RUN_AS" "$LOG_DIR"
echo "✓ 디렉터리 생성 완료"
echo ""

echo "📋 3. 바이너리 설치 중..."
# 실행 중인 바이너리에 직접 cp 하면 ETXTBSY(Text file busy)로 실패한다.
# 임시 파일로 복사한 뒤 mv 로 교체하면 inode가 바뀌므로 실행 중이어도 안전하다.
TMP_BIN="$(mktemp "${BIN_PATH}.XXXXXX")"
cp "$BIN_SRC" "$TMP_BIN"
chmod 755 "$TMP_BIN"
mv -f "$TMP_BIN" "$BIN_PATH"
echo "✓ 바이너리 설치 완료: $BIN_PATH"
echo ""

echo "⚙️  4. 설정 파일 설치 중..."
for i in $TARGETS; do
    ENV_NAME="$(instance_env_name "$i")"
    if [ -f "$CONF_DIR/$ENV_NAME" ]; then
        BACKUP_FILE="$CONF_DIR/$ENV_NAME.backup.$(date +%Y%m%d_%H%M%S)"
        cp "$CONF_DIR/$ENV_NAME" "$BACKUP_FILE"
        echo "   기존 ${i}호기 설정 백업: $BACKUP_FILE"
    fi
    cp "config/$ENV_NAME" "$CONF_DIR/"
    chmod 644 "$CONF_DIR/$ENV_NAME"
    echo "✓ ${i}호기 설정 설치: $CONF_DIR/$ENV_NAME"
done
echo ""

echo "🔧 5. systemd 유닛 설치 중..."
for i in $TARGETS; do
    SERVICE="$(instance_service "$i")"
    cp "systemd/$SERVICE" "$UNIT_DIR/"
    chmod 644 "$UNIT_DIR/$SERVICE"
    echo "✓ ${i}호기 유닛 설치: $UNIT_DIR/$SERVICE"
done
echo ""

echo "🔄 6. systemd 데몬 리로드..."
systemctl daemon-reload
echo "✓ 리로드 완료"
echo ""

echo "🚀 7. 서비스 활성화 및 시작..."
for i in $TARGETS; do
    SERVICE="$(instance_service "$i")"
    systemctl enable "$SERVICE"
    # 이미 돌고 있으면 새 바이너리/설정을 반영하기 위해 restart
    systemctl restart "$SERVICE"
    echo "✓ ${i}호기 시작 완료: $SERVICE"
done
echo ""

# 이번에 설치하지 않았는데 돌고 있는 인스턴스는 옛 바이너리를 계속 쓴다
NOT_TARGETED=""
for i in $ALL_INSTANCES; do
    case " $TARGETS " in
        *" $i "*) continue ;;
    esac
    if systemctl is-active --quiet "$(instance_service "$i")"; then
        NOT_TARGETED="$NOT_TARGETED $i"
    fi
done
if [ -n "$NOT_TARGETED" ]; then
    echo "⚠️  아래 인스턴스는 이번 설치 대상이 아니지만 실행 중입니다."
    echo "   바이너리는 공유되므로, 새 바이너리를 반영하려면 재시작이 필요합니다:"
    for i in $NOT_TARGETED; do
        echo "     sudo systemctl restart $(instance_service "$i")"
    done
    echo ""
fi

# 상태 확인
sleep 2
echo "📊 서비스 상태:"
for i in $TARGETS; do
    systemctl status "$(instance_service "$i")" --no-pager | head -5 || true
    echo ""
done

echo "=========================================="
echo "  ✅ 설치 완료!"
echo "=========================================="
echo ""
echo "사용 방법 (인스턴스별 서비스명 사용):"
for i in $TARGETS; do
    SERVICE="$(instance_service "$i")"
    echo "  ${i}호기:"
    echo "    상태 확인:  sudo systemctl status $SERVICE"
    echo "    로그 확인:  sudo journalctl -u $SERVICE -f"
    echo "    재시작:     sudo systemctl restart $SERVICE"
    echo "    설정 파일:  $CONF_DIR/$(instance_env_name "$i")"
done
echo ""
