use std::thread;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write, ErrorKind};
use std::str::from_utf8;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender, TryRecvError, RecvTimeoutError};
use std::time::Duration;

fn handle_client(mut stream: TcpStream, tx:Sender<[u8;50]>, rx: Receiver<[u8;50]>) {
    stream.set_read_timeout(Some(Duration::new(1, 0))).expect("Stream set duration failed");
    let (sub_tx, sub_rx) = mpsc::channel();
    let stream_clone = stream.try_clone().expect("stream clone fail");
    thread::spawn(move|| {
        // connection succeeded
        handle_subclient(stream_clone, sub_tx)
    });

    let mut hay_cliente = true;
    while hay_cliente {
        loop {
            match sub_rx.recv_timeout(Duration::new(1,0)) {
                Ok(msg) => {
                    //println!("llegó {} 1", from_utf8(&msg).unwrap());
                    tx.send(msg).expect("sub_tx failed");
                },
                Err(why) => {
                    if why == RecvTimeoutError::Timeout{
                        break;
                    } else {
                        println!("reader_tx terminated");
                        hay_cliente = false;
                        break;
                    }
                }
            };
        }
        match rx.try_recv() {
            Ok(data) => {
                match stream.write(&data[0..50]) {
                    Ok(_) => { 
                        println!("salió {} 2", from_utf8(&data).unwrap());
                    },
                    Err(why) => { println!("stream write failed {}", why); },
                };
            },
            Err(why) => {
                if why != TryRecvError::Empty {
                    println!("An error occurred, rx client");
                    break;
                }
            }
        } 
    }

    drop(stream);
}

fn handle_subclient(mut stream: TcpStream, sub_tx: Sender<[u8;50]>) {
    let addr = stream.peer_addr().expect("Stream peer_addr failed on subclient");
    let mut data = [0 as u8; 50]; // using 50 byte buffer
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
                //println!("Timeout read from connection {}", stream.peer_addr().expect("Stream peer_addr failed on error"));
                true 
            }
            else {
                println!("An error occurred, terminating connection with {}", addr);
                false
            }
        }
    } {}
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:3333").expect("listener failed on bind");
    // accept connections and process them, spawning a new thread for each one
    println!("Server listening on port 3333");
    let (client_tx, server_rx) = mpsc::channel();
    let (sender_tx, sender_rx) = mpsc::channel();
    let mut txs: Vec<Sender<[u8;50]>> = Vec::new();

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
        let data = match server_rx.try_recv() {
            Ok(msg) => {
                //println!("llegó {} 2", from_utf8(&msg).unwrap());
                msg
            },
            Err(why) => {
                if why == TryRecvError::Empty{
                    [0;50]
                }
                else {
                    panic!("server_rx failed");
                }
            }
        };

        //Escritura a los clientes
        if data != [0;50]{
            let mut index = 0;
            let txsc = txs.clone();
            for tx in txsc {
                //println!("salió {} 1", from_utf8(&data).unwrap());
                match tx.send(data){
                    Ok(_) => { },
                    Err(why) => {
                        println!("Broadcast failed {}", why);
                        txs.remove(index);
                    },
                };
                index += 1;
            }
        }
    }

}
