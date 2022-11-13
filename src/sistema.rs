use std::sync::mpsc::{Receiver,TryRecvError, Sender};
use std::thread::{spawn,sleep};
use std::time::Duration;

use gpiod::{Chip, Options, EdgeDetect};

extern crate modulos_comunes;
use modulos_comunes::{TcpMessage, DataStruct, Convert, Estado};

fn pausa(estado: &mut Estado){
    match estado {
        Estado::Pausa => {
            *estado = Estado::Marcha;
        },
        Estado::Marcha => {
            *estado = Estado::Pausa;
        },
        _ => { },
    }
}

fn emergencia(estado: &mut Estado){
    *estado = Estado::Parado;
}

fn inicio(estado: &mut Estado){
    *estado = Estado::Marcha;
}

pub fn gestionar_estado(nuevo: char, estado: &mut Estado){
    match nuevo {
        'p' => { pausa(estado); },
        'e' => { emergencia(estado); },
        's' => { inicio(estado); },
         _  => { },
    }
}

pub fn ver_estado_del_sistema(
    data: char,
    estado: &mut Estado,
    prev_data: DataStruct,
    pogos_rx: &Receiver<bool>,
    selector_rx: &Receiver<bool>,
    selector_tx: &Sender<bool>,
    cinta1_rx: &Receiver<bool>,
    cinta1_tx: &Sender<bool>,
    sensor1_rx: &Receiver<bool>,
    cinta2_rx: &Receiver<bool>,
    sensor2_rx: &Receiver<bool>,
) -> TcpMessage {
    match data {
        'h' => {
            cinta1_tx.send(prev_data.cinta1 ^ true).expect("No se envió");
        },
        'l' => {
            selector_tx.send(prev_data.selector ^ true).expect("No se envió");
        },
        'e'|'s'|'p' => {
            gestionar_estado(data, estado);
        }
        _ => {}
    };

    let ret = DataStruct {
        cinta1: match cinta1_rx.try_recv(){
            Ok(on) => {
                on
            },
            Err(why) => {
                if why == TryRecvError::Empty {
                    prev_data.cinta1
                } else {
                    panic!("Perdimos la cinta1");
                }
            },
        },
        cinta2: match cinta2_rx.try_recv(){
            Ok(on) => {
                on
            },
            Err(why) => {
                if why == TryRecvError::Empty {
                    prev_data.cinta2
                } else {
                    panic!("Perdimos la cinta2");
                }
            },
        },
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
        sensor1: match sensor1_rx.try_recv(){
            Ok(plac) => {
                plac
            },
            Err(why) => {
                if why == TryRecvError::Empty {
                    prev_data.sensor1
                } else {
                    panic!("Perdimos los pogos");
                }
            },
        },
        sensor2: match sensor2_rx.try_recv(){
            Ok(plac) => {
                plac
            },
            Err(why) => {
                if why == TryRecvError::Empty {
                    prev_data.sensor2
                } else {
                    panic!("Perdimos los pogos");
                }
            },
        },
        caracter: Default::default(),
        estado: *estado,
    }.to_bytes();

    ret
}

