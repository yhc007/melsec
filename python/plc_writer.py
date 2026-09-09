#!/usr/bin/env python3
"""
MELSEC PLC D 레지스터 쓰기 프로그램
"""

import pymcprotocol
import sys
import os
from typing import List


class PLCWriter:
    """PLC 데이터 쓰기 클래스"""
    
    def __init__(self, ip: str = "192.168.21.112", port: int = 5010):
        self.ip = ip
        self.port = port
        self.plc = None
        self.connected = False
        
    def connect(self) -> bool:
        """PLC에 연결"""
        try:
            print(f"PLC 연결 시도 중... ({self.ip}:{self.port})")
            
            self.plc = pymcprotocol.Type3E()
            self.plc.network = 0
            self.plc.pc = 0xFF
            self.plc.dest_moduleio = 0x03FF
            self.plc.timer_sec = 3
            
            self.plc.connect(self.ip, self.port)
            self.connected = True
            print("✓ PLC 연결 성공!\n")
            return True
            
        except Exception as e:
            print(f"✗ PLC 연결 실패: {e}\n")
            self.connected = False
            return False
    
    def disconnect(self):
        """PLC 연결 종료"""
        if self.plc and self.connected:
            try:
                self.plc.close()
                self.connected = False
                print("✓ PLC 연결 종료")
            except Exception as e:
                print(f"연결 종료 중 오류: {e}")
    
    def write_word(self, addr: int, value: int) -> bool:
        """
        D 레지스터에 단일 워드 쓰기
        
        Args:
            addr: 주소 (예: 1000)
            value: 쓸 값 (0~65535)
            
        Returns:
            성공 여부
        """
        if not self.connected:
            print("✗ PLC에 연결되어 있지 않습니다.")
            return False
        
        try:
            device = f"D{addr}"
            self.plc.batchwrite_wordunits(device, [value])
            print(f"✓ D{addr}에 {value} (0x{value:04X}) 쓰기 성공")
            return True
            
        except Exception as e:
            print(f"✗ 데이터 쓰기 실패: {e}")
            return False
    
    def write_words(self, start_addr: int, values: List[int]) -> bool:
        """
        D 레지스터에 여러 워드 쓰기
        
        Args:
            start_addr: 시작 주소
            values: 쓸 값들의 리스트
            
        Returns:
            성공 여부
        """
        if not self.connected:
            print("✗ PLC에 연결되어 있지 않습니다.")
            return False
        
        try:
            device = f"D{start_addr}"
            self.plc.batchwrite_wordunits(device, values)
            print(f"✓ D{start_addr}부터 {len(values)}개 워드 쓰기 성공")
            
            # 쓴 값 표시
            for i, value in enumerate(values):
                addr = start_addr + i
                print(f"  D{addr} = {value} (0x{value:04X})")
            
            return True
            
        except Exception as e:
            print(f"✗ 데이터 쓰기 실패: {e}")
            return False


def main():
    """메인 함수"""
    if len(sys.argv) < 3:
        print("사용법:")
        print("  1. 단일 워드 쓰기:")
        print("     python plc_writer.py <주소> <값>")
        print("     예) python plc_writer.py 1000 1234")
        print()
        print("  2. 여러 워드 쓰기:")
        print("     python plc_writer.py <시작주소> <값1> <값2> <값3> ...")
        print("     예) python plc_writer.py 1000 100 200 300 400")
        sys.exit(1)
    
    print("=================================")
    print("  PLC D 레지스터 쓰기 (Python)")
    print("=================================\n")
    
    # 환경 변수에서 설정 읽기
    ip = os.environ.get("PLC_IP", "192.168.21.112")
    port = int(os.environ.get("PLC_PORT", "5010"))
    
    # 인자 파싱
    try:
        start_addr = int(sys.argv[1])
        values = [int(v) for v in sys.argv[2:]]
        
        # 값 범위 검사
        for value in values:
            if not (0 <= value <= 65535):
                print(f"✗ 오류: 값은 0~65535 범위여야 합니다: {value}")
                sys.exit(1)
    
    except ValueError as e:
        print(f"✗ 오류: 잘못된 숫자 형식: {e}")
        sys.exit(1)
    
    # PLC 쓰기 수행
    writer = PLCWriter(ip, port)
    
    if not writer.connect():
        print("\n연결 정보를 확인하세요:")
        print("  export PLC_IP=<PLC_IP_주소>")
        print("  export PLC_PORT=<PLC_포트>")
        sys.exit(1)
    
    try:
        if len(values) == 1:
            writer.write_word(start_addr, values[0])
        else:
            writer.write_words(start_addr, values)
    
    finally:
        writer.disconnect()


if __name__ == "__main__":
    main()
