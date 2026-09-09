# Python PLC 프로그램 사용 가이드

## 📦 설치

```bash
cd python
./setup.sh
```

설치가 완료되면 가상환경이 생성되고 필요한 패키지가 설치됩니다.

## 🚀 빠른 시작

### 1. 가상환경 활성화

```bash
cd python
source venv/bin/activate
```

### 2. D1000 데이터 읽기

```bash
python plc_reader.py
```

**출력:**
```
=================================
  PLC D1000 주소 읽기 (Python)
=================================

✓ PLC 연결 성공!
✓ 데이터 읽기 성공!

주소       | 10진수 값  | 16진수 값  | 2진수 값
───────────┼───────────┼───────────┼──────────────────
D1000      | 392        | 0x0188     | 0000000110001000
D1001      | 0          | 0x0000     | 0000000000000000
...
```

## 📋 사용 가능한 프로그램

### 1. plc_reader.py - 데이터 읽기

**기본 읽기:**
```bash
python plc_reader.py
```

**연속 모니터링 (1초 간격):**
```bash
python plc_reader.py --continuous
```

### 2. plc_writer.py - 데이터 쓰기

**단일 워드 쓰기:**
```bash
python plc_writer.py 1000 1234
# D1000에 1234 저장
```

**여러 워드 쓰기:**
```bash
python plc_writer.py 1000 100 200 300 400 500
# D1000=100, D1001=200, D1002=300, D1003=400, D1004=500
```

### 3. plc_gui.py - GUI 모니터링

```bash
python plc_gui.py
```

**기능:**
- 그래픽 인터페이스로 PLC 모니터링
- 실시간 데이터 표시
- 자동 갱신 기능
- 시작 주소/개수 변경 가능

## ⚙️ 환경 설정

### IP 주소 변경

```bash
export PLC_IP=192.168.1.100
export PLC_PORT=6000
python plc_reader.py
```

### 기본 설정

- **IP**: 192.168.21.112
- **Port**: 5010
- **시작 주소**: D1000
- **읽기 개수**: 10

## 🔄 Rust vs Python 비교

| 특징 | Rust 프로그램 | Python 프로그램 |
|------|---------------|-----------------|
| **실행 속도** | 매우 빠름 | 빠름 |
| **설치** | cargo build | pip install |
| **GUI** | egui (네이티브) | tkinter |
| **TUI** | ✓ (터미널 UI) | ✗ |
| **쓰기 기능** | ✗ | ✓ |
| **CLI** | ✓ | ✓ |
| **시작 주소** | D1000 | D1000 |
| **크로스 플랫폼** | ✓ | ✓ |

## 💡 사용 예제

### 예제 1: 데이터 확인

```bash
# D1000~D1009 읽기
python plc_reader.py
```

### 예제 2: 데이터 쓰고 확인

```bash
# D1000에 9999 쓰기
python plc_writer.py 1000 9999

# 확인
python plc_reader.py
```

### 예제 3: 여러 주소에 연속 값 쓰기

```bash
# D1000~D1004에 1,2,3,4,5 쓰기
python plc_writer.py 1000 1 2 3 4 5

# 확인
python plc_reader.py
```

### 예제 4: 실시간 모니터링

```bash
# 1초마다 갱신 (Ctrl+C로 종료)
python plc_reader.py --continuous
```

### 예제 5: GUI로 모니터링

```bash
python plc_gui.py
```

1. IP/Port 입력 후 "연결" 클릭
2. 시작 주소 "1000" 확인
3. "읽기" 클릭
4. "자동 읽기" 체크하면 실시간 갱신

## 🐛 문제 해결

### 연결 실패

```
✗ PLC 연결 실패: [Errno 111] Connection refused
```

**해결 방법:**
- PLC IP 주소 확인
- PLC가 켜져 있는지 확인
- 네트워크 연결 확인
- 방화벽 설정 확인

### 타임아웃

```
✗ 데이터 읽기 실패: timeout
```

**해결 방법:**
- PLC 응답 시간 확인
- 네트워크 지연 확인
- 주소 범위가 올바른지 확인

### tkinter 오류 (Linux)

```
ModuleNotFoundError: No module named '_tkinter'
```

**해결 방법:**
```bash
sudo apt install python3-tk
```

## 📊 데이터 형식

Python 프로그램은 다음 형식으로 데이터를 표시합니다:

```
주소       | 10진수 값  | 16진수 값  | 2진수 값
───────────┼───────────┼───────────┼──────────────────
D1000      | 1234       | 0x04D2     | 0000010011010010
```

- **10진수**: 일반적인 숫자 (0~65535)
- **16진수**: 0x0000~0xFFFF
- **2진수**: 16비트 표현

## 🔧 개발자 정보

### 사용 라이브러리

- **pymcprotocol 0.3.0**: MELSEC MC Protocol 통신
- **tkinter**: GUI (Python 기본 포함)
- **threading**: 비동기 처리

### 코드 구조

```
python/
├── plc_reader.py      # 읽기 프로그램
├── plc_writer.py      # 쓰기 프로그램
├── plc_gui.py         # GUI 프로그램
├── requirements.txt   # 패키지 목록
├── setup.sh          # 설치 스크립트
└── README.md         # 문서
```

## 🎯 추천 워크플로우

### 일반 사용자

1. `python plc_gui.py` 실행
2. GUI에서 모니터링

### 개발자/테스트

1. `python plc_reader.py` - 빠른 확인
2. `python plc_writer.py 1000 값` - 값 설정
3. `python plc_reader.py --continuous` - 실시간 모니터링

### 자동화/스크립트

```python
from plc_reader import PLCReader

reader = PLCReader("192.168.21.112", 5010)
if reader.connect():
    data = reader.read_words(1000, 10)
    print(f"D1000 값: {data[0]}")
    reader.disconnect()
```

## 🚀 다음 단계

- Kafka 연동 (Rust 프로그램과 동일)
- 데이터 로깅 기능
- 그래프 시각화
- 알람/알림 기능
- 배치 작업 스크립트

---

더 자세한 정보는 `python/README.md`를 참조하세요.
