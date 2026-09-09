#!/usr/bin/env python3
"""
패킷 캡처 프로그램
"""

import socket
import struct

# MC Protocol 3E 읽기 패킷 수동 생성
def create_read_packet(start_addr, count):
    """D 레지스터 읽기 패킷 생성"""
    packet = bytearray()
    
    # Sub-header (2 bytes)
    packet.extend([0x50, 0x00])
    
    # Network number (1 byte)
    packet.append(0x00)
    
    # PC number (1 byte)  
    packet.append(0xFF)
    
    # Request destination module I/O number (2 bytes, little-endian)
    packet.extend(struct.pack('<H', 0x03FF))
    
    # Request destination module station number (1 byte)
    packet.append(0x00)
    
    # Data length placeholder (2 bytes)
    data_start = len(packet)
    packet.extend([0x00, 0x00])
    
    # CPU monitoring timer (2 bytes)
    packet.extend([0x00, 0x00])
    
    # === Request data ===
    
    # Command code: Batch read (2 bytes, little-endian)
    packet.extend(struct.pack('<H', 0x0401))
    
    # Sub-command (2 bytes, little-endian)
    packet.extend(struct.pack('<H', 0x0000))
    
    # Device code: D = 0xA8 (1 byte)
    packet.append(0xA8)
    
    # Start address (3 bytes, little-endian, 24-bit)
    packet.append(start_addr & 0xFF)
    packet.append((start_addr >> 8) & 0xFF)
    packet.append((start_addr >> 16) & 0xFF)
    
    # Device count (2 bytes, little-endian)
    packet.extend(struct.pack('<H', count))
    
    # Update data length
    data_len = len(packet) - data_start - 2
    packet[data_start] = data_len & 0xFF
    packet[data_start + 1] = (data_len >> 8) & 0xFF
    
    return bytes(packet)

# 패킷 생성 및 출력
print("=== Rust 패킷 (로그에서) ===")
print("50 00 00 FF FF 03 00 0A 00 00 00 01 04 00 00 A8 E8 03 00 0A 00")
print()

print("=== Python 패킷 (수동 생성) ===")
packet = create_read_packet(1000, 10)
hex_str = ' '.join(f'{b:02X}' for b in packet)
print(hex_str)
print()

print(f"패킷 길이: {len(packet)} bytes")
print()

# 바이트별 분석
print("=== 바이트별 분석 ===")
print(f"Byte 0-1 (Sub-header):        {packet[0]:02X} {packet[1]:02X}")
print(f"Byte 2 (Network):             {packet[2]:02X}")
print(f"Byte 3 (PC):                  {packet[3]:02X}")
print(f"Byte 4-5 (Dest IO):           {packet[4]:02X} {packet[5]:02X}")
print(f"Byte 6 (Dest Station):        {packet[6]:02X}")
print(f"Byte 7-8 (Data Length):       {packet[7]:02X} {packet[8]:02X} = {packet[7] | (packet[8] << 8)}")
print(f"Byte 9-10 (Timer):            {packet[9]:02X} {packet[10]:02X}")
print(f"Byte 11-12 (Command):         {packet[11]:02X} {packet[12]:02X}")
print(f"Byte 13-14 (Sub-command):     {packet[13]:02X} {packet[14]:02X}")
print(f"Byte 15 (Device code):        {packet[15]:02X}")
print(f"Byte 16-18 (Address):         {packet[16]:02X} {packet[17]:02X} {packet[18]:02X} = {packet[16] | (packet[17] << 8) | (packet[18] << 16)}")
print(f"Byte 19-20 (Count):           {packet[19]:02X} {packet[20]:02X} = {packet[19] | (packet[20] << 8)}")

# 실제 PLC에 전송 테스트
print()
print("=== 실제 PLC 테스트 ===")
try:
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.settimeout(3)
    sock.connect(("192.168.21.112", 5010))
    
    print("✓ 연결 성공")
    sock.sendall(packet)
    print("✓ 패킷 전송 완료")
    
    # 응답 수신
    response = sock.recv(1024)
    hex_str = ' '.join(f'{b:02X}' for b in response)
    print(f"✓ 응답 수신 ({len(response)} bytes): {hex_str}")
    
    # 에러 코드 확인
    if len(response) >= 11:
        error_code = response[9] | (response[10] << 8)
        if error_code == 0:
            print("✓ 성공! 에러 없음")
            # 데이터 파싱
            data_count = (len(response) - 11) // 2
            print(f"  데이터 개수: {data_count}")
            for i in range(min(data_count, 10)):
                offset = 11 + i * 2
                value = response[offset] | (response[offset + 1] << 8)
                print(f"  D{1000 + i} = {value} (0x{value:04X})")
        else:
            print(f"✗ 에러 코드: 0x{error_code:04X}")
    
    sock.close()
except Exception as e:
    print(f"✗ 오류: {e}")
