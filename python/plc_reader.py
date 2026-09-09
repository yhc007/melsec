#!/usr/bin/env python3
"""
MELSEC PLC D1000 주소 읽기 프로그램
Rust 프로그램과 동일한 기능을 Python으로 구현
"""

import pymcprotocol
import sys
import time
import os
from typing import List, Tuple, Optional


class PLCReader:
    """PLC 데이터 읽기 클래스"""
    
    def __init__(self, ip: str = "192.168.21.112", port: int = 5010):
        """
        Args:
            ip: PLC IP 주소
            port: PLC 포트 번호
        """
        self.ip = ip
        self.port = port
        self.plc = None
        self.connected = False
        
    def connect(self) -> bool:
        """PLC에 연결"""
        try:
            print(f"PLC 연결 시도 중...")
            print(f"  IP: {self.ip}")
            print(f"  Port: {self.port}")
            
            # MC Protocol 3E 타입으로 연결
            self.plc = pymcprotocol.Type3E()
            self.plc.network = 0
            self.plc.pc = 0xFF
            self.plc.dest_moduleio = 0x03FF
            self.plc.timer_sec = 3
            
            # 연결
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
    
    def read_words(self, start_addr: int, count: int) -> Optional[List[int]]:
        """
        D 레지스터에서 워드 데이터 읽기
        
        Args:
            start_addr: 시작 주소 (예: 1000)
            count: 읽을 워드 개수
            
        Returns:
            읽은 데이터 리스트 또는 None (실패 시)
        """
        if not self.connected:
            print("✗ PLC에 연결되어 있지 않습니다.")
            return None
        
        try:
            # D 레지스터 읽기
            device = f"D{start_addr}"
            data = self.plc.batchread_wordunits(device, count)
            return data
            
        except Exception as e:
            print(f"✗ 데이터 읽기 실패: {e}")
            return None
    
    def display_data(self, start_addr: int, data: List[int]):
        """데이터를 표 형식으로 출력"""
        print("주소       | 10진수 값  | 16진수 값  | 2진수 값")
        print("───────────┼───────────┼───────────┼──────────────────")
        
        for i, value in enumerate(data):
            addr = start_addr + i
            print(f"D{addr:<8} | {value:<10} | 0x{value:04X}     | {value:016b}")
    
    def continuous_read(self, start_addr: int, count: int, interval: float = 1.0):
        """
        연속으로 데이터 읽기 (Ctrl+C로 종료)
        
        Args:
            start_addr: 시작 주소
            count: 읽을 워드 개수
            interval: 읽기 간격 (초)
        """
        print(f"\nD{start_addr}부터 {count}개 워드 연속 읽기 중... (Ctrl+C로 종료)\n")
        
        try:
            while True:
                data = self.read_words(start_addr, count)
                if data:
                    # 화면 지우기 (Linux/Mac)
                    os.system('clear' if os.name == 'posix' else 'cls')
                    print(f"=== PLC 모니터링 (D{start_addr}~D{start_addr+count-1}) ===")
                    print(f"시간: {time.strftime('%Y-%m-%d %H:%M:%S')}\n")
                    self.display_data(start_addr, data)
                    print(f"\n[{interval}초마다 갱신 중... Ctrl+C로 종료]")
                
                time.sleep(interval)
                
        except KeyboardInterrupt:
            print("\n\n모니터링 종료")


def main():
    """메인 함수"""
    print("=================================")
    print("  PLC D1000 주소 읽기 (Python)")
    print("=================================\n")
    
    # 환경 변수에서 설정 읽기
    ip = os.environ.get("PLC_IP", "192.168.21.112")
    port = int(os.environ.get("PLC_PORT", "5010"))
    
    # PLC 리더 생성 및 연결
    reader = PLCReader(ip, port)
    
    if not reader.connect():
        print("\n연결 정보를 확인하세요:")
        print("  export PLC_IP=<PLC_IP_주소>")
        print("  export PLC_PORT=<PLC_포트>")
        sys.exit(1)
    
    try:
        # D1000부터 10개 워드 읽기
        start_addr = 1000
        count = 10
        
        print(f"D{start_addr}부터 {count}개의 워드 읽기 중...")
        print("─────────────────────────────────")
        
        data = reader.read_words(start_addr, count)
        
        if data:
            print("✓ 데이터 읽기 성공!\n")
            reader.display_data(start_addr, data)
            
            print(f"\n=== 요약 ===")
            print(f"총 {len(data)}개의 워드를 성공적으로 읽었습니다.")
            print(f"주소 범위: D{start_addr} ~ D{start_addr + count - 1}")
            
            # 연속 읽기 옵션
            if len(sys.argv) > 1 and sys.argv[1] == "--continuous":
                reader.continuous_read(start_addr, count)
        else:
            print("데이터를 읽을 수 없습니다.")
            sys.exit(1)
    
    finally:
        reader.disconnect()


if __name__ == "__main__":
    main()
