use std::thread;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write, ErrorKind};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender, TryRecvError, RecvTimeoutError};
use std::time::Duration;

extern crate modulos_comunes;
use modulos_comunes::{TcpMessage, EMPTYTCPMESSAGE};

fn handle_client(mut stream: TcpStream, tx:Sender<TcpMessage>, rx: Receiver<TcpMessage>) {
    let (sub_tx, sub_rx) = mpsc::channel();
    let stream_clone = stream.try_clone().expect("stream clone fail");
    thread::spawn(move|| {
        handle_subclient(stream_clone, sub_tx)
    });

    let mut hay_cliente = true;
    while hay_cliente {
        //mensajes entrantes del cliente
        match sub_rx.recv_timeout(Duration::new(1,0)) {
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

fn listener_launch(listener: TcpListener, client_tx: Sender<TcpMessage>, sender_tx: Sender<Sender<TcpMessage>>) {
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

fn main() {
    let listener = TcpListener::bind("0.0.0.0:3333").expect("listener failed on bind");
    println!("Server listening on port 3333");
    let (client_tx, server_rx) = mpsc::channel();
    let (sender_tx, sender_rx) = mpsc::channel();
    let mut txs: Vec<Sender<TcpMessage>> = Vec::new();

    listener_launch(listener, client_tx, sender_tx);

    loop {
        loop {
            //Nuevas coneciones
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

        //Lectura de los clientes
        let data: TcpMessage = match server_rx.try_recv() {
            Ok(msg) => {
                msg
            },
            Err(why) => {
                if why == TryRecvError::Empty{
                    EMPTYTCPMESSAGE
                }
                else {
                    panic!("server_rx failed");
                }
            }
        };

        //Escritura a los clientes
        if data != EMPTYTCPMESSAGE {
            let mut index = 0;
            let txsc = txs.clone();
            for tx in txsc {
                match tx.send(data){
                    Ok(_) => { },
                    Err(_) => {
                        txs.remove(index);
                    },
                };
                index += 1;
            }
        }
    }

}
