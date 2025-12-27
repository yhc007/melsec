use eframe::egui;
use melsec_plc::{Device, MelsecClient, BitDevice, WordDevice};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::Mutex as TokioMutex;

enum AppMessage {
    Connected(Result<MelsecClient, String>),
    WordData(Vec<(u16, u16)>),
    BitData(Vec<(u16, bool)>),
    Error(String),
}

struct PlcApp {
    // 연결 설정
    ip_address: String,
    port: u16,
    network: u8,
    pc: u16,
    
    // 연결 상태
    client: Option<Arc<TokioMutex<MelsecClient>>>,
    connected: bool,
    connection_error: String,
    connecting: bool,
    
    // 읽기 설정
    device_type: String,
    start_address: String,
    count: String,
    is_bit_device: bool,
    
    // 데이터
    word_data: Vec<(u16, u16)>,
    bit_data: Vec<(u16, bool)>,
    
    // 읽기 상태
    last_error: String,
    auto_read: bool,
    read_interval_ms: u64,
    last_read_time: Option<std::time::Instant>,
    
    // 메시지 수신기
    message_rx: Option<mpsc::Receiver<AppMessage>>,
    
    // Tokio 런타임 핸들
    rt_handle: tokio::runtime::Handle,
}

impl Default for PlcApp {
    fn default() -> Self {
        // 전역 런타임 핸들 사용
        let handle = tokio::runtime::Handle::try_current()
            .unwrap_or_else(|_| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                let handle = rt.handle().clone();
                std::thread::spawn(move || {
                    rt.block_on(async {
                        loop {
                            tokio::time::sleep(Duration::from_secs(1)).await;
                        }
                    });
                });
                handle
            });
        
        Self {
            ip_address: "192.168.21.112".to_string(),
            port: 5007,
            network: 0,
            pc: 0xFF,
            client: None,
            connected: false,
            connection_error: String::new(),
            connecting: false,
            device_type: "D".to_string(),
            start_address: "0".to_string(),
            count: "10".to_string(),
            is_bit_device: false,
            word_data: Vec::new(),
            bit_data: Vec::new(),
            last_error: String::new(),
            auto_read: false,
            read_interval_ms: 500,
            last_read_time: None,
            message_rx: None,
            rt_handle: handle,
        }
    }
}

impl PlcApp {
    fn connect(&mut self) {
        if self.connecting || self.connected {
            return;
        }
        
        self.connecting = true;
        self.connection_error.clear();
        
        let ip = self.ip_address.clone();
        let port = self.port;
        let network = self.network;
        let pc = self.pc as u8;
        let (tx, rx) = mpsc::channel(10);
        self.message_rx = Some(rx);
        
        let handle = self.rt_handle.clone();
        handle.spawn(async move {
            match MelsecClient::connect_str(&ip, port, network, pc).await {
                Ok(mut client) => {
                    client.set_timeout(Duration::from_secs(3));
                    let _ = tx.send(AppMessage::Connected(Ok(client))).await;
                }
                Err(e) => {
                    let _ = tx.send(AppMessage::Connected(Err(format!("연결 오류: {}", e)))).await;
                }
            }
        });
    }
    
    fn disconnect(&mut self) {
        if let Some(client) = self.client.take() {
            // disconnect는 self를 소유권으로 받으므로 Arc에서 빼내야 함
            // 하지만 Arc<Mutex<>>에서 빼낼 수 없으므로, 클라이언트를 드롭하기만 함
            drop(client);
        }
        self.connected = false;
        self.client = None;
        self.message_rx = None;
    }
    
