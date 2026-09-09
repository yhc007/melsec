#!/bin/bash

# 사용법: ./manage-daemon.sh [명령] [1|2|all]
#   인스턴스 인자를 생략하면 all (두 인스턴스 모두)
#   config 와 kafka 는 대상이 하나여야 한다.

CONF_DIR="/etc/melsec-plc"
ALL_INSTANCES="1 2"

# 색상 정의
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

CMD="${1:-help}"
TARGET="${2:-all}"

instance_service() {
    case "$1" in
        1) echo "melsec-plc-daemon" ;;
        2) echo "melsec-plc-daemon-2" ;;
    esac
}

instance_config() {
    case "$1" in
        1) echo "$CONF_DIR/daemon.env" ;;
        2) echo "$CONF_DIR/daemon-2.env" ;;
    esac
}

# 설정 파일에서 Kafka 토픽을 읽는다 (없으면 관례적 기본값)
instance_topic() {
    local cfg topic
    cfg="$(instance_config "$1")"
    if [ -f "$cfg" ]; then
        topic="$(grep -E '^KAFKA_TOPIC=' "$cfg" | tail -1 | cut -d= -f2- | tr -d '"'"'"' ')"
    fi
    echo "${topic:-melsec-plc-data-$1}"
}

# 인스턴스 인자 검증 (help 계열은 인자를 보지 않는다)
case "$CMD" in
    help|--help|-h) ;;
    *)
        case "$TARGET" in
            1|2|all) ;;
            *)
                echo -e "${RED}❌ 알 수 없는 인스턴스: $TARGET${NC}" >&2
                echo "   1, 2 또는 all 중 하나여야 합니다." >&2
                exit 1
                ;;
        esac
        ;;
esac

# 대상 인스턴스 목록 (검증은 위에서 끝났다)
resolve_targets() {
    case "$TARGET" in
        all) echo "$ALL_INSTANCES" ;;
        *)   echo "$TARGET" ;;
    esac
}

# 단일 인스턴스를 요구하는 명령용
# 주의: 호출부가 $(...) 로 stdout을 받으므로 오류는 반드시 stderr로 보낸다.
require_single() {
    if [ "$TARGET" = "all" ]; then
        echo -e "${RED}❌ '$CMD' 명령은 인스턴스를 지정해야 합니다.${NC}" >&2
        echo "   예: $0 $CMD 1" >&2
        return 1
    fi
    echo "$TARGET"
}

# 루트 권한 확인
check_root() {
    if [ "$EUID" -ne 0 ]; then
        echo -e "${RED}❌ 이 명령은 root 권한이 필요합니다.${NC}"
        echo "   sudo $0 $CMD $TARGET"
        exit 1
    fi
}

# 상태 확인
status() {
    for i in $(resolve_targets); do
        echo -e "${BLUE}📊 ${i}호기 서비스 상태:${NC}"
        systemctl status "$(instance_service "$i").service" --no-pager
        echo ""
    done
}

start() {
    check_root
    for i in $(resolve_targets); do
        echo -e "${BLUE}🚀 ${i}호기 서비스 시작 중...${NC}"
        systemctl start "$(instance_service "$i").service"
    done
    sleep 1
    status
}

stop() {
    check_root
    for i in $(resolve_targets); do
        echo -e "${BLUE}⏹️  ${i}호기 서비스 중지 중...${NC}"
        systemctl stop "$(instance_service "$i").service"
        echo -e "${GREEN}✓ 중지 완료: $(instance_service "$i")${NC}"
    done
}

restart() {
    check_root
    for i in $(resolve_targets); do
        echo -e "${BLUE}🔄 ${i}호기 서비스 재시작 중...${NC}"
        systemctl restart "$(instance_service "$i").service"
    done
    sleep 1
    status
}

# 실시간 로그 (여러 인스턴스는 journalctl -u 를 여러 번 준다)
logs() {
    local args=()
    for i in $(resolve_targets); do
        args+=(-u "$(instance_service "$i").service")
    done
    echo -e "${BLUE}📋 로그 (Ctrl+C로 종료): 대상 $TARGET${NC}"
    journalctl "${args[@]}" -f
}

logs_recent() {
    local args=()
    for i in $(resolve_targets); do
        args+=(-u "$(instance_service "$i").service")
    done
    echo -e "${BLUE}📋 최근 로그 (100줄): 대상 $TARGET${NC}"
    journalctl "${args[@]}" -n 100 --no-pager
}

# 설정 편집 (단일 인스턴스만)
config() {
    local i
    i="$(require_single)" || exit 1
    check_root

    local cfg
    cfg="$(instance_config "$i")"
    if [ ! -f "$cfg" ]; then
        echo -e "${RED}❌ 설정 파일이 없습니다: $cfg${NC}"
        exit 1
    fi

    echo -e "${BLUE}⚙️  ${i}호기 설정 파일 편집:${NC}"
    echo "   $cfg"
    echo ""

    local backup
    backup="${cfg}.backup.$(date +%Y%m%d_%H%M%S)"
    cp "$cfg" "$backup"
    echo -e "${GREEN}백업 생성: $backup${NC}"
    echo ""

    ${EDITOR:-nano} "$cfg"

    echo ""
    echo -e "${YELLOW}설정을 적용하려면 서비스를 재시작해야 합니다.${NC}"
    read -p "지금 ${i}호기를 재시작하시겠습니까? (y/N): " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        systemctl restart "$(instance_service "$i").service"
        sleep 1
        systemctl status "$(instance_service "$i").service" --no-pager
    fi
}

