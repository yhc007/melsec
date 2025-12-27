# 문제 해결 가이드

## OpenGL 오류 (GLXBadFBConfig)

현재 시스템에서 OpenGL 컨텍스트를 생성할 수 없는 경우 발생하는 오류입니다.

### 해결 방법 1: Xvfb 스크립트 사용 (권장)

```bash
./run_gui.sh
```

이 스크립트는 가상 디스플레이를 생성하고 소프트웨어 렌더링을 활성화합니다.

### 해결 방법 2: Mesa 소프트웨어 렌더러 설치

```bash
sudo apt-get update
sudo apt-get install -y mesa-utils libgl1-mesa-dri libgl1-mesa-glx
```

### 해결 방법 3: 환경 변수 설정 후 실행

```bash
export DISPLAY=:0
export LIBGL_ALWAYS_SOFTWARE=1
export GALLIUM_DRIVER=llvmpipe
export MESA_GL_VERSION_OVERRIDE=3.3
./target/release/melsec-plc
```

### 해결 방법 4: VNC 서버 사용

GUI 환경이 필요한 경우 VNC 서버를 설치하고 사용할 수 있습니다:

```bash
sudo apt-get install -y tigervnc-standalone-server tigervnc-common
vncserver :1 -geometry 1024x768
export DISPLAY=:1
./target/release/melsec-plc
```

### 해결 방법 5: SSH X11 포워딩 (원격 접속 시)

SSH로 접속하는 경우:

```bash
ssh -X user@hostname
./target/release/melsec-plc
```

## 기타 문제

### 연결 실패
- PLC IP 주소 확인
- 네트워크 연결 확인: `ping <PLC_IP>`
- 방화벽 설정 확인
- PLC에서 MC Protocol 통신 활성화 확인

### 데이터 읽기 실패
- 디바이스 타입이 올바른지 확인 (D, M, X, Y 등)
- 주소 범위가 유효한지 확인
- PLC 연결 상태 확인

