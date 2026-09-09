# Kafka 토픽 깜빡임 기능 테스트

## 변경 사항

TUI 프로그램에서 Kafka가 연결되면 토픽 이름이 **녹색으로 깜빡이고 굵게** 표시됩니다.

## 코드 변경

`src/main_tui.rs`의 Kafka 토픽 표시 부분:

```rust
// Kafka가 연결되었을 때 토픽 이름이 깜빡임
if app.kafka_enabled && app.kafka_producer.is_some() {
    Span::styled(
        &app.kafka_topic,
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::SLOW_BLINK)  // 느린 깜빡임
            .add_modifier(Modifier::BOLD)        // 굵게
    )
} else {
    Span::raw(&app.kafka_topic)
}
```

## 테스트 방법

### 1. TUI 프로그램 실행
```bash
cd /home/root1/Work/RS_PLC
./target/release/melsec-plc-tui
```

### 2. Kafka 연결
- `K` 키를 눌러 Kafka 활성화
- 또는 `b` 키로 Kafka 브로커 설정
- 또는 `t` 키로 Kafka 토픽 설정

### 3. 확인 사항

#### Kafka 연결 전:
```
Kafka: 연결 안됨  브로커: localhost:9092  토픽: melsec-plc-data
                                              ^^^^^^^^^^^^^^^^
                                              (일반 텍스트)
```

#### Kafka 연결 후:
```
Kafka: 연결됨  브로커: localhost:9092  토픽: melsec-plc-data
                                            ^^^^^^^^^^^^^^^^
                                            (녹색, 굵게, 깜빡임)
```

## 기능 설명

### 깜빡임 효과
- **SLOW_BLINK**: 느린 속도로 깜빡임 (약 1초 간격)
- **BOLD**: 굵은 글씨로 강조
- **Green**: 연결 성공을 나타내는 녹색

### 조건
- `app.kafka_enabled == true`: Kafka가 활성화됨
- `app.kafka_producer.is_some()`: Kafka Producer가 생성됨

## 키 바인딩

- `K`: Kafka 연결/해제 토글
- `b`: Kafka 브로커 주소 편집
- `t`: Kafka 토픽 이름 편집
- `c`: PLC 연결
- `r`: 데이터 읽기
- `a`: 자동 읽기 토글
- `q`: 종료

## 환경 변수

```bash
export KAFKA_BROKERS=localhost:9092
export KAFKA_TOPIC=melsec-plc-data
```

## 실행 예제

```bash
# 1. Kafka 확인
ps aux | grep kafka

# 2. 토픽 확인
/usr/local/kafka/bin/kafka-topics.sh --list --bootstrap-server localhost:9092

# 3. TUI 실행
./target/release/melsec-plc-tui

# 4. Kafka 활성화 (K 키)
# 5. PLC 연결 (c 키)
# 6. 자동 읽기 (a 키)
```

## 데이터 흐름

```
PLC (D1000~D1016)
    ↓
TUI 프로그램 (읽기)
    ↓
Kafka Producer
    ↓
Kafka Topic (melsec-plc-data) ← 깜빡임!
    ↓
Kafka Consumer / Grafana
```

## 문제 해결

### 깜빡임이 보이지 않는 경우
1. 터미널이 ANSI 이스케이프 시퀀스를 지원하는지 확인
2. `TERM` 환경 변수 확인: `echo $TERM`
3. 일부 터미널은 깜빡임을 지원하지 않을 수 있음

### Kafka 연결 실패
1. Kafka 서버 실행 확인: `ps aux | grep kafka`
2. 포트 확인: `netstat -tlnp | grep 9092`
3. 토픽 존재 확인: `/usr/local/kafka/bin/kafka-topics.sh --list --bootstrap-server localhost:9092`

## 추가 기능

향후 추가 가능한 기능:
- 데이터 전송 시 깜빡임 속도 변경
- 에러 발생 시 빨간색으로 깜빡임
- 전송된 메시지 카운터 표시
- Kafka 연결 상태에 따른 다른 시각 효과
