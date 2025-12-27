use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use melsec_plc::{Device, MelsecClient, BitDevice, WordDevice};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table, Wrap},
    Frame, Terminal,
};
use std::io;
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

enum InputMode {
    Normal,
    EditingIp,
    EditingPort,
    EditingNetwork,
    EditingPc,
    EditingDeviceType,
    EditingStartAddr,
    EditingCount,
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
    
    // UI 상태
    should_quit: bool,
}

impl App {
    fn new(rt_handle: tokio::runtime::Handle) -> Self {
        Self {
            ip_address: "192.168.21.112".to_string(),
            port: 5007,
            network: 0,
            pc: 0xFF,
            input_mode: InputMode::Normal,
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
        if let Some(old_rx) = self.message_rx.take() {
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
    
    let mut read_text = vec![
        Line::from(vec![
            Span::styled("디바이스: ", Style::default().fg(Color::Yellow)),
            Span::raw(&app.device_type),
            Span::raw("  "),
            Span::styled("시작주소: ", Style::default().fg(Color::Yellow)),
            Span::raw(&app.start_address),
            Span::raw("  "),
            Span::styled("개수: ", Style::default().fg(Color::Yellow)),
            Span::raw(&app.count),
            Span::raw("  "),
            Span::styled("비트: ", Style::default().fg(Color::Yellow)),
            Span::raw(if app.is_bit_device { "예" } else { "아니오" }),
        ]),
        Line::from(vec![
            Span::styled("자동읽기: ", Style::default().fg(Color::Yellow)),
            Span::raw(if app.auto_read { "예" } else { "아니오" }),
            if app.auto_read {
                Span::raw(format!(" (간격: {}ms)", app.read_interval_ms))
            } else {
                Span::raw("")
            },
        ]),
    ];
    
    if !app.last_error.is_empty() {
        read_text.push(Line::from(vec![
            Span::styled(
                &app.last_error,
                Style::default().fg(Color::Red),
            ),
        ]));
    }
    
    let read_paragraph = Paragraph::new(read_text)
        .block(read_block)
        .wrap(Wrap { trim: true });
    f.render_widget(read_paragraph, chunks[2]);
    
    // 데이터 표시
    let data_block = Block::default()
        .borders(Borders::ALL)
        .title("데이터");
    
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
        Span::styled("F1:IP ", Style::default().fg(Color::DarkGray)),
        Span::styled("F2:포트 ", Style::default().fg(Color::DarkGray)),
        Span::styled("F3:디바이스 ", Style::default().fg(Color::DarkGray)),
        Span::styled("C:연결 ", Style::default().fg(Color::DarkGray)),
        Span::styled("D:해제 ", Style::default().fg(Color::DarkGray)),
        Span::styled("R:읽기 ", Style::default().fg(Color::DarkGray)),
        Span::styled("A:자동읽기 ", Style::default().fg(Color::DarkGray)),
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
    // tokio 런타임 생성 (멀티스레드)
    let rt = tokio::runtime::Runtime::new().unwrap();
    let handle = rt.handle().clone();
    
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
            terminal.draw(|f| ui(f, &app))?;
            
            // 메시지 처리
            app.process_messages();
            
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
            
            // 이벤트 처리
            if crossterm::event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match app.input_mode {
                            InputMode::Normal => {
                                match key.code {
                                    KeyCode::Char('q') | KeyCode::Esc => {
                                        app.should_quit = true;
                                    }
                                    KeyCode::Char('c') | KeyCode::Char('C') => {
                                        if !app.connected && !app.connecting {
                                            app.connect();
                                        }
                                    }
                                    KeyCode::Char('d') | KeyCode::Char('D') => {
                                        if app.connected {
                                            app.disconnect();
                                        }
                                    }
                                    KeyCode::Char('r') | KeyCode::Char('R') => {
                                        if app.connected {
                                            app.read_data();
                                        }
                                    }
                                    KeyCode::Char('a') | KeyCode::Char('A') => {
                                        app.auto_read = !app.auto_read;
                                    }
                                    KeyCode::F(1) => {
                                        app.input_mode = InputMode::EditingIp;
                                    }
                                    KeyCode::F(2) => {
                                        app.input_mode = InputMode::EditingPort;
                                    }
                                    KeyCode::F(3) => {
                                        app.input_mode = InputMode::EditingDeviceType;
                                    }
                                    KeyCode::F(4) => {
                                        app.input_mode = InputMode::EditingStartAddr;
                                    }
                                    KeyCode::F(5) => {
                                        app.input_mode = InputMode::EditingCount;
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
                                    app.input_mode = InputMode::Normal;
                                }
                                KeyCode::Esc => {
                                    app.input_mode = InputMode::Normal;
                                }
                                KeyCode::Char(c) if c.is_ascii_alphabetic() => {
                                    app.device_type = c.to_uppercase().to_string();
                                    app.input_mode = InputMode::Normal;
                                }
                                _ => {                                }
                            }
                            }
                            InputMode::EditingStartAddr => {
                            match key.code {
                                KeyCode::Enter => {
                                    app.input_mode = InputMode::Normal;
                                }
                                KeyCode::Esc => {
                                    app.input_mode = InputMode::Normal;
                                }
                                KeyCode::Char(c) if c.is_ascii_digit() => {
                                    app.start_address.push(c);
                                }
                                KeyCode::Backspace => {
                                    app.start_address.pop();
                                }
                                _ => {                                }
                            }
                            }
                            InputMode::EditingCount => {
                            match key.code {
                                KeyCode::Enter => {
                                    app.input_mode = InputMode::Normal;
                                }
                                KeyCode::Esc => {
                                    app.input_mode = InputMode::Normal;
                                }
                                KeyCode::Char(c) if c.is_ascii_digit() => {
                                    app.count.push(c);
                                }
                                KeyCode::Backspace => {
                                    app.count.pop();
                                }
                                _ => {                                }
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

