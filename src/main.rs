use std::net::TcpListener;
use std::sync::mpsc;
use std::sync::mpsc::Sender;

extern crate modulos_comunes;
use modulos_comunes::TcpMessage;

mod server;
use crate::server::*;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:3333").expect("listener failed on bind");
    println!("Server listening on port 3333");
    let (client_tx, server_rx) = mpsc::channel();
    let (sender_tx, sender_rx) = mpsc::channel();
    let mut txs: Vec<Sender<TcpMessage>> = Vec::new();

    listener_launch(listener, client_tx, sender_tx);

    loop {
        recibir_conecciones_nuevas(&sender_rx, &mut txs);

        let data = leer_clientes(&server_rx);

        escribir_clientes(data, &mut txs);
    }

}
