use std::thread;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write, ErrorKind};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender, TryRecvError, RecvTimeoutError};
use std::time::Duration;

extern crate modulos_comunes;
use modulos_comunes::{TcpMessage, EMPTYTCPMESSAGE, from_bytes, Estado};

use crate::sistema::gestionar_estado;

fn handle_client(mut stream: TcpStream, tx:Sender<TcpMessage>, rx: Receiver<TcpMessage>) {
    let (sub_tx, sub_rx) = mpsc::channel();
    let stream_clone = stream.try_clone().expect("stream clone fail");
    thread::spawn(move|| {
        handle_subclient(stream_clone, sub_tx)
    });

    let mut hay_cliente = true;
    while hay_cliente {
        //mensajes entrantes del cliente
        match sub_rx.recv_timeout(Duration::new(0,1)) {
            Ok(msg) => {
                tx.send(msg).expect("sub_tx failed");
            },
            Err(why) => {
                if why == RecvTimeoutError::Disconnected{
                    println!("reader_tx terminated");
                    hay_cliente = false;
                } 
            }
        };

        //mensajes salientes del server
        match rx.try_recv() {
            Ok(data) => {
                match stream.write(&data) {
                    Ok(_) => { },
                    Err(why) => { println!("stream write failed {}", why); },
                };
            },
            Err(why) => {
                if why != TryRecvError::Empty {
                    println!("An error occurred, rx client");
                    hay_cliente = false;
                }
            }
        } 
    }

    drop(stream);
}

fn handle_subclient(mut stream: TcpStream, sub_tx: Sender<TcpMessage>) {
    let addr = stream.peer_addr().expect("Stream peer_addr failed on subclient");
    let mut data: TcpMessage = EMPTYTCPMESSAGE; // using 50 byte buffer
    while match stream.read(&mut data) {
        Ok(size) => {
            if size==0 { 
                println!("Connection with {}, closed", addr);
                false 
            } else {
                sub_tx.send(data).expect("tx subclient, falló");
                true
            }
        },
        Err(why) => {
            if why.kind().eq(&ErrorKind::WouldBlock) { 
                true 
            }
            else {
                println!("An error occurred, terminating connection with {}", addr);
                false
            }
        }
    } {}
}

pub fn listener_launch(listener: TcpListener, client_tx: Sender<TcpMessage>, sender_tx: Sender<Sender<TcpMessage>>) {
    thread::spawn( move || {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("New connection: {}", stream.peer_addr().expect("Stream peer_addr failed on new connection"));
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
}

pub fn leer_clientes(server_rx: &Receiver<TcpMessage>, estado: &mut Estado) -> char {
    while *estado != Estado::Marcha {
        let data = from_bytes(&server_rx.recv().expect("server_rx, bloqueante, failed")).caracter;
        match data {
            'e'|'p'|'s' => { gestionar_estado(data, estado); },
            _ => {},
        };
    }
    match server_rx.try_recv() {
        Ok(msg) => {
            from_bytes(&msg).caracter
        },
        Err(why) => {
            if why == TryRecvError::Empty{
                Default::default()
            }
            else {
                panic!("server_rx failed");
            }
        }
    }
}

pub fn recibir_conecciones_nuevas(sender_rx: &Receiver<Sender<TcpMessage>>, txs: &mut Vec<Sender<TcpMessage>>) {
    loop {
        match sender_rx.try_recv() {
            Ok(sender) => {
                txs.push(sender);
            },
            Err(why) => {
                if why == TryRecvError::Empty{
                    break;
                } else {
                    panic!("reader_tx terminated");
                }
            }
        };
    }
}

pub fn escribir_clientes(data: TcpMessage, txs: &mut Vec<Sender<TcpMessage>>) {
    let mut index = 0;
    let txsc = txs.clone();
    for tx in txsc {
        match tx.send(data){
            Ok(_) => {
                index += 1;
            },
            Err(_) => {
                txs.remove(index);
            },
        };
    }
}