    fn read_data(&mut self) {
        if !self.connected {
            return;
        }
        
        let client = match &self.client {
            Some(c) => Arc::clone(c),
            None => return,
        };
        
        let device_type = self.device_type.clone();
        let start_addr_str = self.start_address.clone();
        let count_str = self.count.clone();
        let is_bit = self.is_bit_device;
        
        let (tx, rx) = mpsc::channel(10);
        // 메시지 수신기 업데이트
        if let Some(old_rx) = self.message_rx.take() {
            // 이전 메시지 처리
            let mut old_rx = old_rx;
            while let Ok(msg) = old_rx.try_recv() {
                self.handle_message(msg);
            }
        }
        self.message_rx = Some(rx);
        
        let handle = self.rt_handle.clone();
        handle.spawn(async move {
            let start_addr = match start_addr_str.parse::<u16>() {
                Ok(addr) => addr,
                Err(e) => {
                    let _ = tx.send(AppMessage::Error(format!("주소 파싱 오류: {}", e))).await;
                    return;
                }
            };
            
            let count = match count_str.parse::<u16>() {
                Ok(c) => c,
                Err(e) => {
                    let _ = tx.send(AppMessage::Error(format!("개수 파싱 오류: {}", e))).await;
                    return;
                }
            };
            
            let mut client_guard = client.lock().await;
            
            if is_bit {
                let device = match device_type.as_str() {
                    "X" => Device::Bit(BitDevice::X),
                    "Y" => Device::Bit(BitDevice::Y),
                    "M" => Device::Bit(BitDevice::M),
                    "L" => Device::Bit(BitDevice::L),
                    "F" => Device::Bit(BitDevice::F),
                    "V" => Device::Bit(BitDevice::V),
                    "B" => Device::Bit(BitDevice::B),
                    "SB" => Device::Bit(BitDevice::SB),
                    _ => {
                        let _ = tx.send(AppMessage::Error("알 수 없는 비트 디바이스".to_string())).await;
                        return;
                    }
                };
                
                match client_guard.read_bits(device, start_addr, count).await {
                    Ok(bits) => {
                        let mut data = Vec::new();
                        for (i, &bit) in bits.iter().enumerate() {
                            data.push((start_addr + i as u16, bit));
                        }
                        let _ = tx.send(AppMessage::BitData(data)).await;
                    }
                    Err(e) => {
                        let _ = tx.send(AppMessage::Error(format!("읽기 오류: {}", e))).await;
                    }
                }
            } else {
                let device = match device_type.as_str() {
                    "D" => Device::Word(WordDevice::D),
                    "W" => Device::Word(WordDevice::W),
                    "SD" => Device::Word(WordDevice::SD),
                    "SW" => Device::Word(WordDevice::SW),
                    "FD" => Device::Word(WordDevice::FD),
                    "R" => Device::Word(WordDevice::R),
                    "ZR" => Device::Word(WordDevice::ZR),
                    _ => {
                        let _ = tx.send(AppMessage::Error("알 수 없는 워드 디바이스".to_string())).await;
                        return;
                    }
                };
                
                match client_guard.read_words(device, start_addr, count).await {
                    Ok(words) => {
                        let mut data = Vec::new();
                        for (i, &word) in words.iter().enumerate() {
                            data.push((start_addr + i as u16, word));
                        }
                        let _ = tx.send(AppMessage::WordData(data)).await;
                    }
                    Err(e) => {
                        let _ = tx.send(AppMessage::Error(format!("읽기 오류: {}", e))).await;
                    }
                }
            }
        });
        
        self.last_read_time = Some(std::time::Instant::now());
    }
    
    fn handle_message(&mut self, msg: AppMessage) {
        match msg {
            AppMessage::Connected(Ok(client)) => {
                self.client = Some(Arc::new(TokioMutex::new(client)));
                self.connected = true;
                self.connecting = false;
                self.connection_error.clear();
            }
            AppMessage::Connected(Err(e)) => {
                self.connection_error = e;
                self.connecting = false;
                self.connected = false;
            }
            AppMessage::WordData(data) => {
                self.word_data = data;
                self.last_error.clear();
            }
            AppMessage::BitData(data) => {
                self.bit_data = data;
                self.last_error.clear();
            }
            AppMessage::Error(e) => {
                self.last_error = e;
            }
        }
    }
    
    fn process_messages(&mut self) {
        let mut messages = Vec::new();
        if let Some(rx) = &mut self.message_rx {
            while let Ok(msg) = rx.try_recv() {
                messages.push(msg);
            }
        }
        for msg in messages {
            self.handle_message(msg);
        }
    }
}

