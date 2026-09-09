# 빠른 시작 가이드

## 📦 프로그램 종류

이 프로젝트는 3가지 프로그램을 제공합니다:

| 프로그램 | 설명 | 용도 |
|---------|------|------|
| **melsec-plc** | GUI 프로그램 | 수동 테스트, 개발 |
| **melsec-plc-tui** | TUI 프로그램 | 터미널 모니터링 |
| **melsec-plc-daemon** | 백그라운드 데몬 | 운영 환경, 자동화 |

---

## 🔨 1. 컴파일

### 전체 빌드

```bash
cd /home/root1/Work/RS_PLC
cargo build --release
```

### 개별 빌드

```bash
# GUI
cargo build --release --bin melsec-plc

# TUI
cargo build --release --bin melsec-plc-tui

# Daemon
cargo build --release --bin melsec-plc-daemon
```

---

## 🚀 2. 실행

### GUI 프로그램

```bash
./run_gui.sh

# 또는
DISPLAY=:0 ./target/release/melsec-plc
```

### TUI 프로그램

```bash
./target/release/melsec-plc-tui
```

**조작법**:
- `Tab` / `Shift+Tab`: 필드 이동
- `Enter`: 연결/읽기
- `k`: Kafka 설정
- `a`: 자동 읽기 토글
- `q`: 종료

### Daemon 프로그램

#### 테스트 실행 (10초)

```bash
./run-daemon-test.sh
```

#### 로컬 실행

```bash
PLC_IP=192.168.21.112 \
PLC_PORT=5010 \
KAFKA_BROKERS=localhost:9092 \
KAFKA_TOPIC=melsec-plc-data-1 \
RUST_LOG=info \
./target/release/melsec-plc-daemon
```

종료: `Ctrl+C`

#### systemd 서비스 설치 (권장)

```bash
# 설치
sudo ./install-daemon.sh

# 상태 확인
sudo systemctl status melsec-plc-daemon

# 로그 확인
journalctl -u melsec-plc-daemon -f

# 관리
./manage-daemon.sh help
```

---

## ⚙️ 3. 설정

### PLC 연결 정보

기본값:
- IP: `192.168.21.112`
- Port: `5010`
- 주소: `D1000~D1009`

### Kafka 설정

기본값:
- Brokers: `localhost:9092`
- Topic: 1호기 `melsec-plc-data-1`, 2호기 `melsec-plc-data-2`

토픽은 `KAFKA_TOPIC` 으로 지정한다 (`/etc/melsec-plc/daemon.env`,
`daemon-2.env`). 지정하지 않으면 코드 기본값인 `melsec-plc-data` 로
발행되는데, 이 토픽을 읽는 소비자는 없다.

Kafka 시작:

```bash
# Zookeeper 시작
sudo systemctl start zookeeper

# Kafka 시작
sudo systemctl start kafka

# 토픽 확인
kafka-topics.sh --list --bootstrap-server localhost:9092
```

---

## 🧪 4. 테스트

### PLC 연결 테스트

```bash
# 네트워크 확인
ping 192.168.21.112

# 포트 확인
nc -zv 192.168.21.112 5010
```

### 데이터 읽기 테스트

```bash
# GUI로 테스트
./run_gui.sh

# TUI로 테스트
./target/release/melsec-plc-tui

# Daemon 테스트
./run-daemon-test.sh
```

### Kafka 메시지 확인

```bash
# kafkacat 사용
kafkacat -b localhost:9092 -t melsec-plc-data-1 -C

# 또는 Kafka console consumer
kafka-console-consumer.sh \
    --bootstrap-server localhost:9092 \
    --topic melsec-plc-data-1 \
    --from-beginning

# 또는 관리 스크립트
./manage-daemon.sh kafka 1
```

---

## 📚 5. 자세한 문서

- **GUI 사용법**: `README.md`
- **TUI 사용법**: `RUN_TUI.md`
- **Daemon 사용법**: `DAEMON_GUIDE.md`
- **Python 프로그램**: `python/README.md`
- **Kafka 가이드**: `TEST_KAFKA_BLINK.md`
- **문제 해결**: `TROUBLESHOOTING.md`

---

## 💡 사용 시나리오

### 시나리오 1: 개발/테스트

```bash
# TUI로 실시간 모니터링
./target/release/melsec-plc-tui
```

### 시나리오 2: 데이터 수집 자동화

```bash
# Daemon 설치 및 시작
sudo ./install-daemon.sh

# 로그 모니터링
journalctl -u melsec-plc-daemon -f
```

### 시나리오 3: Python으로 데이터 분석

```bash
# Daemon이 Kafka로 데이터 전송
# Python에서 Kafka Consumer로 데이터 읽기

cd python
source venv/bin/activate
python plc_gui.py
```

---

## 🆘 문제 해결

### 컴파일 에러

```bash
# 의존성 업데이트
cargo clean
cargo update
cargo build --release
```

### PLC 연결 실패

```bash
# 네트워크 확인
ping 192.168.21.112
nc -zv 192.168.21.112 5010

# 방화벽 확인
sudo iptables -L
```

### Kafka 연결 실패

```bash
# 서비스 상태 확인
sudo systemctl status zookeeper
sudo systemctl status kafka

# 재시작
sudo systemctl restart zookeeper
sudo systemctl restart kafka
```

### 권한 문제

```bash
# 실행 권한 부여
chmod +x run_gui.sh
chmod +x run-daemon-test.sh
chmod +x manage-daemon.sh
```

---

## 📞 더 많은 도움이 필요하신가요?

각 프로그램의 자세한 문서를 참조하세요:

- GUI: `README.md`
- TUI: `RUN_TUI.md`
- **Daemon: `DAEMON_GUIDE.md` ⭐️**
- Python: `python/README.md`
