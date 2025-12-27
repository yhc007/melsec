# TUI 버전 실행 가이드 (X11 불필요)

## TUI 버전 실행

X11/XWindows 없이 터미널에서 바로 실행할 수 있는 TUI (Terminal User Interface) 버전입니다.

### 빌드

```bash
cargo build --bin melsec-plc-tui --release
```

### 실행

```bash
# 방법 1: 빌드된 실행 파일 실행
./target/release/melsec-plc-tui

# 방법 2: cargo로 실행
cargo run --bin melsec-plc-tui --release
```

**중요**: 실제 터미널(SSH 세션 등)에서 실행해야 합니다. `TERM=dumb` 환경에서는 실행되지 않습니다.

### 키보드 단축키

- **C**: 연결
- **D**: 연결 해제
- **R**: 데이터 읽기
- **A**: 자동 읽기 토글
- **F1**: IP 주소 편집
- **F2**: 포트 편집
- **F3**: 디바이스 타입 편집
- **F4**: 시작 주소 편집
- **F5**: 개수 편집
- **Q** 또는 **ESC**: 종료

### 사용 방법

1. **PLC 연결**
   - 기본 IP 주소는 192.168.21.112로 설정되어 있습니다
   - 필요시 F1 키로 IP 주소를 편집할 수 있습니다
   - **C** 키를 눌러 연결합니다

2. **읽기 설정**
   - **F3**: 디바이스 타입 설정 (D, M, X, Y 등)
   - **F4**: 시작 주소 입력
   - **F5**: 읽을 개수 입력

3. **데이터 읽기**
   - **R**: 한 번만 읽기
   - **A**: 자동 읽기 (토글)

4. **데이터 확인**
   - 화면에 표로 표시됩니다
   - 워드 디바이스: 10진수와 16진수 표시
   - 비트 디바이스: ON/OFF 상태 표시

### 장점

- ✅ X11/XWindows 불필요
- ✅ 원격 SSH 세션에서 바로 사용 가능
- ✅ 가볍고 빠름 (1.5MB)
- ✅ 서버 환경에 적합
- ✅ OpenGL 드라이버 불필요

### 주의사항

- 실제 터미널 환경에서만 실행 가능 (SSH 세션 등)
- `TERM=dumb` 환경에서는 실행되지 않습니다
- 터미널 크기는 최소 80x24 이상 권장

### GUI 버전과의 차이

- GUI 버전: `cargo run --release` (또는 `./target/release/melsec-plc`)
- TUI 버전: `cargo run --bin melsec-plc-tui --release` (또는 `./target/release/melsec-plc-tui`)

둘 다 같은 PLC 통신 기능을 제공하지만, TUI 버전은 터미널에서 실행됩니다.
