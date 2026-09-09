#!/bin/bash

SERVICE_NAME="melsec-plc-daemon"
CONFIG_FILE="/etc/melsec-plc/daemon.env"

# 색상 정의
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 루트 권한 확인
check_root() {
    if [ "$EUID" -ne 0 ]; then 
        echo -e "${RED}❌ 이 명령은 root 권한이 필요합니다.${NC}"
        echo "   sudo $0 $1"
        exit 1
    fi
}

# 상태 확인
status() {
    echo -e "${BLUE}📊 서비스 상태:${NC}"
    systemctl status $SERVICE_NAME.service --no-pager
}

# 시작
start() {
    check_root "start"
    echo -e "${BLUE}🚀 서비스 시작 중...${NC}"
    systemctl start $SERVICE_NAME.service
    sleep 1
    status
}

# 중지
stop() {
    check_root "stop"
    echo -e "${BLUE}⏹️  서비스 중지 중...${NC}"
    systemctl stop $SERVICE_NAME.service
    echo -e "${GREEN}✓ 서비스 중지 완료${NC}"
}

# 재시작
restart() {
    check_root "restart"
    echo -e "${BLUE}🔄 서비스 재시작 중...${NC}"
    systemctl restart $SERVICE_NAME.service
    sleep 1
    status
}

# 로그 보기
logs() {
    echo -e "${BLUE}📋 로그 (Ctrl+C로 종료):${NC}"
    journalctl -u $SERVICE_NAME.service -f
}

# 로그 보기 (최근 100줄)
logs_recent() {
    echo -e "${BLUE}📋 최근 로그 (100줄):${NC}"
    journalctl -u $SERVICE_NAME.service -n 100 --no-pager
}

# 설정 편집
config() {
    check_root "config"
    
    if [ ! -f "$CONFIG_FILE" ]; then
        echo -e "${RED}❌ 설정 파일이 없습니다: $CONFIG_FILE${NC}"
        exit 1
    fi
    
    echo -e "${BLUE}⚙️  설정 파일 편집:${NC}"
    echo "   $CONFIG_FILE"
    echo ""
    
    # 백업 생성
    BACKUP_FILE="${CONFIG_FILE}.backup.$(date +%Y%m%d_%H%M%S)"
    cp "$CONFIG_FILE" "$BACKUP_FILE"
    echo -e "${GREEN}백업 생성: $BACKUP_FILE${NC}"
    echo ""
    
    # 편집기 선택
    EDITOR=${EDITOR:-nano}
    $EDITOR "$CONFIG_FILE"
    
    echo ""
    echo -e "${YELLOW}설정을 적용하려면 서비스를 재시작해야 합니다.${NC}"
    read -p "지금 재시작하시겠습니까? (y/N): " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        restart
    fi
}

# 설정 보기
config_show() {
    if [ ! -f "$CONFIG_FILE" ]; then
        echo -e "${RED}❌ 설정 파일이 없습니다: $CONFIG_FILE${NC}"
        exit 1
    fi
    
    echo -e "${BLUE}⚙️  현재 설정:${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    cat "$CONFIG_FILE"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
}

# 활성화
enable() {
    check_root "enable"
    echo -e "${BLUE}🔓 서비스 활성화 중...${NC}"
    systemctl enable $SERVICE_NAME.service
    echo -e "${GREEN}✓ 부팅 시 자동 시작 활성화${NC}"
}

# 비활성화
disable() {
    check_root "disable"
    echo -e "${BLUE}🔒 서비스 비활성화 중...${NC}"
    systemctl disable $SERVICE_NAME.service
    echo -e "${GREEN}✓ 부팅 시 자동 시작 비활성화${NC}"
}

