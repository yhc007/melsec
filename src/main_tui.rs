use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use melsec_plc::{Device, MelsecClient, BitDevice, WordDevice, KafkaProducer, PlcReadResult};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table, Wrap},
    Frame, Terminal,
};
use std::io;
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::Mutex as TokioMutex;

// 디버그 로그 함수
fn log_debug(msg: &str) {
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("tui_debug.log") {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let _ = writeln!(file, "[{}] {}", timestamp, msg);
    }
}

enum AppMessage {
    Connected(Result<MelsecClient, String>),
    WordData(Vec<(u16, u16)>),
    BitData(Vec<(u16, bool)>),
    Error(String),
}

enum InputMode {
    Normal,
    EditingIp,
    EditingPort,
    EditingNetwork,
    EditingPc,
    EditingDeviceType,
    EditingAddresses,
    EditingKafkaBrokers,
    EditingKafkaTopic,
}

struct App {
    // 연결 설정
    ip_address: String,
    port: u16,
    network: u8,
    pc: u16,
    input_mode: InputMode,
    
    // 연결 상태
    client: Option<Arc<TokioMutex<MelsecClient>>>,
    connected: bool,
    connection_error: String,
    connecting: bool,
    
    // 읽기 설정
    device_type: String,
    device_type_input: String, // 디바이스 타입 편집용 임시 버퍼
    addresses: Vec<String>,
    address_input: String, // 주소 편집용 임시 버퍼
    is_bit_device: bool,
    
    // 데이터
    word_data: Vec<(u16, u16)>,
    bit_data: Vec<(u16, bool)>,
    
    // 읽기 상태
    last_error: String,
    auto_read: bool,
    read_interval_ms: u64,
    last_read_time: Option<std::time::Instant>,
    reading_in_progress: bool, // 읽기 진행 중 플래그
    
    // Kafka 설정
    kafka_producer: Option<KafkaProducer>,
    kafka_brokers: String,
    kafka_topic: String,
    kafka_enabled: bool,
    kafka_error: String,
    
    // 메시지 수신기
    message_rx: Option<mpsc::Receiver<AppMessage>>,
    
    // Tokio 런타임 핸들
    rt_handle: tokio::runtime::Handle,
    
    // UI 상태
    should_quit: bool,
}