impl eframe::App for PlcApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 메시지 처리
        self.process_messages();
        
        // 자동 읽기
        if self.auto_read && self.connected {
            let should_read = match self.last_read_time {
                Some(last) => last.elapsed().as_millis() as u64 >= self.read_interval_ms,
                None => true,
            };
            
            if should_read {
                self.read_data();
            }
            
            ctx.request_repaint_after(Duration::from_millis(100));
        }
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("MELSEC PLC 모니터링");
            
            ui.separator();
            
            // 연결 설정
            ui.group(|ui| {
                ui.heading("연결 설정");
                ui.horizontal(|ui| {
                    ui.label("IP 주소:");
                    ui.text_edit_singleline(&mut self.ip_address);
                    ui.label("포트:");
                    ui.add(egui::DragValue::new(&mut self.port).clamp_range(1..=65535));
                    ui.label("네트워크:");
                    ui.add(egui::DragValue::new(&mut self.network).clamp_range(0..=255));
                    ui.label("PC:");
                    ui.add(egui::DragValue::new(&mut self.pc).clamp_range(0..=255));
                });
                
                ui.horizontal(|ui| {
                    if self.connected {
                        ui.label(egui::RichText::new("● 연결됨").color(egui::Color32::GREEN));
                        if ui.button("연결 해제").clicked() {
                            self.disconnect();
                        }
                    } else if self.connecting {
                        ui.label(egui::RichText::new("○ 연결 중...").color(egui::Color32::YELLOW));
                    } else {
                        ui.label(egui::RichText::new("○ 연결 안됨").color(egui::Color32::RED));
                        if ui.button("연결").clicked() {
                            self.connect();
                        }
                    }
                });
                
                if !self.connection_error.is_empty() {
                    ui.colored_label(egui::Color32::RED, &self.connection_error);
                }
            });
            
            ui.separator();
            
            // 읽기 설정
            ui.group(|ui| {
                ui.heading("읽기 설정");
                ui.horizontal(|ui| {
                    ui.label("디바이스 타입:");
                    ui.text_edit_singleline(&mut self.device_type);
                    ui.label("시작 주소:");
                    ui.text_edit_singleline(&mut self.start_address);
                    ui.label("개수:");
                    ui.text_edit_singleline(&mut self.count);
                    ui.checkbox(&mut self.is_bit_device, "비트 디바이스");
                });
                
                ui.horizontal(|ui| {
                    if ui.button("읽기").clicked() && self.connected {
                        self.read_data();
                    }
                    ui.checkbox(&mut self.auto_read, "자동 읽기");
                    if self.auto_read {
                        ui.label("간격(ms):");
                        ui.add(egui::DragValue::new(&mut self.read_interval_ms).clamp_range(100..=5000));
                    }
                });
            });
            
            ui.separator();
            
            // 데이터 표시
            ui.group(|ui| {
                ui.heading("데이터");
                
                if !self.last_error.is_empty() {
                    ui.colored_label(egui::Color32::RED, &self.last_error);
                }
                
                let scroll_area = egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .auto_shrink([false; 2]);
                
                scroll_area.show(ui, |ui| {
                    if self.is_bit_device {
                        egui::Grid::new("bit_grid")
                            .num_columns(2)
                            .spacing([40.0, 4.0])
                            .striped(true)
                            .show(ui, |ui| {
                                ui.strong("주소");
                                ui.strong("값");
                                ui.end_row();
                                
                                for (addr, bit) in &self.bit_data {
                                    let device_name = format!("{}{}", self.device_type, addr);
                                    ui.label(device_name);
                                    let color = if *bit { egui::Color32::GREEN } else { egui::Color32::GRAY };
                                    ui.colored_label(color, if *bit { "ON" } else { "OFF" });
                                    ui.end_row();
                                }
                                
                                if self.bit_data.is_empty() {
                                    ui.label("");
                                    ui.label("데이터 없음");
                                    ui.end_row();
                                }
                            });
                    } else {
                        egui::Grid::new("word_grid")
                            .num_columns(2)
                            .spacing([40.0, 4.0])
                            .striped(true)
                            .show(ui, |ui| {
                                ui.strong("주소");
                                ui.strong("값");
                                ui.end_row();
                                
                                for (addr, value) in &self.word_data {
                                    let device_name = format!("{}{}", self.device_type, addr);
                                    ui.label(device_name);
                                    ui.label(format!("{} (0x{:04X})", value, value));
                                    ui.end_row();
                                }
                                
                                if self.word_data.is_empty() {
                                    ui.label("");
                                    ui.label("데이터 없음");
                                    ui.end_row();
                                }
                            });
                    }
                });
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    // 소프트웨어 렌더링 강제 (OpenGL 오류 방지)
    std::env::set_var("LIBGL_ALWAYS_SOFTWARE", "1");
    
    // tokio 런타임 생성
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();
    
    // 백그라운드에서 런타임 실행
    std::thread::spawn(move || {
        rt.block_on(async {
            loop {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });
    });
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 700.0])
            .with_title("MELSEC PLC 모니터링"),
        ..Default::default()
    };
    
    eframe::run_native(
        "MELSEC PLC 모니터링",
        options,
        Box::new(|_cc| Box::new(PlcApp::default())),
    )
}
