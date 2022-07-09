use chrono::*;
use std::net::*;
use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
    path::Path,
};

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        eprintln!("fatal error : ip:port not found.");
        return;
    }
    let socket = match UdpSocket::bind(args[1].clone()) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("fatal error : failed to bind socket : {}", e);
            return;
        }
    };
    loop {
        let mut buf = [0u8; 1024];
        match socket.recv_from(&mut buf) {
            Ok((size, srcaddr)) => {
                let buf = &buf[..size];
                let mut msg = String::from_utf8_lossy(buf).to_string();
                if msg.ends_with('\n') {
                    msg.pop();
                }
                if msg.ends_with('\r') {
                    msg.pop();
                }
                process(&socket, srcaddr, msg, Local::now());
            }
            Err(e) => {
                eprintln!("runtime warning : failed to recieve request : {:?}", e);
            }
        }
    }
}
fn process(socket: &UdpSocket, srcaddr: SocketAddr, msg: String, dt: DateTime<Local>) {
    println!("{} : {} : {}", dt.format("%Y-%m-%d-%H-%M-%S"), srcaddr, msg);
    let spltd = msg.split(' ').collect::<Vec<&str>>();
    if spltd[0] == "pay" {
        pay(socket, srcaddr, spltd, dt);
    } else if spltd[0] == "list" {
        list(socket, srcaddr);
    } else {
        send_message(socket, srcaddr, "[treasurer] invalid message.\n");
    }
}
fn pay(socket: &UdpSocket, srcaddr: SocketAddr, spltd: Vec<&str>, dt: DateTime<Local>) {
    if spltd.len() != 4 {
        eprintln!("format error : invalid 'pay' message : too few arguments");
        send_message(
            socket,
            srcaddr,
            "[treasurer] invalid 'pay' message : too few arguments\n",
        );
        return;
    }
    if spltd[2].parse::<i64>().is_err() {
        eprintln!("format error : invalid 'pay' message : 'howmuch' is not integer");
        send_message(
            socket,
            srcaddr,
            "[treasurer] invalid 'pay' message : 'howmuch' is not integer\n",
        );
        return;
    }
    let data = format!(
        "{},{},{},{}\n",
        dt.format("%Y-%m-%d-%H-%M-%S"),
        spltd[1],
        spltd[2],
        spltd[3]
    );
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open("./data")
        .unwrap();
    match file.write_all(data.as_bytes()) {
        Ok(()) => {
            println!("record the 'pay' message.");
            send_message(socket, srcaddr, "[treasurer] message sent successfully.\n");
        }
        Err(e) => {
            eprintln!("runtime error : failed to write message : {:?}", e);
            send_message(socket, srcaddr, "[treasurer] failed to record.\n");
        }
    }
}
fn list(socket: &UdpSocket, srcaddr: SocketAddr) {
    let reader = BufReader::new(File::open(Path::new("./data")).unwrap());
    let lines = reader.lines().map(|n| n.unwrap()).collect::<Vec<String>>();
    if lines.is_empty() {
        send_message(socket, srcaddr, "[treasurer] data is empty.\n");
        return;
    }
    let mut res = String::new();
    for l in lines {
        let words = l.split(',').collect::<Vec<&str>>();
        let msg = format!(
            "{}\n    Who: {}\n    Howmuch: {}\n    Forwhat: {}\n",
            words[0], words[1], words[2], words[3]
        );
        res.push_str(&msg);
    }
    send_message(socket, srcaddr, &res);
}
fn send_message(socket: &UdpSocket, srcaddr: SocketAddr, msg: &str) {
    match socket.send_to(msg.as_bytes(), srcaddr) {
        Ok(_) => (),
        Err(e) => eprintln!("runtime error : failed to send responce : {:?}", e),
    }
}