impl App {
    fn new(rt_handle: tokio::runtime::Handle) -> Self {
        Self {
            ip_address: "192.168.21.112".to_string(),
            port: 5010,
            network: 0,
            pc: 0xFF, // PC 번호 255로 복원 (브로드캐스트 주소)
            input_mode: InputMode::Normal,
            client: None,
            connected: false,
            connection_error: String::new(),
            connecting: false,
            device_type: "D".to_string(),
            device_type_input: String::new(),
            addresses: vec!["D120".to_string()],
            address_input: String::new(),
            is_bit_device: false, // D는 워드 디바이스
            word_data: Vec::new(),
            bit_data: Vec::new(),
            last_error: String::new(),
            auto_read: false,
            read_interval_ms: 500,
            last_read_time: None,
            reading_in_progress: false,
            kafka_producer: None,
            kafka_brokers: std::env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string()),
            kafka_topic: std::env::var("KAFKA_TOPIC").unwrap_or_else(|_| "melsec-plc-data".to_string()),
            kafka_enabled: false,
            kafka_error: String::new(),
            message_rx: None,
            rt_handle,
            should_quit: false,
        }
    }
    
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
            drop(client);
            self.last_error = "PLC 연결 해제됨".to_string();
        }
        self.connected = false;
        self.client = None;
        self.message_rx = None;
    }
    
    fn read_data(&mut self) {
        log_debug("=== read_data() 호출됨 ===");
        
        if self.reading_in_progress {
            log_debug("이미 읽기 진행 중 - 건너뜀");
            return;
        }
        
        if !self.connected {
            log_debug("오류: PLC에 연결되지 않음");
            self.last_error = "PLC에 연결되지 않았습니다".to_string();
            return;
        }
        log_debug("연결 상태 확인: OK");

        let client = match &self.client {
            Some(c) => Arc::clone(c),
            None => {
                log_debug("오류: client가 None");
                return;
            }
        };
        log_debug("클라이언트 확인: OK");

        let addresses = self.addresses.clone();
        log_debug(&format!("주소 리스트: {:?}", addresses));

        if addresses.is_empty() {
            log_debug("오류: 주소가 비어있음");
            self.last_error = "읽을 주소가 없습니다. E 키로 주소를 설정하세요".to_string();
            return;
        }

        // 읽기 시작 표시
        self.reading_in_progress = true;
        self.last_error = format!("{}개 주소 읽는 중... ({})", addresses.len(), addresses.join(", "));
        log_debug(&format!("읽기 시작: {} 주소, reading_in_progress=true", addresses.len()));

        let (tx, rx) = mpsc::channel(10);
        if let Some(old_rx) = self.message_rx.take() {
            let mut old_rx = old_rx;
            while let Ok(msg) = old_rx.try_recv() {
                self.handle_message(msg);
            }
        }
        self.message_rx = Some(rx);

        let handle = self.rt_handle.clone();
        handle.spawn(async move {
            log_debug("비동기 태스크 시작");
            let mut client_guard = client.lock().await;
            log_debug("클라이언트 락 획득");

            let mut word_data = Vec::new();
            let mut bit_data = Vec::new();

            for addr_str in &addresses {
                log_debug(&format!("주소 파싱 중: {}", addr_str));
                if let Some((device, addr_num)) = Device::from_str(&addr_str) {
                    log_debug(&format!("파싱 성공: device={:?}, addr={}", device, addr_num));
                    match device {
                        Device::Bit(_) => {
                            log_debug(&format!("{} 비트 읽기 시도", addr_str));
                            match client_guard.read_bit(device, addr_num).await {
                                Ok(bit) => {
                                    log_debug(&format!("{} 읽기 성공: {}", addr_str, bit));
                                    bit_data.push((addr_num, bit));
                                }
                                Err(e) => {
                                    log_debug(&format!("{} 읽기 실패: {}", addr_str, e));
                                    let _ = tx.send(AppMessage::Error(format!("{} 읽기 오류: {}", addr_str, e))).await;
                                    return;
                                }
                            }
                        }
                        Device::Word(_) => {
                            log_debug(&format!("{} 워드 읽기 시도", addr_str));
                            match client_guard.read_word(device, addr_num).await {
                                Ok(word) => {
                                    log_debug(&format!("{} 읽기 성공: {}", addr_str, word));
                                    word_data.push((addr_num, word));
                                }
                                Err(e) => {
                                    log_debug(&format!("{} 읽기 실패: {}", addr_str, e));
                                    let _ = tx.send(AppMessage::Error(format!("{} 읽기 오류: {}", addr_str, e))).await;
                                    return;
                                }
                            }
                        }
                    }
                } else {
                    log_debug(&format!("파싱 실패: {}", addr_str));
                    let _ = tx.send(AppMessage::Error(format!("잘못된 주소 형식: {}", addr_str))).await;
                    return;
                }
            }

            log_debug(&format!("워드 데이터 개수: {}, 비트 데이터 개수: {}", word_data.len(), bit_data.len()));
            
            // 결과를 전송
            if !word_data.is_empty() {
                log_debug("워드 데이터 메시지 전송");
                let _ = tx.send(AppMessage::WordData(word_data)).await;
            } else if !bit_data.is_empty() {
                log_debug("비트 데이터 메시지 전송");
                let _ = tx.send(AppMessage::BitData(bit_data)).await;
            } else {
                log_debug("데이터가 없음");
                // 데이터가 없는 경우 (이론상 발생하지 않아야 함)
                let _ = tx.send(AppMessage::Error("읽기 완료했지만 데이터가 없습니다".to_string())).await;
            }
            log_debug("비동기 태스크 종료");
        });

        self.last_read_time = Some(std::time::Instant::now());
    }

    fn connect_kafka(&mut self) {
        if self.kafka_producer.is_some() {
            return; // 이미 연결됨
        }

        match KafkaProducer::new(&self.kafka_brokers, &self.kafka_topic) {
            Ok(producer) => {
                let handle = self.rt_handle.clone();
                let brokers = self.kafka_brokers.clone();
                let topic = self.kafka_topic.clone();
                
                // 비동기 연결 테스트 (에러는 로그로만 출력)
                handle.spawn(async move {
                    let producer_clone = match KafkaProducer::new(&brokers, &topic) {
                        Ok(p) => p,
                        Err(e) => {
                            eprintln!("⚠️  Kafka Producer 생성 실패 ({}): {}", brokers, e);
                            return;
                        }
                    };
                    
                    match producer_clone.test_connection().await {
                        Ok(_) => {
                            match producer_clone.check_topic().await {
                                Ok(true) => {
                                    // 토픽 존재, 연결 성공
                                }
                                Ok(false) => {
                                    eprintln!("⚠️  Kafka 토픽 '{}'이 존재하지 않습니다. Kafka Admin Tool로 토픽을 생성해주세요.", topic);
                                }
                                Err(e) => {
                                    eprintln!("⚠️  Kafka 토픽 확인 실패: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("⚠️  Kafka 연결 실패 ({}): {}", brokers, e);
                        }
                    }
                });
                
                self.kafka_producer = Some(producer);
                self.kafka_error.clear();
            }
            Err(e) => {
                self.kafka_error = format!("Kafka 연결 실패: {}", e);
            }
        }
    }

    fn disconnect_kafka(&mut self) {
        self.kafka_producer = None;
        self.kafka_error.clear();
    }

    fn toggle_kafka(&mut self) {
        if self.kafka_producer.is_some() {
            self.disconnect_kafka();
            self.kafka_enabled = false;
        } else {
            self.connect_kafka();
            self.kafka_enabled = self.kafka_producer.is_some();
        }
    }
    
    fn handle_message(&mut self, msg: AppMessage) {
        match msg {
            AppMessage::Connected(Ok(client)) => {
                log_debug("메시지 처리: 연결 성공");
                self.client = Some(Arc::new(TokioMutex::new(client)));
                self.connected = true;
                self.connecting = false;
                self.connection_error.clear();
                self.last_error = format!("✓ PLC 연결 성공 ({}:{})", self.ip_address, self.port);
            }
            AppMessage::Connected(Err(e)) => {
                log_debug(&format!("메시지 처리: 연결 실패 - {}", e));
                self.connection_error = e.clone();
                self.last_error = format!("✗ {}", e);
                self.connecting = false;
                self.connected = false;
            }
            AppMessage::WordData(data) => {
                log_debug(&format!("메시지 처리: 워드 데이터 수신 ({}개)", data.len()));
                self.reading_in_progress = false;
                self.word_data = data.clone();
                let values_str: Vec<String> = data.iter()
                    .map(|(addr, val)| format!("{}{}={}", self.device_type, addr, val))
                    .collect();
                self.last_error = format!("✓ 읽기 성공: {}", values_str.join(", "));
                log_debug(&format!("상태 메시지 설정: {}, reading_in_progress=false", self.last_error));
                
                // Kafka 전송
                if self.kafka_enabled {
                    if let Some(ref producer) = self.kafka_producer {
                        let producer_clone = match KafkaProducer::new(&self.kafka_brokers, &self.kafka_topic) {
                            Ok(p) => p,
                            Err(e) => {
                                self.kafka_error = format!("Kafka Producer 생성 실패: {}", e);
                                return;
                            }
                        };
                        let addresses = self.addresses.clone();
                        let handle = self.rt_handle.clone();
                        
                        handle.spawn(async move {
                            let result = PlcReadResult::new(data, Vec::new(), &addresses);
                            if let Err(e) = producer_clone.send_plc_data(&result).await {
                                eprintln!("Kafka 전송 실패: {}", e);
                            }
                        });
                    }
                }
            }
            AppMessage::BitData(data) => {
                log_debug(&format!("메시지 처리: 비트 데이터 수신 ({}개)", data.len()));
                self.reading_in_progress = false;
                self.bit_data = data.clone();
                let values_str: Vec<String> = data.iter()
                    .map(|(addr, val)| format!("{}{}={}", self.device_type, addr, if *val { "ON" } else { "OFF" }))
                    .collect();
                self.last_error = format!("✓ 읽기 성공: {}", values_str.join(", "));
                log_debug(&format!("상태 메시지 설정: {}, reading_in_progress=false", self.last_error));
                
                // Kafka 전송
                if self.kafka_enabled {
                    if let Some(ref producer) = self.kafka_producer {
                        let producer_clone = match KafkaProducer::new(&self.kafka_brokers, &self.kafka_topic) {
                            Ok(p) => p,
                            Err(e) => {
                                self.kafka_error = format!("Kafka Producer 생성 실패: {}", e);
                                return;
                            }
                        };
                        let addresses = self.addresses.clone();
                        let word_data = self.word_data.clone();
                        let handle = self.rt_handle.clone();
                        
                        handle.spawn(async move {
                            let result = PlcReadResult::new(word_data, data, &addresses);
                            if let Err(e) = producer_clone.send_plc_data(&result).await {
                                eprintln!("Kafka 전송 실패: {}", e);
                            }
                        });
                    }
                }
            }
            AppMessage::Error(e) => {
                log_debug(&format!("메시지 처리: 오류 수신 - {}", e));
                self.reading_in_progress = false;
                self.last_error = format!("✗ {}", e);
                log_debug("reading_in_progress=false");
            }
        }
    }

    fn parse_addresses(&self, input: &str) -> Vec<(Device, u16)> {
        let mut result = Vec::new();

        // 줄바꿈과 쉼표로 분리
        for line in input.split(&[',', '\n'][..]) {
            let addr = line.trim();
            if !addr.is_empty() {
                if let Some((device, addr_num)) = Device::from_str(addr) {
                    result.push((device, addr_num));
                }
            }
        }

        result
    }

    fn process_messages(&mut self) {
        let mut messages = Vec::new();
        if let Some(rx) = &mut self.message_rx {
            while let Ok(msg) = rx.try_recv() {
                messages.push(msg);
            }
        }
        if !messages.is_empty() {
            log_debug(&format!("process_messages: {}개 메시지 수신", messages.len()));
        }
        for msg in messages {
            self.handle_message(msg);
        }
    }
}

fn ui(f: &mut Frame<CrosstermBackend<io::Stdout>>, app: &App) {
    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(7),
            Constraint::Length(6),
            Constraint::Min(0),
        ])
        .split(f.size());
    
    // 타이틀
    let title = Block::default()
        .borders(Borders::ALL)
        .title("MELSEC PLC 모니터링 (TUI)")
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(title, chunks[0]);
    
    // 연결 설정
    let connection_block = Block::default()
        .borders(Borders::ALL)
        .title("연결 설정");
    
    let pc_str = format!("0x{:02X}", app.pc);
    let port_str = app.port.to_string();
    let network_str = app.network.to_string();
    let mut connection_text = vec![
        Line::from(vec![
            Span::styled("IP: ", Style::default().fg(Color::Yellow)),
            Span::raw(&app.ip_address),
            Span::raw("  "),
            Span::styled("포트: ", Style::default().fg(Color::Yellow)),
            Span::raw(&port_str),
            Span::raw("  "),
            Span::styled("네트워크: ", Style::default().fg(Color::Yellow)),
            Span::raw(&network_str),
            Span::raw("  "),
            Span::styled("PC: ", Style::default().fg(Color::Yellow)),
            Span::raw(&pc_str),
        ]),
        Line::from(vec![
            Span::styled(
                if app.connected {
                    "● 연결됨"
                } else if app.connecting {
                    "○ 연결 중..."
                } else {
                    "○ 연결 안됨"
                },
                Style::default().fg(if app.connected { Color::Green } else { Color::Red }),
            ),
        ]),
    ];
    
    if !app.connection_error.is_empty() {
        connection_text.push(Line::from(vec![
            Span::styled(
                &app.connection_error,
                Style::default().fg(Color::Red),
            ),
        ]));
    }
    
    let connection_paragraph = Paragraph::new(connection_text)
        .block(connection_block)
        .wrap(Wrap { trim: true });
    f.render_widget(connection_paragraph, chunks[1]);
    
    // 읽기 설정
    let read_block = Block::default()
        .borders(Borders::ALL)
        .title("읽기 설정");
    
    let addresses_str = app.addresses.join(", ");
    let auto_read_interval = if app.auto_read {
        format!(" (간격: {}ms)", app.read_interval_ms)
    } else {
        String::new()
    };
    let kafka_status = if app.kafka_enabled && app.kafka_producer.is_some() {
        "연결됨"
    } else {
        "연결 안됨"
    };
    let kafka_error_msg = if !app.kafka_error.is_empty() {
        format!("Kafka 오류: {}", app.kafka_error)
    } else {
        String::new()
    };
    
    let address_display = if matches!(app.input_mode, InputMode::EditingAddresses) {
        format!("{}▊", app.address_input)
    } else {
        addresses_str.clone()
    };

    let device_type_display = if matches!(app.input_mode, InputMode::EditingDeviceType) {
        format!("{}▊", app.device_type_input)
    } else {
        let device_kind = if app.is_bit_device { "(비트)" } else { "(워드)" };
        format!("{} {}", app.device_type, device_kind)
    };

    let mut read_text = vec![
        Line::from(vec![
            Span::styled("디바이스 타입: ", Style::default().fg(if matches!(app.input_mode, InputMode::EditingDeviceType) { Color::Green } else { Color::Yellow })),
            Span::raw(&device_type_display),
        ]),
        Line::from(vec![
            Span::styled("주소 리스트: ", Style::default().fg(if matches!(app.input_mode, InputMode::EditingAddresses) { Color::Green } else { Color::Yellow })),
            Span::raw(&address_display),
        ]),
        Line::from(vec![
            Span::styled("자동읽기: ", Style::default().fg(Color::Yellow)),
            Span::raw(if app.auto_read { "예" } else { "아니오" }),
            Span::raw(&auto_read_interval),
        ]),
        Line::from(vec![
            Span::styled("Kafka: ", Style::default().fg(Color::Yellow)),
            Span::raw(kafka_status),
            Span::raw("  "),
            Span::styled("브로커: ", Style::default().fg(Color::Yellow)),
            Span::raw(&app.kafka_brokers),
            Span::raw("  "),
            Span::styled("토픽: ", Style::default().fg(Color::Yellow)),
            Span::raw(&app.kafka_topic),
        ]),
    ];
    
    if !kafka_error_msg.is_empty() {
        read_text.push(Line::from(vec![
            Span::styled(
                &kafka_error_msg,
                Style::default().fg(Color::Red),
            ),
        ]));
    }
    
    if !app.last_error.is_empty() {
        let color = if app.last_error.contains("✓") || app.last_error.contains("성공") {
            Color::Green
        } else if app.last_error.contains("✗") || app.last_error.contains("오류") || app.last_error.contains("실패") {
            Color::Red
        } else if app.last_error.contains("읽는 중") {
            Color::Cyan
        } else {
            Color::Yellow
        };
        read_text.push(Line::from(vec![
            Span::styled("상태: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                &app.last_error,
                Style::default().fg(color),
            ),
        ]));
    }
    
    let read_paragraph = Paragraph::new(read_text)
        .block(read_block)
        .wrap(Wrap { trim: true });
    f.render_widget(read_paragraph, chunks[2]);
    
    // 데이터 표시
    let data_count = if app.is_bit_device {
        app.bit_data.len()
    } else {
        app.word_data.len()
    };
    let data_title = if data_count == 0 {
        "데이터 (데이터 없음)".to_string()
    } else {
        format!("데이터 ({}개)", data_count)
    };
    
    let data_block = Block::default()
        .borders(Borders::ALL)
        .title(data_title);
    
    if app.is_bit_device {
        let items: Vec<ListItem> = app.bit_data
            .iter()
            .map(|(addr, bit)| {
                let device_name = format!("{}{}", app.device_type, addr);
                let status = if *bit { "ON" } else { "OFF" };
                let color = if *bit { Color::Green } else { Color::Gray };
                ListItem::new(Line::from(vec![
                    Span::styled(device_name.clone(), Style::default().fg(Color::Cyan)),
                    Span::raw(" = "),
                    Span::styled(status, Style::default().fg(color)),
                ]))
            })
            .collect();
        
        let list = List::new(items)
            .block(data_block)
            .style(Style::default().fg(Color::White));
        f.render_widget(list, chunks[3]);
    } else {
        let rows: Vec<Row> = app.word_data
            .iter()
            .map(|(addr, value)| {
                let device_name = format!("{}{}", app.device_type, addr);
                let value_dec = value.to_string();
                let value_hex = format!("0x{:04X}", value);
                Row::new(vec![
                    Cell::from(device_name),
                    Cell::from(value_dec),
                    Cell::from(value_hex),
                ])
            })
            .collect();
        
        let table = Table::new(rows)
            .widths(&[
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Length(10),
            ])
            .header(Row::new(vec![
                Cell::from("주소").style(Style::default().fg(Color::Yellow)),
                Cell::from("값(10진)").style(Style::default().fg(Color::Yellow)),
                Cell::from("값(16진)").style(Style::default().fg(Color::Yellow)),
            ]))
            .block(data_block)
            .style(Style::default().fg(Color::White));
        f.render_widget(table, chunks[3]);
    }
    
    // 도움말
    let help = Paragraph::new(Line::from(vec![
        Span::styled("I:IP ", Style::default().fg(Color::DarkGray)),
        Span::styled("P:포트 ", Style::default().fg(Color::DarkGray)),
        Span::styled("T:디바이스 ", Style::default().fg(Color::DarkGray)),
        Span::styled("E:주소편집 ", Style::default().fg(Color::DarkGray)),
        Span::styled("C:연결 ", Style::default().fg(Color::DarkGray)),
        Span::styled("D:해제 ", Style::default().fg(Color::DarkGray)),
        Span::styled("R:읽기 ", Style::default().fg(Color::DarkGray)),
        Span::styled("A:자동읽기 ", Style::default().fg(Color::DarkGray)),
        Span::styled("K:Kafka ", Style::default().fg(Color::DarkGray)),
        Span::styled("B:브로커 ", Style::default().fg(Color::DarkGray)),
        Span::styled("O:토픽 ", Style::default().fg(Color::DarkGray)),
        Span::styled("Q:종료", Style::default().fg(Color::DarkGray)),
    ]));
    let help_area = Rect {
        x: 0,
        y: f.size().height - 1,
        width: f.size().width,
        height: 1,
    };
    f.render_widget(help, help_area);
}

