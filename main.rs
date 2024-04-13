use std::{collections::HashSet, net::{SocketAddr, IpAddr, Ipv4Addr, Ipv6Addr, UdpSocket}};
use std::thread;
use std::sync::Arc;
use std::time;
use std::error::Error;
use std::io;
use std::sync::mpsc;
use thread::sleep;
use std::sync::atomic::{AtomicBool, Ordering};
use time::Duration;
use tui::layout::{Layout, Direction, Constraint};
use tui::text::Text;
use tui::widgets::{Block, Borders, List, ListItem, Paragraph };

use socket2::{Socket, Domain, Type, Protocol, SockAddr};

use crossterm::{
    execute,
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    event::{Event as CEvent, KeyEvent, KeyCode}
};

use unicode_width::UnicodeWidthStr;

enum Event {
    Input(KeyEvent),
    MessageReceived(String, String),
}

// todo implement async/await for the socket below. Learn more about async/await in rust here:
// https://rust-lang.github.io/async-book/02_execution/05_io.html

// todo use rust's Foreign Function Interface (FFI) to handle ctrl+c:
// https://docs.microsoft.com/en-us/windows/console/registering-a-control-handler-function

// consider using argh for cmdline arg parsing to struct

struct ChatModel {
    users: HashSet<String>,
    messages: Vec<String>,
    user_input: String
}

impl Default for ChatModel {
    fn default() -> Self {
        let mut model = ChatModel {
            users: HashSet::new(),
            messages: Vec::new(),
            user_input: String::new()
        };

        model.users.insert(String::from("You"));
        model.messages.push(String::from("Welcome to the chat!"));

        model
    }
}

