# MELSEC PLC 통신 라이브러리 및 모니터링 GUI (Rust)

미쓰비시 전기(Mitsubishi Electric) MELSEC PLC와 통신하기 위한 Rust 라이브러리 및 GUI 모니터링 프로그램입니다.

## 프로그램 실행

### TUI 버전 (터미널 UI - 권장, X11 불필요)

```bash
# 빌드
cargo build --bin melsec-plc-tui --release

# 실행
./target/release/melsec-plc-tui
# 또는
cargo run --bin melsec-plc-tui --release
```

**장점**: X11/XWindows 불필요, 원격 SSH에서 바로 사용 가능

자세한 사용법: [RUN_TUI.md](RUN_TUI.md)

### GUI 버전 (그래픽 UI)

```bash
# 빌드
cargo build --release

# 실행
./target/release/melsec-plc
# 또는
cargo run --release
```

**참고**: X11 디스플레이가 필요합니다. OpenGL 오류 발생 시 `run_gui.sh` 스크립트를 사용하세요.

자세한 사용법: [RUN.md](RUN.md)

### GUI 기능
- PLC 연결 설정 (IP 주소, 포트, 네트워크, PC 번호)
- 디바이스 읽기 설정 (타입: D, M, X, Y 등, 시작 주소, 개수)
- 실시간 데이터 표시 (워드/비트 디바이스)
- 자동 읽기 (주기적 데이터 업데이트)
- 연결 상태 모니터링

## 라이브러리 사용

## 기능

- MC Protocol (MELSEC Communication Protocol) 지원
- TCP/IP 통신
- 비트 디바이스 읽기/쓰기 (X, Y, M, L, F, V, B 등)
- 워드 디바이스 읽기/쓰기 (D, W, SD, SW, FD, R, ZR 등)
- 배치 읽기/쓰기 지원
- 비동기 I/O (tokio 기반)

## 설치

```toml
[dependencies]
melsec-plc = { path = "." }
tokio = { version = "1.0", features = ["full"] }
```

## 사용 예제

### 기본 사용법

```rust
use melsec_plc::{Device, MelsecClient};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // PLC 연결 (IP, 포트, 네트워크 번호, PC 번호)
    let mut client = MelsecClient::connect_str("192.168.1.100", 5007, 0, 0xFF).await?;
    client.set_timeout(Duration::from_secs(3));
    
    // 워드 디바이스 읽기 (D0부터 10개)
    let values = client.read_words(Device::Word(WordDevice::D), 0, 10).await?;
    println!("D0~D9: {:?}", values);
    
    // 단일 워드 읽기
    let value = client.read_word(Device::Word(WordDevice::D), 100).await?;
    println!("D100 = {}", value);
    
    // 비트 디바이스 읽기 (M0부터 16개)
    let bits = client.read_bits(Device::Bit(BitDevice::M), 0, 16).await?;
    println!("M0~M15: {:?}", bits);
    
    // 워드 쓰기
    client.write_word(Device::Word(WordDevice::D), 200, 12345).await?;
    
    // 비트 쓰기
    client.write_bit(Device::Bit(BitDevice::M), 0, true).await?;
    
    // 연결 종료
    client.disconnect().await?;
    
    Ok(())
}
```

### 지원하는 디바이스

#### 비트 디바이스
- X: 입력 릴레이
- Y: 출력 릴레이
- M: 내부 릴레이
- L: 래치 릴레이
- F: 어넌시에이터
- V: 엣지 릴레이
- B: 링크 릴레이
- SB: 링크 특수 릴레이
- DX: 직접 입력 릴레이
- DY: 직접 출력 릴레이

#### 워드 디바이스
- D: 데이터 레지스터
- W: 링크 레지스터
- SD: 링크 특수 레지스터
- SW: 링크 특수 레지스터
- FD: 파일 레지스터
- R: 파일 레지스터
- ZR: 파일 레지스터

## PLC 설정

PLC에서 다음 설정이 필요합니다:

1. Ethernet 모듈 (예: E71) 설치 및 설정
2. MC Protocol 통신 설정 활성화
3. IP 주소 및 포트 번호 설정 (기본 포트: 5007)
4. 네트워크 번호 및 PC 번호 확인

## 주의사항

- 이 라이브러리는 기본적인 MC Protocol 기능만 구현했습니다
- 실제 PLC 모델에 따라 프로토콜 버전이나 디바이스 코드가 다를 수 있습니다
- 프로덕션 환경에서 사용하기 전에 충분히 테스트하세요

## 라이선스

MIT License

