use std::net::UdpSocket;

fn send_p2p(ip:&str,value:&str){
    let ip_from="0.0.0.0:1234";
    println!("p2p");
    let socket = UdpSocket::bind(ip_from).expect("could not create socket");
    //socket.set_broadcast(true);
    //socket.set_read_timeout(Some(Duration::new(5, 0)));
    socket.connect(ip).expect("could not connect to peer");
    socket.send(value.as_bytes()).unwrap();
    let mut buf = vec![0; 1024];
    while let Ok((n, addr)) = socket.recv_from(&mut buf) {
        let s = String::from_utf8_lossy(&buf);
        println!("received {n} bytes from {addr}: {}",s);
        buf.resize(n, 0);
    }
}
fn send_cast(value:&str){
    let ip_from="0.0.0.0:1234";
    println!("broadcast");
    let socket = UdpSocket::bind(ip_from).expect("could not create socket");
    //socket.set_broadcast(true);
    //socket.set_read_timeout(Some(Duration::new(5, 0)));
    socket.connect("0.0.0.0:1234").expect("could not connect to peer");
    socket.send(value.as_bytes()).unwrap();
    let mut buf = vec![0; 1024];
    while let Ok((n, addr)) = socket.recv_from(&mut buf) {
        let s = String::from_utf8_lossy(&buf);
        println!("received {n} bytes from {addr}: {}",s);
        buf.resize(n, 0);
    }
}
fn main(){
    //send_p2p("192.168.1.2:1234","abc");
    send_cast("def");
}