fn main() -> Result<(), Box<dyn Error>> {

    let loopback = false;
    let multicast_port = 35767;
    let multicast_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(239,255, 101, 33)), multicast_port);
    //let multicast_addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0xFF02, 0, 0, 0, 0, 0, 0, 0x0123)), multicast_port);

    let domain = match multicast_addr.ip() {
        IpAddr::V4(_) => Domain::IPV4,
        IpAddr::V6(_) => Domain::IPV6
    };

    let interface = Ipv4Addr::new(10,100,0,3);

    assert!(multicast_addr.ip().is_multicast());

    let closing = Arc::from(AtomicBool::new(false));

    // input handling
    let (tx, rx) = mpsc::channel();

    let tx_clone = tx.clone();
    let listener_closing = Arc::clone(&closing);
    let listener = thread::spawn(move || {

        let socket = Socket::new(domain, Type::DGRAM, Some(Protocol::UDP)).expect("Could not make socket");
        socket.set_read_timeout(Some(Duration::from_millis(10))).expect("Could not set read timeout");
        match multicast_addr.ip() {
            IpAddr::V4(ref ipv4) => {
                socket.set_multicast_loop_v4(loopback).expect("Could not set multicast loopback");
                socket.join_multicast_v4(ipv4, &interface).expect("Could not join multicast v4");
            },
            IpAddr::V6(ref ipv6) => { 
                socket.set_multicast_loop_v6(loopback).expect("Could not set multicast loopback");
                socket.join_multicast_v6(ipv6, 0).expect("Could not join multicast v6");
                socket.set_only_v6(true).expect("Could not set only v6");
            }
        };

        // on windows don't bind to multicast address
        let addr = match multicast_addr {
            SocketAddr::V4(addr) => SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), addr.port()),
            SocketAddr::V6(addr) => {
                SocketAddr::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0).into(), addr.port())
            }
        };
        socket.bind(&socket2::SockAddr::from(addr)).expect("Could not bind listener socket");
        
        // on unix this would be safe to do
        //socket.bind(&SockAddr::from(multicast_addr)).expect("Could not bind listener");

        let socket: UdpSocket = socket.into();

        let mut buf = [0; 1024];

        while !listener_closing.load(Ordering::Relaxed) {
            match socket.recv_from(&mut buf) {
                Ok((_, from)) => {
                    let message = String::from_utf8(buf.to_vec()).expect("Could not convert message from utf8");
                    tx_clone.send(Event::MessageReceived(from.to_string(), message)).expect("Could not send tx in listener");
                },
                _ => sleep(time::Duration::from_millis(10))
            };
        }
    });

    let socket = Socket::new(domain, Type::DGRAM, Some(Protocol::UDP))?;

    match multicast_addr.ip() {
        IpAddr::V4(_) => {
            socket.set_multicast_if_v4(&interface)?;
            socket.bind(
                &SockAddr::from(SocketAddr::new(interface.into(), 0))
            )?;
        }
        IpAddr::V6(_) => {
            // need to adjust the below index 0 to something else. ipv6 uses indices
            socket.set_multicast_if_v6(15)?;

            socket.bind(&SockAddr::from(SocketAddr::new(
                Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0).into(),
                0)))?;
        }
    }

    let socket: UdpSocket = socket.into();

    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = tui::backend::CrosstermBackend::new(std::io::stdout());
    let mut terminal = tui::Terminal::new(backend)?;

    terminal.clear()?;

    let mut chat_model = ChatModel::default();

    loop {

        // Update

        // Handle crossterm events like user input
        let has_event = crossterm::event::poll(Duration::from_millis(10)).unwrap();
        if has_event {
            if let CEvent::Key(key) = crossterm::event::read().unwrap() {
                tx.send(Event::Input(key)).unwrap();
            }
        }

        match rx.recv_timeout(Duration::from_millis(10)) {
            Ok(event) => match event {
                Event::Input(event) => match event.code {
                    KeyCode::Char(c) => chat_model.user_input.push(c),
                    KeyCode::Esc => {
                        closing.store(true, Ordering::Relaxed);
                        disable_raw_mode()?;
                        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                        terminal.show_cursor()?;
                        break;
                    }
                    KeyCode::Backspace => { chat_model.user_input.pop(); },
                    KeyCode::Enter => {
                        let message = chat_model.user_input.clone();
                        socket.send_to(message.as_bytes(), multicast_addr)?;

                        chat_model.messages.insert(0, format!("{}: {}", "You", chat_model.user_input));
                        chat_model.user_input.clear();
                    }
                    _ => {}
                },
                Event::MessageReceived(from, message) => {
                    if !chat_model.users.contains(&from) {
                        chat_model.messages.insert(0, format!("{} has joined the chat", from));
                        chat_model.users.insert(from.clone());
                    }
                    chat_model.messages.insert(0, format!("{}: {}", from, message));
                }
            },
            // No new events to process
            Err(_) => {}
        }

        // Draw
        terminal.draw(|f| {
            // ui sample: https://github.com/fdehau/tui-rs/blob/master/examples/user_input.rs
            let size = f.size();

            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(20),
                        Constraint::Min(1),
                    ].as_ref()
                ).split(size);

            let users: Vec<ListItem> = chat_model.users
                .iter()
                .map(|user| ListItem::new(Text::raw(user)))
                .collect();

            let list = List::new(users)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title("Users")
                );

            f.render_widget(list, chunks[0]);

            let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Min(1),
                    Constraint::Length(3)
                ].as_ref()
            ).split(chunks[1]);

            let messages: Vec<ListItem> = chat_model.messages
                .iter()
                .map(|msg| ListItem::new(Text::raw(msg)))
                .collect();

            let list = List::new(messages)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title("Messages")
                ).start_corner(tui::layout::Corner::BottomLeft);

            f.render_widget(list, chunks[0]);

            let paragraph = Paragraph::new(Text::from(chat_model.user_input.as_ref()))
                .block(Block::default()
                    .borders(Borders::all())
                    .title("Enter text")
                );

            f.render_widget(paragraph, chunks[1]);

            f.set_cursor(chunks[1].x + chat_model.user_input.width() as u16 + 1, chunks[1].y + 1);
        })?;
    }

    listener.join().expect("Could not join listener thread");

    Ok(())
}