pub fn pogos_launch(
    tx: Sender<bool>,
    rx: Receiver<bool>,
){
    spawn(move || {
        let (mut high, mut low) = (
            Duration::from_micros(600),
            Duration::from_micros(19600)
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
                        Duration::from_micros(2530),
                        Duration::from_micros(17470)
                    )} else {(
                        Duration::from_micros(600),
                        Duration::from_micros(19600)
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
            Duration::from_micros(2100),
            Duration::from_micros(17900)
        );

        let chip = Chip::new("gpiochip3").expect("No se abrió el chip, selector"); // open chip

        let opts = Options::output([6]) // configure lines offsets
            .values([false]) // optionally set initial values
            .consumer("my-outputs"); // optionally set consumer string

        let outputs = chip.request_lines(opts).expect("Pedido de salidas rechazado, selector");

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

pub fn cinta1_launch(
    tx_cinta: Sender<bool>,
    _rx_cinta: Receiver<bool>,
    tx_sensor: Sender<bool>,
    tx_cinta2: Sender<bool>,
    pogos_tx: Sender<bool>,
){
    spawn(move || {
        let chip = Chip::new("gpiochip2").expect("No se abrió el chip, cinta1"); // open chip

        let opts = Options::output([19]) // configure lines offsets
            .values([false]) // optionally set initial values
            .consumer("my-outputs"); // optionally set consumer string

        let ipts = Options::input([2]) // configure lines offsets
            .edge(EdgeDetect::Rising) 
            .consumer("my-inputs"); // optionally set consumer string

        let outputs = chip.request_lines(opts).expect("Pedido de lineas rechazado, cinta1 out");
        let mut inputs = chip.request_lines(ipts).expect("Pedido de lines rechazado, cinta1 in");

        let wait = Duration::from_secs(5);
        let pausa = Duration::from_secs(1);

        loop{
            outputs.set_values([true]).expect("No se seteó high, cinta1");
            tx_cinta.send(true).expect("Rip tx_cinta cinta1");
            while inputs.get_values([false;1]).expect("No se leyó true, cinta1") == [true] { }
            tx_sensor.send(false).expect("Rip tx_sensor cinta1");
            while inputs.get_values([false;1]).expect("No se leyó false, cinta1") == [false] {
                let event = inputs.read_event().expect("No se leyó el evento, cinta1");
                println!("Evento: {:}",event);
            }
            tx_sensor.send(true).expect("Rip tx_sensor cinta1");
            outputs.set_values([false]).expect("No se seteó low, cinta1");
            tx_cinta.send(false).expect("Rip tx_cinta cinta1");
            sleep(pausa);
            pogos_tx.send(true).expect("No se envió a pogos, cinta1");
            sleep(wait);
            pogos_tx.send(false).expect("No se envió a pogos, cinta1");
            sleep(pausa);
            tx_cinta2.send(true).expect("No salió a la otra cinta, cinta1");
        }
    });
}

pub fn cinta2_launch(
    tx_cinta: Sender<bool>,
    rx_cinta: Receiver<bool>,
    tx_sensor: Sender<bool>,
){
    spawn(move || {
        let chip = Chip::new("gpiochip3").expect("No se abrió el chip, cinta2"); // open chip

        let opts = Options::output([7]) // configure lines offsets
            .values([false]) // optionally set initial values
            .consumer("my-outputs"); // optionally set consumer string

        let ipts = Options::input([5]) // configure lines offsets
            .edge(EdgeDetect::Falling) 
            .consumer("my-inputs"); // optionally set consumer string

        let outputs = chip.request_lines(opts).expect("Pedido de lineas rechazado, cinta2 out");
        let mut inputs = chip.request_lines(ipts).expect("Pedido de lines rechazado, cinta2 in");

        loop{
            rx_cinta.recv().expect("No se recivó, cinta2"); //Recibe de forma bloqueante la salida de una placa
            outputs.set_values([true]).expect("No se seteó high, cinta2");
            tx_cinta.send(true).expect("Rip tx_cinta cinta2");
            while inputs.get_values([false;1]).expect("No se leyó falsse, cinta2") == [false] { }
            tx_sensor.send(true).expect("Rip tx_sensor cinta2");
            while inputs.get_values([false;1]).expect("No se leyó true, cinta2") == [true] {
                let event = inputs.read_event().expect("No se leyó el evento, cinta2");
                println!("Evento: {:?}",event);
            }
            tx_sensor.send(false).expect("Rip tx_sensor cinta2");
            outputs.set_values([false]).expect("No se seteó low, cinta2");
            tx_cinta.send(false).expect("Rip tx_cinta cinta2");
        }
    });
}
