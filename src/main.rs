use std::net::TcpListener;
use std::sync::mpsc;
use std::sync::mpsc::Sender;

extern crate modulos_comunes;
use modulos_comunes::{TcpMessage,EMPTYTCPMESSAGE, from_bytes};

mod sistema;
use crate::sistema::*;

mod server;
use crate::server::*;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:3333").expect("listener failed on bind");
    println!("Server listening on port 3333");
    let (client_tx, server_rx) = mpsc::channel();
    let (sender_tx, sender_rx) = mpsc::channel();
    let mut txs: Vec<Sender<TcpMessage>> = Vec::new();

    let (tx_pogos, pogos_rx) = mpsc::channel();
    let (pogos_tx, rx_pogos) = mpsc::channel();
    pogos_launch(tx_pogos,rx_pogos);

    let (tx_selector, selector_rx) = mpsc::channel();
    let (selector_tx, rx_selector) = mpsc::channel();
    selector_launch(tx_selector,rx_selector);

    listener_launch(listener, client_tx, sender_tx);

    let mut prev_data = from_bytes(&EMPTYTCPMESSAGE);

    loop {
        recibir_conecciones_nuevas(&sender_rx, &mut txs);

        let data = leer_clientes(&server_rx);

        let estado = ver_estado_del_sistema(
            &data,
            prev_data,
            &pogos_rx,
            &pogos_tx,
            &selector_rx,
            &selector_tx,
        );

        if prev_data != from_bytes(&estado) {
            escribir_clientes(estado, &mut txs);
            prev_data = from_bytes(&estado);
            println!("salió {}", prev_data.selector);
        }
    }

}
