#!/usr/bin/env python3
"""
Python pymcprotocol이 보내는 패킷 확인
"""

import pymcprotocol
import socket

# 소켓을 모니터링하기 위한 패치
original_sendall = socket.socket.sendall
original_recv = socket.socket.recv

def patched_sendall(self, data):
    hex_str = ' '.join(f'{b:02X}' for b in data)
    print(f"[PYTHON SEND] ({len(data)} bytes): {hex_str}")
    return original_sendall(self, data)

def patched_recv(self, bufsize):
    data = original_recv(self, bufsize)
    hex_str = ' '.join(f'{b:02X}' for b in data)
    print(f"[PYTHON RECV] ({len(data)} bytes): {hex_str}")
    return data

socket.socket.sendall = patched_sendall
socket.socket.recv = patched_recv

# PLC 연결 및 읽기
plc = pymcprotocol.Type3E()
plc.network = 0
plc.pc = 0xFF
plc.dest_moduleio = 0x03FF
plc.timer_sec = 3

try:
    print("PLC 연결 중...")
    plc.connect("192.168.21.112", 5010)
    print("\nD1000 읽기 시도...")
    data = plc.batchread_wordunits("D1000", 10)
    print(f"\n✓ 성공! 데이터: {data[:5]}...")
    plc.close()
except Exception as e:
    print(f"✗ 오류: {e}")
