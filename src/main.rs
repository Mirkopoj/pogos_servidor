use std::thread;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};

fn handle_client(mut stream: TcpStream, tx:Sender<[u8;50]>, rx: Receiver<[u8;50]>) {
    let (sub_tx, sub_rx) = mpsc::channel();
    let stream_clone = stream.try_clone().expect("stream clone fail");
    thread::spawn(move|| {
        // connection succeeded
        handle_subclient(stream_clone, sub_tx)
    });

    loop {
        match sub_rx.try_recv() {
            Ok(msg) => {
                tx.send(msg).expect("sub_tx failed");
            },
            Err(why) => {
                if why == TryRecvError::Empty {
                    break;
                } else {
                    panic!("reader_tx terminated")
                }
            }
        };
        match rx.recv() {
            Ok(data) => {
                stream.write(&data[0..50]).unwrap();
            },
            Err(_) => {
                println!("An error occurred, rx");
                break;
            }
        } {}
    }

    stream.shutdown(Shutdown::Both).unwrap();
    drop(stream);
}

fn handle_subclient(mut stream: TcpStream, tx: Sender<[u8;50]>) {
    let mut data = [0 as u8; 50]; // using 50 byte buffer
    while match stream.read(&mut data) {
        Ok(size) => {
            if size==0 { 
                println!("Connection with {}, closed", stream.peer_addr().unwrap());
                false 
            } else {
                tx.send(data).expect("tx subclient, falló");
                true
            }
        },
        Err(_) => {
            println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
            false
        }
    } {}
    stream.shutdown(Shutdown::Both).unwrap();
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:3333").unwrap();
    // accept connections and process them, spawning a new thread for each one
    println!("Server listening on port 3333");
    let (client_tx, server_rx) = mpsc::channel();
    let (sender_tx, sender_rx) = mpsc::channel();
    let mut txs: Vec<Sender<[u8;50]>> = Vec::new();

    thread::spawn( move || {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("New connection: {}", stream.peer_addr().unwrap());
                    let (server_tx, client_rx) = mpsc::channel();
                    let txcli = client_tx.clone();
                    sender_tx.clone().send(server_tx).expect("sendersender fail");
                    thread::spawn(move|| {
                        // connection succeeded
                        handle_client(stream, txcli, client_rx)
                    });
                }
                Err(e) => {
                    println!("Error: {}", e);
                    /* connection failed */
                }
            }
        }
        // close the socket server
        drop(listener);
    });

    loop {
        match sender_rx.try_recv() {
            Ok(sender) => {
                txs.push(sender);
            },
            Err(why) => {
                if why == TryRecvError::Empty {
                    break;
                } else {
                    panic!("reader_tx terminated")
                }
            }
        };
    }

    let data = server_rx.recv().expect("server_rx failed");

    for tx in txs {
        tx.send(data).expect("Broadcast failed");
    }

}
