# Python PLC 통신 프로그램

MELSEC PLC와 통신하는 Python 프로그램입니다. Rust 프로그램과 동일한 기능을 제공하며, D1000번 주소부터 데이터를 읽어옵니다.

## 설치

```bash
cd python
./setup.sh
```

또는 수동 설치:

```bash
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
```

## 프로그램 목록

### 1. plc_reader.py - 데이터 읽기 프로그램

D1000부터 데이터를 읽어서 표 형식으로 출력합니다.

**기본 사용:**
```bash
python plc_reader.py
```

**연속 읽기 모드:**
```bash
python plc_reader.py --continuous
```

**출력 예시:**
```
=================================
  PLC D1000 주소 읽기 (Python)
=================================

PLC 연결 시도 중...
  IP: 192.168.21.112
  Port: 5010

✓ PLC 연결 성공!

D1000부터 10개의 워드 읽기 중...
─────────────────────────────────
✓ 데이터 읽기 성공!

주소       | 10진수 값  | 16진수 값  | 2진수 값
───────────┼───────────┼───────────┼──────────────────
D1000      | 1234       | 0x04D2     | 0000010011010010
D1001      | 5678       | 0x162E     | 0001011000101110
...
```

### 2. plc_writer.py - 데이터 쓰기 프로그램

D 레지스터에 데이터를 씁니다.

**단일 워드 쓰기:**
```bash
python plc_writer.py 1000 1234
```

**여러 워드 쓰기:**
```bash
python plc_writer.py 1000 100 200 300 400
```

D1000=100, D1001=200, D1002=300, D1003=400이 설정됩니다.

### 3. plc_gui.py - GUI 모니터링 프로그램

Tkinter 기반 GUI 프로그램입니다. Rust GUI 프로그램과 유사한 기능을 제공합니다.

```bash
python plc_gui.py
```

**기능:**
- PLC 연결/해제
- D 레지스터 읽기
- 자동 갱신 모드
- 실시간 데이터 표시
- 로그 기록

## 환경 설정

### IP 주소와 포트 변경

```bash
export PLC_IP=192.168.1.100
export PLC_PORT=6000
python plc_reader.py
```

### 기본값
- IP: 192.168.21.112
- Port: 5010
- 시작 주소: D1000
- 읽기 개수: 10 워드

## Rust 프로그램과 비교

| 기능 | Rust | Python |
|------|------|--------|
| CLI 읽기 | ✓ | ✓ |
| GUI | ✓ (egui) | ✓ (tkinter) |
| TUI | ✓ | - |
| 쓰기 | - | ✓ |
| 연속 읽기 | ✓ | ✓ |
| 성능 | 매우 빠름 | 빠름 |
| 설치 | cargo | pip |

## 예제 사용 시나리오

### 1. 빠른 테스트
```bash
# D1000 읽기
python plc_reader.py
```

### 2. 실시간 모니터링
```bash
# 1초마다 D1000~D1009 모니터링
python plc_reader.py --continuous
```

### 3. 데이터 쓰기 및 확인
```bash
# D1000에 값 쓰기
python plc_writer.py 1000 9999

# 확인
python plc_reader.py
```

### 4. GUI로 모니터링
```bash
python plc_gui.py
```

## 문제 해결

### pymcprotocol 설치 오류
```bash
pip install --upgrade pip
pip install pymcprotocol
```

### tkinter 없음 오류 (Linux)
```bash
sudo apt install python3-tk
```

### 연결 타임아웃
- PLC IP 주소 확인
- 네트워크 연결 확인
- 방화벽 설정 확인
- PLC가 실행 중인지 확인

## 라이브러리

- **pymcprotocol**: MELSEC PLC MC Protocol 통신
- **tkinter**: GUI 프레임워크 (Python 기본 포함)
- **threading**: 비동기 처리

## 라이선스

이 프로젝트와 동일한 라이선스를 따릅니다.
