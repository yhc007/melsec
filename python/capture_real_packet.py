#!/usr/bin/env python3
"""
실제 pymcprotocol 패킷 캡처
"""

import pymcprotocol
import socket

# socket.send를 가로채기
_original_send = socket.socket.send
_original_sendall = socket.socket.sendall
_original_recv = socket.socket.recv

def patched_send(self, data):
    hex_str = ' '.join(f'{b:02X}' for b in data)
    print(f"\n[PYTHON SEND] ({len(data)} bytes):")
    print(f"{hex_str}")
    print()
    
    # 바이트별 분석
    if len(data) >= 21:
        print("분석:")
        print(f"  Byte 0-1 (Sub-header):    {data[0]:02X} {data[1]:02X}")
        print(f"  Byte 2 (Network):         {data[2]:02X}")
        print(f"  Byte 3 (PC):              {data[3]:02X}")
        print(f"  Byte 4-5 (Dest IO):       {data[4]:02X} {data[5]:02X}")
        print(f"  Byte 6 (Dest Station):    {data[6]:02X}")
        print(f"  Byte 7-8 (Data Length):   {data[7]:02X} {data[8]:02X} = {data[7] | (data[8] << 8)}")
        print(f"  Byte 9-10 (Timer):        {data[9]:02X} {data[10]:02X}")
        print(f"  Byte 11-12 (Command):     {data[11]:02X} {data[12]:02X}")
        print(f"  Byte 13-14 (Sub-command): {data[13]:02X} {data[14]:02X}")
        print(f"  Byte 15 (Device code):    {data[15]:02X}")
        if len(data) >= 21:
            addr = data[16] | (data[17] << 8) | (data[18] << 16)
            count = data[19] | (data[20] << 8)
            print(f"  Byte 16-18 (Address):     {data[16]:02X} {data[17]:02X} {data[18]:02X} = {addr}")
            print(f"  Byte 19-20 (Count):       {data[19]:02X} {data[20]:02X} = {count}")
        print()
    
    return _original_send(self, data)

def patched_sendall(self, data):
    hex_str = ' '.join(f'{b:02X}' for b in data)
    print(f"\n[PYTHON SENDALL] ({len(data)} bytes):")
    print(f"{hex_str}")
    return _original_sendall(self, data)

def patched_recv(self, bufsize):
    data = _original_recv(self, bufsize)
    if len(data) > 0:
        hex_str = ' '.join(f'{b:02X}' for b in data)
        print(f"\n[PYTHON RECV] ({len(data)} bytes):")
        print(f"{hex_str}")
        
        if len(data) >= 11:
            error_code = data[9] | (data[10] << 8)
            if error_code == 0:
                print(f"  에러 없음 ✓")
            else:
                print(f"  에러 코드: 0x{error_code:04X}")
        print()
    
    return data

socket.socket.send = patched_send
socket.socket.sendall = patched_sendall
socket.socket.recv = patched_recv

# 실제 테스트
print("=" * 50)
print("  pymcprotocol 실제 패킷 캡처")
print("=" * 50)

plc = pymcprotocol.Type3E()
plc.network = 0
plc.pc = 0xFF
plc.dest_moduleio = 0x03FF
plc.timer_sec = 3

try:
    print("\nPLC 연결 중...")
    plc.connect("192.168.21.112", 5010)
    
    print("\nD1000 부터 10개 읽기...")
    data = plc.batchread_wordunits("D1000", 10)
    
    print(f"\n✓ 성공! 데이터: {data}")
    plc.close()
    
except Exception as e:
    print(f"\n✗ 오류: {e}")
    import traceback
    traceback.print_exc()
