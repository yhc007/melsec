#!/bin/bash
# MELSEC PLC GUI 실행 스크립트

# 기존 Xvfb 프로세스 정리
pkill -f "Xvfb :99" 2>/dev/null

# Xvfb로 가상 디스플레이 생성 (GLX 확장 포함)
Xvfb :99 -screen 0 1024x768x24 -ac +extension GLX +render -noreset 2>/dev/null &
XVFB_PID=$!

# Xvfb 시작 대기
sleep 1

# 디스플레이 설정
export DISPLAY=:99

# 소프트웨어 렌더링 활성화
export LIBGL_ALWAYS_SOFTWARE=1
export GALLIUM_DRIVER=llvmpipe
export MESA_GL_VERSION_OVERRIDE=3.3
export MESA_GLSL_VERSION_OVERRIDE=330

# Xvfb가 실행 중인지 확인
if ! kill -0 $XVFB_PID 2>/dev/null; then
    echo "오류: Xvfb를 시작할 수 없습니다."
    exit 1
fi

# 프로그램 실행
if [ -f "./target/release/melsec-plc" ]; then
    ./target/release/melsec-plc
    EXIT_CODE=$?
elif [ -f "./target/debug/melsec-plc" ]; then
    ./target/debug/melsec-plc
    EXIT_CODE=$?
else
    cargo run --release
    EXIT_CODE=$?
fi

# 프로그램 종료 후 Xvfb 종료
kill $XVFB_PID 2>/dev/null
wait $XVFB_PID 2>/dev/null

exit $EXIT_CODE

