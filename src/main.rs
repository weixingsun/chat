use std::net::UdpSocket;
use std::collections::HashMap;
use clap::{arg, Command};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_timestamp() -> i64 {
    let since_epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    return since_epoch.as_millis() as i64;
}
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
fn recv_new(timeout:i64){
    let socket = UdpSocket::bind("0.0.0.0:1234").unwrap();
    socket.set_read_timeout(Some(std::time::Duration::from_millis(20))).unwrap();
    let mut buf = [0; 1024];
    let mut map:HashMap<String,String> = HashMap::new();
    let start_time=get_timestamp();
    loop {
        match socket.recv_from(&mut buf) {
        Ok((_n, addr)) => {
            let s = String::from_utf8_lossy(&buf).into_owned();
            let s = s.trim_end_matches(char::from(0));
            let s = s.trim_matches(char::from(0));
            map.insert(addr.to_string(),s.to_owned());
            //println!("received {n} bytes from {addr}: {}",s);
            println!("{:?}",map);
            let curr_time=get_timestamp();
            let delta=curr_time-start_time;
            if delta>timeout{break}
        }
        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
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
        .arg(arg!(--ip <VALUE>).required(false))
        .get_matches();
    let server = *matches.get_one::<bool>("server").unwrap();
    let ip = matches.get_one::<String>("ip");
    let ip = if ip.is_none() {"".to_owned()} else {ip.unwrap().to_owned()};
    if server{
        recv_new(10000);
    }else if ip.eq("p2p"){
        send_p2p(&ip,"p2p");
    }else {
        send_cast("cast");
    }
}
