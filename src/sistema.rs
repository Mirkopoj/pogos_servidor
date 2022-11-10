use std::sync::mpsc::{Receiver,TryRecvError, Sender};
use std::thread::{spawn,sleep};
use std::time::Duration;

use gpiod::{Chip, Options, EdgeDetect};
extern crate i2c_linux;
use i2c_linux::I2c;

extern crate modulos_comunes;
use modulos_comunes::DataStruct;

fn pausa(){}

pub fn ver_estado_del_sistema(
    data: DataStruct,
    prev_data: DataStruct,
    pogos_rx: &Receiver<bool>,
    selector_rx: &Receiver<bool>,
    selector_tx: &Sender<bool>,
    cinta1_rx: &Receiver<bool>,
    cinta1_tx: &Sender<bool>,
    sensor1_rx: &Receiver<bool>,
    cinta2_rx: &Receiver<bool>,
    sensor2_rx: &Receiver<bool>,
) -> DataStruct {
    match &data.caracter {
        b"h" => {
            cinta1_tx.send(prev_data.cinta1 ^ true).expect("No se envió");
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
        caracter: [0;1],
    };

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
            probar_placa();
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

fn adc() -> [f64;3] {
    let mut i2c = I2c::from_path("/dev/i2c-0").expect("i2c");
    i2c.smbus_set_slave_address(0x08, false).expect("addr");
    let mut cont = 0;
    i2c.smbus_write_byte(42).expect("Write");//Sincronizar el pic
    let mut tensiones: [f64;3] = [0.0;3];
    for _ in 0..3000{
        let adch = i2c.smbus_read_byte().expect("ReadH") as u16;
        let adcl = i2c.smbus_read_byte().expect("ReadL") as u16;
        let adc:u16 = (adch <<8) + adcl;
        let tension = adc as f64 * 0.003225806452;
        tensiones[cont as usize] += tension;
        cont += 1;
        cont %= 3;
        sleep(Duration::from_millis(1));
    }

    for v in tensiones.as_mut() {
        *v /= 1000.0;
    }

    tensiones
}

fn probar_placa(){
    let tensiones = adc();
    println!("{:?}",tensiones);
    let wait = Duration::from_secs(5);
    sleep(wait);
}
