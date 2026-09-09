# MELSEC PLC Daemon 가이드

PLC 데이터를 자동으로 읽어 Kafka로 전송하는 백그라운드 데몬 서비스입니다.

## 📋 목차

- [특징](#특징)
- [설치](#설치)
- [사용법](#사용법)
- [설정](#설정)
- [관리](#관리)
- [로그 확인](#로그-확인)
- [문제 해결](#문제-해결)

## ✨ 특징

- 🚀 **자동 시작**: 시스템 부팅 시 자동으로 실행
- 🔄 **자동 재연결**: PLC 연결이 끊어지면 자동으로 재연결 시도
- 📊 **Kafka 전송**: 읽은 데이터를 자동으로 Kafka로 전송
- 🛡️ **Graceful Shutdown**: 시그널 수신 시 안전하게 종료
- 📝 **구조화된 로깅**: systemd journal과 통합된 로그 관리
- ⚙️ **환경 설정**: 코드 수정 없이 설정 파일로 동작 변경

## 🔧 설치

### 1. 자동 설치 (권장)

```bash
# 프로젝트 디렉토리에서
sudo ./install-daemon.sh
```

설치 스크립트가 자동으로:
- 데몬 빌드 (release 모드)
- 바이너리 복사 (`/usr/local/bin/melsec-plc-daemon`)
- 설정 파일 복사 (`/etc/melsec-plc/daemon.env`)
- systemd 서비스 등록
- 서비스 시작

### 2. 수동 설치

```bash
# 1. 빌드
cargo build --release --bin melsec-plc-daemon

# 2. 디렉토리 생성
sudo mkdir -p /etc/melsec-plc
sudo mkdir -p /var/lib/melsec-plc
sudo mkdir -p /var/log/melsec-plc

# 3. 바이너리 복사
sudo cp target/release/melsec-plc-daemon /usr/local/bin/
sudo chmod +x /usr/local/bin/melsec-plc-daemon

# 4. 설정 파일 복사
sudo cp config/daemon.env /etc/melsec-plc/

# 5. systemd 서비스 설치
sudo cp systemd/melsec-plc-daemon.service /etc/systemd/system/
sudo systemctl daemon-reload

# 6. 서비스 활성화 및 시작
sudo systemctl enable melsec-plc-daemon
sudo systemctl start melsec-plc-daemon
```

## 📖 사용법

### 기본 명령어

```bash
# 상태 확인
sudo systemctl status melsec-plc-daemon

# 시작
sudo systemctl start melsec-plc-daemon

# 중지
sudo systemctl stop melsec-plc-daemon

# 재시작
sudo systemctl restart melsec-plc-daemon

# 로그 확인 (실시간)
sudo journalctl -u melsec-plc-daemon -f

# 로그 확인 (최근 100줄)
sudo journalctl -u melsec-plc-daemon -n 100
```

### 관리 스크립트 사용

편리한 관리 스크립트를 제공합니다:

```bash
# 상태 확인
./manage-daemon.sh status

# 시작/중지/재시작
sudo ./manage-daemon.sh start
sudo ./manage-daemon.sh stop
sudo ./manage-daemon.sh restart

# 로그 보기
./manage-daemon.sh logs          # 실시간
./manage-daemon.sh logs-recent   # 최근 100줄

# 설정 보기/편집
./manage-daemon.sh config-show   # 현재 설정 보기
sudo ./manage-daemon.sh config 1 # 설정 편집

# 통계 확인
./manage-daemon.sh stats

# Kafka 메시지 확인
./manage-daemon.sh kafka 1
```

## ⚙️ 설정

설정 파일 위치: `/etc/melsec-plc/daemon.env`

```bash
# PLC 연결 설정
PLC_IP=192.168.21.112        # PLC IP 주소
PLC_PORT=5010                # PLC 포트

# Kafka 설정
KAFKA_BROKERS=localhost:9092  # Kafka 브로커 주소
KAFKA_TOPIC=melsec-plc-data-1 # 전송할 토픽 이름 (2호기는 melsec-plc-data-2)

# 읽기 설정
START_ADDRESS=1000            # 시작 주소 (D1000)
READ_COUNT=17                 # 읽을 개수 (D1000~D1016)
READ_INTERVAL_MS=1000         # 읽기 간격 (밀리초)

# 로그 레벨
RUST_LOG=info                 # error, warn, info, debug, trace

# 패킷 트레이스 (기본 꺼짐, 진단 시에만 1)
#PACKET_TRACE=1
```

### 설정 변경

```bash
# 1. 설정 파일 편집
sudo nano /etc/melsec-plc/daemon.env

# 또는 관리 스크립트 사용
sudo ./manage-daemon.sh config 1

# 2. 서비스 재시작 (설정 적용)
sudo systemctl restart melsec-plc-daemon
```

## 🔍 관리

### 부팅 시 자동 시작

```bash
# 활성화 (기본적으로 설치 시 활성화됨)
sudo systemctl enable melsec-plc-daemon

# 비활성화
sudo systemctl disable melsec-plc-daemon

# 상태 확인
systemctl is-enabled melsec-plc-daemon
```

### 제거

```bash
# 자동 제거
sudo ./uninstall-daemon.sh

# 설정 파일 백업은 /tmp에 저장됩니다
```

## 📊 로그 확인

### systemd journal

```bash
# 실시간 로그
sudo journalctl -u melsec-plc-daemon -f

# 최근 로그
sudo journalctl -u melsec-plc-daemon -n 100

# 오늘 로그
sudo journalctl -u melsec-plc-daemon --since today

# 특정 시간 이후 로그
sudo journalctl -u melsec-plc-daemon --since "2026-01-13 14:00:00"

# 에러만 필터링
sudo journalctl -u melsec-plc-daemon -p err

# JSON 형식
sudo journalctl -u melsec-plc-daemon -o json-pretty
```

### 로그 레벨 변경

```bash
# 설정 파일에서 RUST_LOG 변경
sudo nano /etc/melsec-plc/daemon.env

# RUST_LOG=info      # 일반 정보
# RUST_LOG=debug     # 상세 정보
# RUST_LOG=error     # 에러만

# 재시작하여 적용
sudo systemctl restart melsec-plc-daemon
```

## 🔧 문제 해결

### 서비스가 시작되지 않음

```bash
# 상세 상태 확인
sudo systemctl status melsec-plc-daemon -l

# 설정 파일 확인
cat /etc/melsec-plc/daemon.env

# 바이너리 확인
ls -l /usr/local/bin/melsec-plc-daemon

# 수동 실행 테스트
/usr/local/bin/melsec-plc-daemon
```

### PLC 연결 실패

로그에서 "PLC 연결 실패" 또는 "타임아웃" 메시지가 보이는 경우:

```bash
# 1. 네트워크 연결 확인
ping 192.168.21.112

# 2. 포트 확인
nc -zv 192.168.21.112 5010

# 3. 설정 확인
./manage-daemon.sh config-show

# 4. 로그 확인
./manage-daemon.sh logs
```

### Kafka 전송 실패

로그에서 "Kafka 전송 실패" 메시지가 보이는 경우:

```bash
# 1. Kafka 서비스 확인
sudo systemctl status kafka

# 2. Zookeeper 확인
sudo systemctl status zookeeper

# 3. 토픽 확인
kafka-topics.sh --list --bootstrap-server localhost:9092

# 4. Kafka 브로커 주소 확인
./manage-daemon.sh config-show | grep KAFKA
```

### 서비스 크래시

데몬이 계속 재시작되는 경우:

```bash
# 최근 크래시 로그 확인
sudo journalctl -u melsec-plc-daemon -n 200 | grep -E "(error|panic|fatal)"

# 재시작 카운트 확인
sudo systemctl show melsec-plc-daemon | grep NRestarts

# 디버그 로그 활성화
sudo nano /etc/melsec-plc/daemon.env
# RUST_LOG=debug
sudo systemctl restart melsec-plc-daemon
```

### 로그 확인

```bash
# 전체 로그
./manage-daemon.sh logs-recent

# 에러만
sudo journalctl -u melsec-plc-daemon -p err -n 50

# 경고 이상
sudo journalctl -u melsec-plc-daemon -p warning -n 100
```

## 📈 성능 모니터링

### 리소스 사용량

```bash
# CPU/메모리 사용량
systemctl show melsec-plc-daemon --property=MemoryCurrent,CPUUsageNSec

# 상세 통계
./manage-daemon.sh stats

# 프로세스 확인
ps aux | grep melsec-plc-daemon
```

### Kafka 메시지 확인

```bash
# 관리 스크립트 사용
./manage-daemon.sh kafka 1

# 또는 직접 확인
kafkacat -b localhost:9092 -t melsec-plc-data-1 -C
```

## 🗂️ 파일 위치

| 파일 | 위치 | 설명 |
|------|------|------|
| 바이너리 | `/usr/local/bin/melsec-plc-daemon` | 실행 파일 |
| 설정 파일 | `/etc/melsec-plc/daemon.env` | 환경 설정 |
| 서비스 파일 | `/etc/systemd/system/melsec-plc-daemon.service` | systemd 유닛 |
| 작업 디렉토리 | `/var/lib/melsec-plc/` | 데이터 저장 |
| 로그 | `journalctl -u melsec-plc-daemon` | systemd journal |

## 🚀 고급 사용법

### 다중 인스턴스 실행

서로 다른 PLC에서 데이터를 수집하려면:

```bash
# 1. 서비스 파일 복사
sudo cp /etc/systemd/system/melsec-plc-daemon.service \
        /etc/systemd/system/melsec-plc-daemon-2.service

# 2. 설정 파일 복사
sudo cp /etc/melsec-plc/daemon.env \
        /etc/melsec-plc/daemon-2.env

# 3. 서비스 파일 수정
sudo nano /etc/systemd/system/melsec-plc-daemon-2.service
# EnvironmentFile=/etc/melsec-plc/daemon-2.env

# 4. 설정 수정 (다른 IP, 토픽 등)
sudo nano /etc/melsec-plc/daemon-2.env

# 5. 시작
sudo systemctl daemon-reload
sudo systemctl enable melsec-plc-daemon-2
sudo systemctl start melsec-plc-daemon-2
```

### 로그 파일 저장

systemd journal 대신 파일로 로그를 저장하려면:

```bash
# journalctl을 사용하여 로그 내보내기
sudo journalctl -u melsec-plc-daemon \
    --since "2026-01-13" \
    -o short-iso > /var/log/melsec-plc/daemon.log

# 또는 cron으로 주기적 백업
# /etc/cron.daily/melsec-plc-backup
#!/bin/bash
journalctl -u melsec-plc-daemon --since "1 day ago" \
    >> /var/log/melsec-plc/daemon-$(date +%Y%m%d).log
```

## 📚 추가 리소스

- TUI 프로그램: `melsec-plc-tui` - 대화형 터미널 UI
- GUI 프로그램: `melsec-plc` - 그래픽 인터페이스
- Python 프로그램: `python/` 디렉토리 참조
- Kafka 가이드: `TEST_KAFKA_BLINK.md`

## 🆘 도움말

```bash
# 관리 스크립트 도움말
./manage-daemon.sh help

# systemd 매뉴얼
man systemd.service
man systemctl
man journalctl
```

---

**문의사항이나 버그 리포트는 GitHub 이슈로 등록해주세요.**
