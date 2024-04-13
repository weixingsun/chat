use std::io;
use std::net::UdpSocket;
use clap::{arg, Command};

fn send_p2p(ip:&str,value:&str){
    let ip_from="0.0.0.0:1234";
    println!("p2p");
    let socket = UdpSocket::bind(ip_from).expect("could not create socket");
    //socket.set_broadcast(true);
    //socket.set_read_timeout(Some(Duration::new(5, 0)));
    socket.connect(ip).expect("could not connect to peer");
    socket.send(value.as_bytes()).unwrap();
}
fn send_cast(value:&str){
    let ip_from="0.0.0.0:12340";
    println!("broadcast");
    let socket = UdpSocket::bind(ip_from).expect("could not create socket");
    //socket.set_broadcast(true);
    //socket.set_read_timeout(Some(Duration::new(5, 0)));
    socket.connect("0.0.0.0:1234").expect("could not connect to peer");
    socket.send(value.as_bytes()).unwrap();
}
fn recv_new(){
    let socket = UdpSocket::bind("0.0.0.0:1234").unwrap();
    socket.set_read_timeout(Some(std::time::Duration::from_millis(20))).unwrap();
    let mut buf = [0; 1024];
    let mut received_count = 0;
    loop {
        match socket.recv_from(&mut buf) {
        Ok((n, addr)) => {
            received_count += 1;
            let s = String::from_utf8_lossy(&buf);
            println!("received {n} bytes from {addr}: {}",s);
        }
        Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
            continue;
        }
        Err(e) => {
            print!("failed to receive a datagram: {}",e);
            break;
        }
        }
    }
}
fn main(){
    //send_p2p("192.168.1.2:1234","abc");
    //send_cast("def");
    let matches = Command::new("BitSpot")                                                                         .version("v0.0.2 20240408")
        .author("Weixing Sun <weixing.sun@gmail.com>")                                                            .about("BitSpot Robot")
        .arg(arg!(--server).required(false))
        .arg(arg!(--interval <VALUE>).required(false))
        .get_matches();
    let server = *matches.get_one::<bool>("server").unwrap();
    if server{
        recv_new();
    }else{
        send_cast("abc");
    }
}
