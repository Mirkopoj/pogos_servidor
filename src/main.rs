use std::net::TcpListener;
use std::sync::mpsc;
use std::sync::mpsc::Sender;

extern crate modulos_comunes;
use modulos_comunes::{TcpMessage, from_bytes, Estado};

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

    let (tx_cinta2, cinta2_rx) = mpsc::channel();
    let (cinta2_tx, rx_cinta2) = mpsc::channel();
    let (tx_sensor2, sensor2_rx) = mpsc::channel();
    cinta2_launch(tx_cinta2,rx_cinta2,tx_sensor2);

    let (tx_cinta1, cinta1_rx) = mpsc::channel();
    let (cinta1_tx, rx_cinta1) = mpsc::channel();
    let (tx_sensor1, sensor1_rx) = mpsc::channel();
    cinta1_launch(tx_cinta1,rx_cinta1,tx_sensor1,cinta2_tx,pogos_tx);

    listener_launch(listener, client_tx, sender_tx);

    let mut prev_data = Default::default();

    let mut estado = Estado::Parado;

    loop {
        recibir_conecciones_nuevas(&sender_rx, &mut txs);

        let data = leer_clientes(&server_rx, &mut estado);

        let situacion = ver_estado_del_sistema(
            data,
            &mut estado,
            prev_data,
            &pogos_rx,
            &selector_rx,
            &selector_tx,
            &cinta1_rx,
            &cinta1_tx,
            &sensor1_rx,
            &cinta2_rx,
            &sensor2_rx,
        );

        if prev_data != from_bytes(&situacion) {
            escribir_clientes(situacion, &mut txs);
            prev_data = from_bytes(&situacion);
            println!("salió {:?}", prev_data.estado);
        }
    }

}
