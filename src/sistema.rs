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
    pogos_tx: &Sender<bool>
) -> TcpMessage {
    match data {
        b"j" => {
            pogos_tx.send(prev_data.pogos ^ true).expect("No se envió");
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
        selector: false,
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

        let chip = Chip::new("gpiochip3")?; // open chip

        let opts = Options::output([4]) // configure lines offsets
            .values([false]) // optionally set initial values
            .consumer("my-outputs"); // optionally set consumer string

        let outputs = chip.request_lines(opts)?;

        loop {
            (high,low) = match rx.try_recv(){
                Ok(pos) => {
                    tx.send(pos).expect("Chingo el msg");
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
            }
            outputs.set_values([true])?;
            sleep(high);
            outputs.set_values([false])?;
            sleep(low);
        }

    });
}