# Kafka 메시지 확인
kafka_consume() {
    KAFKA_TOPIC=${KAFKA_TOPIC:-melsec-plc-data}
    
    echo -e "${BLUE}📨 Kafka 메시지 확인:${NC}"
    echo "   Topic: $KAFKA_TOPIC"
    echo "   (Ctrl+C로 종료)"
    echo ""
    
    if command -v kafkacat &> /dev/null; then
        kafkacat -b localhost:9092 -t "$KAFKA_TOPIC" -C -f 'Timestamp: %T | Partition: %p | Offset: %o\nKey: %k\nValue: %s\n\n'
    elif [ -f "/usr/local/kafka/bin/kafka-console-consumer.sh" ]; then
        /usr/local/kafka/bin/kafka-console-consumer.sh \
            --bootstrap-server localhost:9092 \
            --topic "$KAFKA_TOPIC" \
            --from-beginning \
            --property print.timestamp=true \
            --property print.key=true
    else
        echo -e "${RED}❌ Kafka consumer 도구를 찾을 수 없습니다.${NC}"
        echo "   kafkacat 또는 kafka-console-consumer.sh를 설치하세요."
        exit 1
    fi
}

# 통계 보기
stats() {
    echo -e "${BLUE}📊 데몬 통계:${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # 서비스 상태
    if systemctl is-active --quiet $SERVICE_NAME.service; then
        echo -e "${GREEN}● 상태: 실행 중${NC}"
    else
        echo -e "${RED}○ 상태: 중지됨${NC}"
    fi
    
    # 가동 시간
    UPTIME=$(systemctl show $SERVICE_NAME.service --property=ActiveEnterTimestamp --value)
    if [ -n "$UPTIME" ] && [ "$UPTIME" != "0" ]; then
        echo "  가동 시작: $UPTIME"
    fi
    
    # 메모리 사용량
    MEM=$(systemctl show $SERVICE_NAME.service --property=MemoryCurrent --value)
    if [ -n "$MEM" ] && [ "$MEM" != "[not set]" ] && [ "$MEM" != "0" ]; then
        MEM_MB=$((MEM / 1024 / 1024))
        echo "  메모리: ${MEM_MB} MB"
    fi
    
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # 최근 로그에서 통계 추출
    echo ""
    echo "최근 로그 통계:"
    journalctl -u $SERVICE_NAME.service -n 1000 --no-pager | \
        grep -E "(성공|실패|누적)" | tail -5
}

# 도움말
help() {
    echo "MELSEC PLC Daemon 관리 도구"
    echo ""
    echo "사용법: $0 [명령]"
    echo ""
    echo "명령:"
    echo "  status         서비스 상태 확인"
    echo "  start          서비스 시작"
    echo "  stop           서비스 중지"
    echo "  restart        서비스 재시작"
    echo "  logs           실시간 로그 보기"
    echo "  logs-recent    최근 로그 보기 (100줄)"
    echo "  config         설정 파일 편집"
    echo "  config-show    현재 설정 보기"
    echo "  enable         부팅 시 자동 시작 활성화"
    echo "  disable        부팅 시 자동 시작 비활성화"
    echo "  kafka          Kafka 메시지 확인"
    echo "  stats          통계 보기"
    echo "  help           이 도움말 표시"
    echo ""
    echo "예제:"
    echo "  $0 status              # 상태 확인"
    echo "  $0 logs                # 실시간 로그"
    echo "  sudo $0 restart        # 재시작"
    echo "  sudo $0 config         # 설정 편집"
    echo ""
}

# 메인
case "${1:-help}" in
    status)
        status
        ;;
    start)
        start
        ;;
    stop)
        stop
        ;;
    restart)
        restart
        ;;
    logs)
        logs
        ;;
    logs-recent)
        logs_recent
        ;;
    config)
        config
        ;;
    config-show)
        config_show
        ;;
    enable)
        enable
        ;;
    disable)
        disable
        ;;
    kafka)
        kafka_consume
        ;;
    stats)
        stats
        ;;
    help|--help|-h)
        help
        ;;
    *)
        echo -e "${RED}❌ 알 수 없는 명령: $1${NC}"
        echo ""
        help
        exit 1
        ;;
esac
