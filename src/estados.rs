use modulos_comunes::{Estado, from_bytes, TcpMessage};
use std::sync::mpsc::Receiver;

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
 pub fn espear_cambio_de_estado(server_rx: &Receiver<TcpMessage>) -> char {
    loop {
        let data = from_bytes(&server_rx.recv().expect("server_rx, bloqueante, failed")).caracter;
        match data {
        'e'|'p'|'s' => { return data; },
        _ => {},
        };
    }
 }