fn main() -> io::Result<()> {
    // 로그 파일 초기화
    if let Ok(mut file) = std::fs::File::create("tui_debug.log") {
        let _ = writeln!(file, "=== TUI 시작 ===");
    }
    log_debug("메인 함수 시작");
    
    // tokio 런타임 생성 (멀티스레드)
    let rt = tokio::runtime::Runtime::new().unwrap();
    let handle = rt.handle().clone();
    log_debug("Tokio 런타임 생성 완료");
    
    // 백그라운드에서 런타임 실행
    let rt_handle = handle.clone();
    std::thread::spawn(move || {
        rt.block_on(async {
            loop {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });
    });
    
    // 터미널 설정
    let result = (|| -> io::Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        
        let mut app = App::new(rt_handle);
        
        loop {
            // 메시지 처리 (먼저 처리)
            app.process_messages();
            
            // 화면 그리기
            terminal.draw(|f| ui(f, &app))?;
            
            // 자동 읽기
            if app.auto_read && app.connected {
                let should_read = match app.last_read_time {
                    Some(last) => last.elapsed().as_millis() as u64 >= app.read_interval_ms,
                    None => true,
                };
                
                if should_read {
                    app.read_data();
                }
            }
            
            // 이벤트 처리 (폴링 시간 단축)
            if crossterm::event::poll(Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match app.input_mode {
                            InputMode::Normal => {
                                match key.code {
                                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                                        app.should_quit = true;
                                    }
                                    KeyCode::Char('c') | KeyCode::Char('C') => {
                                        log_debug("C 키 입력 감지");
                                        if !app.connected && !app.connecting {
                                            log_debug(&format!("연결 시도: {}:{}", app.ip_address, app.port));
                                            app.last_error = format!("연결 시도 중... ({}:{})", app.ip_address, app.port);
                                            app.connect();
                                        } else if app.connected {
                                            log_debug("이미 연결됨");
                                            app.last_error = "이미 연결되어 있습니다".to_string();
                                        }
                                    }
                                    KeyCode::Char('d') | KeyCode::Char('D') => {
                                        if app.connected {
                                            app.disconnect();
                                        }
                                    }
                                    KeyCode::Char('r') | KeyCode::Char('R') => {
                                        log_debug("R 키 입력 감지");
                                        if app.connected {
                                            log_debug("연결 상태 OK, read_data() 호출");
                                            app.read_data();
                                        } else {
                                            log_debug("연결 안됨");
                                            app.last_error = "✗ PLC에 먼저 연결하세요 (C 키)".to_string();
                                        }
                                    }
                                    KeyCode::Char('k') | KeyCode::Char('K') => {
                                        app.toggle_kafka();
                                    }
                                    KeyCode::Char('a') | KeyCode::Char('A') => {
                                        app.auto_read = !app.auto_read;
                                    }
                                    KeyCode::Char('i') | KeyCode::Char('I') => {
                                        app.input_mode = InputMode::EditingIp;
                                    }
                                    KeyCode::Char('p') | KeyCode::Char('P') => {
                                        app.input_mode = InputMode::EditingPort;
                                    }
                                    KeyCode::Char('t') | KeyCode::Char('T') => {
                                        app.input_mode = InputMode::EditingDeviceType;
                                        app.device_type_input = app.device_type.clone();
                                    }
                                    KeyCode::Char('e') | KeyCode::Char('E') => {
                                        app.input_mode = InputMode::EditingAddresses;
                                        app.address_input = app.addresses.join(", ");
                                    }
                                    KeyCode::Char('b') | KeyCode::Char('B') => {
                                        app.input_mode = InputMode::EditingKafkaBrokers;
                                    }
                                    KeyCode::Char('o') | KeyCode::Char('O') => {
                                        app.input_mode = InputMode::EditingKafkaTopic;
                                    }
                                    _ => {}
                                }
                            }
                            InputMode::EditingIp => {
                                match key.code {
                                KeyCode::Enter => {
                                    app.input_mode = InputMode::Normal;
                                }
                                KeyCode::Esc => {
                                    app.input_mode = InputMode::Normal;
                                }
                                KeyCode::Char(c) => {
                                    app.ip_address.push(c);
                                }
                                KeyCode::Backspace => {
                                    app.ip_address.pop();
                                }
                                _ => {}
                            }
                        }
                        InputMode::EditingPort => {
                            match key.code {
                                KeyCode::Enter => {
                                    if let Ok(num) = app.port.to_string().parse::<u16>() {
                                        app.port = num;
                                    }
                                    app.input_mode = InputMode::Normal;
                                }
                                KeyCode::Esc => {
                                    app.input_mode = InputMode::Normal;
                                }
                                KeyCode::Char(c) if c.is_ascii_digit() => {
                                    let port_str = app.port.to_string();
                                    let new_port_str = format!("{}{}", port_str, c);
                                    if let Ok(num) = new_port_str.parse::<u16>() {
                                        if num <= 65535 {
                                            app.port = num;
                                        }
                                    }
                                }
                                KeyCode::Backspace => {
                                    let port_str = app.port.to_string();
                                    if port_str.len() > 1 {
                                        app.port = port_str[..port_str.len()-1].parse::<u16>().unwrap_or(0);
                                    } else {
                                        app.port = 0;
                                    }
                                }
                                _ => {                                }
                            }
                            }
                            InputMode::EditingDeviceType => {
                            match key.code {
                                KeyCode::Enter => {
                                    // 입력된 디바이스 타입을 적용
                                    if !app.device_type_input.is_empty() {
                                        let device_type = app.device_type_input.to_uppercase();
                                        app.device_type = device_type.clone();
                                        // 디바이스 타입에 따라 비트/워드 구분 설정
                                        app.is_bit_device = matches!(device_type.as_str(),
                                            "X" | "Y" | "M" | "L" | "F" | "V" | "B" | "SB" | "DX" | "DY");
                                    }
                                    app.input_mode = InputMode::Normal;
                                }
                                KeyCode::Esc => {
                                    app.input_mode = InputMode::Normal;
                                }
                                KeyCode::Char(c) if c.is_ascii_alphabetic() => {
                                    app.device_type_input.push(c.to_ascii_uppercase());
                                }
                                KeyCode::Backspace => {
                                    app.device_type_input.pop();
                                }
                                _ => {}
                            }
                            }
                            InputMode::EditingAddresses => {
                                match key.code {
                                    KeyCode::Enter => {
                                        // 입력된 텍스트를 파싱하여 주소 리스트 업데이트
                                        app.addresses = app.address_input
                                            .split(',')
                                            .map(|s| s.trim().to_string())
                                            .filter(|s| !s.is_empty())
                                            .collect();
                                        app.input_mode = InputMode::Normal;
                                    }
                                    KeyCode::Esc => {
                                        app.input_mode = InputMode::Normal;
                                    }
                                    KeyCode::Char(c) => {
                                        app.address_input.push(c);
                                    }
                                    KeyCode::Backspace => {
                                        app.address_input.pop();
                                    }
                                    _ => {}
                                }
                            }
                            InputMode::EditingKafkaBrokers => {
                                match key.code {
                                    KeyCode::Enter => {
                                        app.input_mode = InputMode::Normal;
                                        // 브로커 변경 시 재연결
                                        app.disconnect_kafka();
                                        app.kafka_enabled = false;
                                    }
                                    KeyCode::Esc => {
                                        app.input_mode = InputMode::Normal;
                                    }
                                    KeyCode::Char(c) => {
                                        app.kafka_brokers.push(c);
                                    }
                                    KeyCode::Backspace => {
                                        app.kafka_brokers.pop();
                                    }
                                    _ => {}
                                }
                            }
                            InputMode::EditingKafkaTopic => {
                                match key.code {
                                    KeyCode::Enter => {
                                        app.input_mode = InputMode::Normal;
                                        // 토픽 변경 시 재연결
                                        app.disconnect_kafka();
                                        app.kafka_enabled = false;
                                    }
                                    KeyCode::Esc => {
                                        app.input_mode = InputMode::Normal;
                                    }
                                    KeyCode::Char(c) => {
                                        app.kafka_topic.push(c);
                                    }
                                    KeyCode::Backspace => {
                                        app.kafka_topic.pop();
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            
            if app.should_quit {
                break;
            }
        }
        
        // 정리
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
        
        Ok(())
    })();
    
    // 에러 발생 시에도 터미널 복구 시도
    if result.is_err() {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen, DisableMouseCapture);
    }
    
    result
}

