use std::sync::mpsc::{Receiver,TryRecvError, Sender};
use std::thread::{spawn,sleep};
use std::time::Duration;

use gpiod::{Chip, Options};

extern crate modulos_comunes;
use modulos_comunes::{TcpMessage, DataStruct, Convert};

fn pausa(){}

pub fn ver_estado_del_sistema(
    data: &TcpMessage,
    prev_data: DataStruct,
    pogos_rx: &Receiver<bool>,
    pogos_tx: &Sender<bool>,
    selector_rx: &Receiver<bool>,
    selector_tx: &Sender<bool>,
) -> TcpMessage {
    match data {
        b"j" => {
            pogos_tx.send(prev_data.pogos ^ true).expect("No se envió");
        },
        b"l" => {
            selector_tx.send(prev_data.selector ^ true).expect("No se envió");
        },
        b"p" => {
            pausa();
        }
        _ => {}
    };

    let ret = DataStruct {
        cinta: false,
        pogos: match pogos_rx.try_recv(){
            Ok(pos) => {
                pos
            },
            Err(why) => {
                if why == TryRecvError::Empty {
                    prev_data.pogos
                } else {
                    panic!("Perdimos los pogos");
                }
            },
        },
        selector: match selector_rx.try_recv(){
            Ok(pos) => {
                pos
            },
            Err(why) => {
                if why == TryRecvError::Empty {
                    prev_data.selector
                } else {
                    panic!("Perdimos los pogos");
                }
            },
        },
        sensor: false,
    }.to_bytes();

    ret
}

pub fn pogos_launch(
    tx: Sender<bool>,
    rx: Receiver<bool>,
){
    spawn(move || {
        let (mut high, mut low) = (
            Duration::from_micros(550),
            Duration::from_micros(19450)
        );

        let chip = Chip::new("gpiochip3").expect("No se abrió el chip, pogos"); // open chip

        let opts = Options::output([4]) // configure lines offsets
            .values([false]) // optionally set initial values
            .consumer("my-outputs, pogos"); // optionally set consumer string

        let outputs = chip.request_lines(opts).expect("Pedido de salidas rechazado, pogos");

        loop {
            (high,low) = match rx.try_recv(){
                Ok(pos) => {
                    tx.send(pos).expect("Chingo el msg, pogos");
                    if pos {(
                        Duration::from_micros(550),
                        Duration::from_micros(19450)
                    )} else {(
                        Duration::from_micros(2530),
                        Duration::from_micros(17470)
                    )}
                },
                Err(why) => {
                    if why == TryRecvError::Empty {
                        (high, low)
                    } else {
                        panic!("Perdimos los pogos");
                    }
                },
            };
            outputs.set_values([true]).expect("No se seteó el high, pogos");
            sleep(high);
            outputs.set_values([false]).expect("No se seteó el low, pogos");
            sleep(low);
        }

    });
}

pub fn selector_launch(
    tx: Sender<bool>,
    rx: Receiver<bool>,
){
    spawn(move || {
        let (mut high, mut low) = (
            Duration::from_micros(1200),
            Duration::from_micros(18800)
        );

        let chip = Chip::new("gpiochip3").expect("No se abrió el chip, "); // open chip

        let opts = Options::output([6]) // configure lines offsets
            .values([false]) // optionally set initial values
            .consumer("my-outputs"); // optionally set consumer string

        let outputs = chip.request_lines(opts).expect("Pedido de salidas rechazado");

        loop {
            (high,low) = match rx.try_recv(){
                Ok(pos) => {
                    tx.send(pos).expect("Chingo el msg, selector");
                    if pos {(
                        Duration::from_micros(1200),
                        Duration::from_micros(18800)
                    )} else {(
                        Duration::from_micros(2100),
                        Duration::from_micros(17900)
                    )}
                },
                Err(why) => {
                    if why == TryRecvError::Empty {
                        (high, low)
                    } else {
                        panic!("Perdimos el selector");
                    }
                },
            };
            outputs.set_values([true]).expect("No se seteó el high, selector");
            sleep(high);
            outputs.set_values([false]).expect("No se seteó el low, selector");
            sleep(low);
        }

    });
}