config_show() {
    for i in $(resolve_targets); do
        local cfg
        cfg="$(instance_config "$i")"
        echo -e "${BLUE}⚙️  ${i}호기 설정: $cfg${NC}"
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        if [ -f "$cfg" ]; then
            cat "$cfg"
        else
            echo -e "${RED}설정 파일이 없습니다.${NC}"
        fi
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo ""
    done
}

enable_svc() {
    check_root
    for i in $(resolve_targets); do
        echo -e "${BLUE}🔓 ${i}호기 서비스 활성화 중...${NC}"
        systemctl enable "$(instance_service "$i").service"
        echo -e "${GREEN}✓ 부팅 시 자동 시작 활성화: $(instance_service "$i")${NC}"
    done
}

disable_svc() {
    check_root
    for i in $(resolve_targets); do
        echo -e "${BLUE}🔒 ${i}호기 서비스 비활성화 중...${NC}"
        systemctl disable "$(instance_service "$i").service"
        echo -e "${GREEN}✓ 부팅 시 자동 시작 비활성화: $(instance_service "$i")${NC}"
    done
}

# Kafka 메시지 확인 (단일 인스턴스만 - 토픽이 인스턴스마다 다르다)
kafka_consume() {
    local i
    i="$(require_single)" || exit 1

    # 환경변수로 덮어쓸 수 있고, 없으면 해당 인스턴스 설정에서 읽는다
    local topic="${KAFKA_TOPIC:-$(instance_topic "$i")}"

    echo -e "${BLUE}📨 ${i}호기 Kafka 메시지 확인:${NC}"
    echo "   Topic: $topic"
    echo "   (Ctrl+C로 종료)"
    echo ""

    if command -v kafkacat &> /dev/null; then
        kafkacat -b localhost:9092 -t "$topic" -C -f 'Timestamp: %T | Partition: %p | Offset: %o\nKey: %k\nValue: %s\n\n'
    elif [ -f "/usr/local/kafka/bin/kafka-console-consumer.sh" ]; then
        /usr/local/kafka/bin/kafka-console-consumer.sh \
            --bootstrap-server localhost:9092 \
            --topic "$topic" \
            --from-beginning \
            --property print.timestamp=true \
            --property print.key=true
    else
        echo -e "${RED}❌ Kafka consumer 도구를 찾을 수 없습니다.${NC}"
        echo "   kafkacat 또는 kafka-console-consumer.sh를 설치하세요."
        exit 1
    fi
}

stats() {
    for i in $(resolve_targets); do
        local svc
        svc="$(instance_service "$i").service"
        echo -e "${BLUE}📊 ${i}호기 통계 ($svc):${NC}"
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

        if systemctl is-active --quiet "$svc"; then
            echo -e "${GREEN}● 상태: 실행 중${NC}"
        else
            echo -e "${RED}○ 상태: 중지됨${NC}"
        fi

        local uptime mem mem_mb
        uptime="$(systemctl show "$svc" --property=ActiveEnterTimestamp --value)"
        if [ -n "$uptime" ] && [ "$uptime" != "0" ]; then
            echo "  가동 시작: $uptime"
        fi

        mem="$(systemctl show "$svc" --property=MemoryCurrent --value)"
        if [ -n "$mem" ] && [ "$mem" != "[not set]" ] && [ "$mem" != "0" ]; then
            mem_mb=$((mem / 1024 / 1024))
            echo "  메모리: ${mem_mb} MB"
        fi

        echo "  Kafka 토픽: $(instance_topic "$i")"
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo ""
        echo "  최근 로그 통계:"
        journalctl -u "$svc" -n 1000 --no-pager | grep -E "(성공|실패|누적)" | tail -5
        echo ""
    done
}

help() {
    echo "MELSEC PLC Daemon 관리 도구"
    echo ""
    echo "사용법: $0 [명령] [1|2|all]"
    echo ""
    echo "인스턴스:"
    echo "  1              1호기 - melsec-plc-daemon"
    echo "  2              2호기 - melsec-plc-daemon-2"
    echo "  all            두 인스턴스 모두 (기본값)"
    echo ""
    echo "명령:"
    echo "  status         서비스 상태 확인"
    echo "  start          서비스 시작"
    echo "  stop           서비스 중지"
    echo "  restart        서비스 재시작"
    echo "  logs           실시간 로그 보기"
    echo "  logs-recent    최근 로그 보기 (100줄)"
    echo "  config         설정 파일 편집       (인스턴스 지정 필수)"
    echo "  config-show    현재 설정 보기"
    echo "  enable         부팅 시 자동 시작 활성화"
    echo "  disable        부팅 시 자동 시작 비활성화"
    echo "  kafka          Kafka 메시지 확인    (인스턴스 지정 필수)"
    echo "  stats          통계 보기"
    echo "  help           이 도움말 표시"
    echo ""
    echo "예제:"
    echo "  $0 status              # 두 인스턴스 상태"
    echo "  $0 status 2            # 2호기만"
    echo "  $0 logs                # 두 인스턴스 실시간 로그"
    echo "  sudo $0 restart 1      # 1호기 재시작"
    echo "  sudo $0 config 2       # 2호기 설정 편집"
    echo "  $0 kafka 2             # 2호기 토픽 구독"
    echo ""
}

case "$CMD" in
    status)       status ;;
    start)        start ;;
    stop)         stop ;;
    restart)      restart ;;
    logs)         logs ;;
    logs-recent)  logs_recent ;;
    config)       config ;;
    config-show)  config_show ;;
    enable)       enable_svc ;;
    disable)      disable_svc ;;
    kafka)        kafka_consume ;;
    stats)        stats ;;
    help|--help|-h) help ;;
    *)
        echo -e "${RED}❌ 알 수 없는 명령: $CMD${NC}"
        echo ""
        help
        exit 1
        ;;
esac
