#!/usr/bin/env python3
"""
MELSEC PLC GUI 모니터링 프로그램 (Tkinter)
Rust GUI 프로그램과 유사한 기능
"""

import tkinter as tk
from tkinter import ttk, scrolledtext, messagebox
import threading
import time
import pymcprotocol
from typing import Optional


class PLCMonitorGUI:
    """PLC 모니터링 GUI 애플리케이션"""
    
    def __init__(self, root):
        self.root = root
        self.root.title("MELSEC PLC 모니터링 (Python)")
        self.root.geometry("800x600")
        
        # PLC 연결 관련
        self.plc: Optional[pymcprotocol.Type3E] = None
        self.connected = False
        self.auto_read = False
        self.read_thread = None
        
        # GUI 구성
        self.create_widgets()
        
    def create_widgets(self):
        """GUI 위젯 생성"""
        
        # 연결 설정 프레임
        conn_frame = ttk.LabelFrame(self.root, text="연결 설정", padding=10)
        conn_frame.pack(fill="x", padx=10, pady=5)
        
        # IP 주소
        ttk.Label(conn_frame, text="IP 주소:").grid(row=0, column=0, padx=5)
        self.ip_entry = ttk.Entry(conn_frame, width=15)
        self.ip_entry.insert(0, "192.168.21.112")
        self.ip_entry.grid(row=0, column=1, padx=5)
        
        # 포트
        ttk.Label(conn_frame, text="포트:").grid(row=0, column=2, padx=5)
        self.port_entry = ttk.Entry(conn_frame, width=8)
        self.port_entry.insert(0, "5010")
        self.port_entry.grid(row=0, column=3, padx=5)
        
        # 연결 버튼
        self.connect_btn = ttk.Button(conn_frame, text="연결", command=self.toggle_connection)
        self.connect_btn.grid(row=0, column=4, padx=5)
        
        # 연결 상태
        self.status_label = ttk.Label(conn_frame, text="○ 연결 안됨", foreground="red")
        self.status_label.grid(row=0, column=5, padx=10)
        
        # 읽기 설정 프레임
        read_frame = ttk.LabelFrame(self.root, text="읽기 설정", padding=10)
        read_frame.pack(fill="x", padx=10, pady=5)
        
        # 시작 주소
        ttk.Label(read_frame, text="시작 주소:").grid(row=0, column=0, padx=5)
        self.start_addr_entry = ttk.Entry(read_frame, width=10)
        self.start_addr_entry.insert(0, "1000")  # D1000부터 시작
        self.start_addr_entry.grid(row=0, column=1, padx=5)
        
        # 개수
        ttk.Label(read_frame, text="개수:").grid(row=0, column=2, padx=5)
        self.count_entry = ttk.Entry(read_frame, width=8)
        self.count_entry.insert(0, "10")
        self.count_entry.grid(row=0, column=3, padx=5)
        
        # 읽기 버튼
        self.read_btn = ttk.Button(read_frame, text="읽기", command=self.read_once, state="disabled")
        self.read_btn.grid(row=0, column=4, padx=5)
        
        # 자동 읽기 체크박스
        self.auto_read_var = tk.BooleanVar()
        self.auto_read_check = ttk.Checkbutton(
            read_frame, 
            text="자동 읽기", 
            variable=self.auto_read_var,
            command=self.toggle_auto_read
        )
        self.auto_read_check.grid(row=0, column=5, padx=5)
        
        # 간격
        ttk.Label(read_frame, text="간격(초):").grid(row=0, column=6, padx=5)
        self.interval_entry = ttk.Entry(read_frame, width=5)
        self.interval_entry.insert(0, "1")
        self.interval_entry.grid(row=0, column=7, padx=5)
        
        # 데이터 표시 프레임
        data_frame = ttk.LabelFrame(self.root, text="데이터", padding=10)
        data_frame.pack(fill="both", expand=True, padx=10, pady=5)
        
        # 트리뷰 생성
        columns = ("address", "decimal", "hex", "binary")
        self.tree = ttk.Treeview(data_frame, columns=columns, show="headings", height=15)
        
        # 컬럼 설정
        self.tree.heading("address", text="주소")
        self.tree.heading("decimal", text="10진수")
        self.tree.heading("hex", text="16진수")
        self.tree.heading("binary", text="2진수")
        
        self.tree.column("address", width=100)
        self.tree.column("decimal", width=150)
        self.tree.column("hex", width=150)
        self.tree.column("binary", width=200)
        
        # 스크롤바
        scrollbar = ttk.Scrollbar(data_frame, orient="vertical", command=self.tree.yview)
        self.tree.configure(yscrollcommand=scrollbar.set)
        
        self.tree.pack(side="left", fill="both", expand=True)
        scrollbar.pack(side="right", fill="y")
        
        # 로그 프레임
        log_frame = ttk.LabelFrame(self.root, text="로그", padding=5)
        log_frame.pack(fill="x", padx=10, pady=5)
        
        self.log_text = scrolledtext.ScrolledText(log_frame, height=5, state="disabled")
        self.log_text.pack(fill="both", expand=True)
        
    def log(self, message: str):
        """로그 메시지 추가"""
        self.log_text.configure(state="normal")
        timestamp = time.strftime("%H:%M:%S")
        self.log_text.insert("end", f"[{timestamp}] {message}\n")
        self.log_text.see("end")
        self.log_text.configure(state="disabled")
        
    def toggle_connection(self):
        """연결 토글"""
        if self.connected:
            self.disconnect()
        else:
            self.connect()
    
    def connect(self):
        """PLC 연결"""
        try:
            ip = self.ip_entry.get()
            port = int(self.port_entry.get())
            
            self.log(f"PLC 연결 시도: {ip}:{port}")
            
            self.plc = pymcprotocol.Type3E()
            self.plc.network = 0
            self.plc.pc = 0xFF
            self.plc.dest_moduleio = 0x03FF
            self.plc.timer_sec = 3
            
            self.plc.connect(ip, port)
            self.connected = True
            
            self.status_label.config(text="● 연결됨", foreground="green")
            self.connect_btn.config(text="연결 해제")
            self.read_btn.config(state="normal")
            self.log("✓ PLC 연결 성공")
            
        except Exception as e:
            messagebox.showerror("연결 오류", f"PLC 연결 실패:\n{str(e)}")
            self.log(f"✗ 연결 실패: {e}")
            self.connected = False
    
    def disconnect(self):
        """PLC 연결 해제"""
        try:
            if self.auto_read:
                self.toggle_auto_read()
            
            if self.plc:
                self.plc.close()
            
            self.connected = False
            self.status_label.config(text="○ 연결 안됨", foreground="red")
            self.connect_btn.config(text="연결")
            self.read_btn.config(state="disabled")
            self.log("✓ 연결 종료")
            
        except Exception as e:
            self.log(f"연결 종료 중 오류: {e}")
    
    def read_once(self):
        """한 번 읽기"""
        if not self.connected:
            return
        
        try:
            start_addr = int(self.start_addr_entry.get())
            count = int(self.count_entry.get())
            
            device = f"D{start_addr}"
            data = self.plc.batchread_wordunits(device, count)
            
            self.update_tree(start_addr, data)
            self.log(f"✓ D{start_addr}~D{start_addr+count-1} 읽기 성공")
            
        except Exception as e:
            self.log(f"✗ 읽기 오류: {e}")
            messagebox.showerror("읽기 오류", f"데이터 읽기 실패:\n{str(e)}")
    
    def update_tree(self, start_addr: int, data: list):
        """트리뷰 데이터 업데이트"""
        # 기존 데이터 삭제
        for item in self.tree.get_children():
            self.tree.delete(item)
        
        # 새 데이터 추가
        for i, value in enumerate(data):
            addr = start_addr + i
            self.tree.insert("", "end", values=(
                f"D{addr}",
                value,
                f"0x{value:04X}",
                f"{value:016b}"
            ))
    
    def toggle_auto_read(self):
        """자동 읽기 토글"""
        self.auto_read = self.auto_read_var.get()
        
        if self.auto_read:
            self.log("자동 읽기 시작")
            self.read_thread = threading.Thread(target=self.auto_read_loop, daemon=True)
            self.read_thread.start()
        else:
            self.log("자동 읽기 중지")
    
    def auto_read_loop(self):
        """자동 읽기 루프"""
        while self.auto_read and self.connected:
            try:
                self.read_once()
                interval = float(self.interval_entry.get())
                time.sleep(interval)
            except Exception as e:
                self.log(f"자동 읽기 오류: {e}")
                break


def main():
    root = tk.Tk()
    app = PLCMonitorGUI(root)
    root.mainloop()


if __name__ == "__main__":
    main()
