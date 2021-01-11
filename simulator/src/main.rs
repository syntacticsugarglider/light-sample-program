use crossterm::{
    cursor,
    event::{read, Event, KeyCode, KeyModifiers},
    style::{style, Color, PrintStyledContent},
    terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType},
    QueueableCommand,
};
use cursor::{Hide, MoveTo, Show};
use smol::{block_on, Timer};
use std::{cell::Cell, env::args, fs::File, intrinsics::transmute, io::Read, time::Duration};
use std::{
    convert::TryInto,
    io::{stdout, Write},
};
use wasmer_runtime::{imports, instantiate, Func, Global, Memory, Value};

#[repr(C)]
pub union OutputData {
    unbuffered: (u8, u8, [u8; 3]),
    buffered: (u8, u8, u32),
}

#[repr(C)]
pub struct Output {
    buffered: bool,
    data: OutputData,
}

fn main() {
    let path = args().skip(1).next().expect("no wasm binary specified");
    let mut buffer = vec![];
    File::open(path)
        .expect("could not open wasm binary")
        .read_to_end(&mut buffer)
        .expect("could not read wasm binary");

    let import_object = imports! {};

    let instance = instantiate(&buffer, &import_object).expect("invalid wasm binary");

    let memory: Memory = instance.exports.get("memory").expect("memory missing");

    let reverse = args()
        .skip(2)
        .next()
        .map(|a| a == "--reverse")
        .unwrap_or(false);

    std::thread::spawn(move || {
        block_on(async {
            let add_one: Func<(), u32> = instance.exports.get("entry").expect("entrypoint missing");

            enable_raw_mode().unwrap();
            let mut stdout = stdout();
            stdout.queue(Clear(ClearType::All)).unwrap();
            stdout.queue(Hide).unwrap();
            stdout.flush().unwrap();

            loop {
                let (_, height) = size().unwrap();
                let ret = match add_one.call() {
                    Ok(data) => data,
                    Err(_) => {
                        stdout.queue(Clear(ClearType::All)).unwrap();
                        stdout.queue(MoveTo(0, 0)).unwrap();
                        let ptr = instance.exports.get::<Global>("PANIC_DATA").unwrap().get();
                        let len = instance.exports.get::<Global>("PANIC_LEN").unwrap().get();
                        let mut data = format!("(no data)");
                        if let Value::I32(ptr) = ptr {
                            if let Value::I32(len) = len {
                                let len = u32::from_le_bytes(
                                    memory.view()[len as usize..len as usize + 4]
                                        .iter()
                                        .map(Cell::get)
                                        .collect::<Vec<_>>()
                                        .as_slice()
                                        .try_into()
                                        .unwrap(),
                                );
                                let len = len as usize;
                                if len != 0 {
                                    let ptr = ptr as usize;
                                    let ptr = u32::from_le_bytes(
                                        memory.view()[ptr..ptr + 4]
                                            .iter()
                                            .map(Cell::get)
                                            .collect::<Vec<_>>()
                                            .as_slice()
                                            .try_into()
                                            .unwrap(),
                                    );
                                    let ptr = ptr as usize;
                                    if let Ok(message) = String::from_utf8(
                                        memory.view()[ptr..ptr + len]
                                            .iter()
                                            .map(Cell::get)
                                            .collect(),
                                    ) {
                                        data = message;
                                    }
                                }
                            }
                        }
                        print!("-- PANIC --\r\n{}\r\n\r\n^C to quit", data);
                        stdout.flush().unwrap();
                        break;
                    }
                };
                let ret = ret as usize;
                let mut data = [0u8; std::mem::size_of::<Output>()];
                for (byte, target) in memory.view()[ret..ret + std::mem::size_of::<Output>()]
                    .iter()
                    .map(Cell::get)
                    .zip(data.iter_mut())
                {
                    *target = byte;
                }
                let data: Output = unsafe { transmute(data) };
                if data.buffered {
                    let (start, end, ptr) = unsafe { data.data.buffered };
                    let len = (end - start + 1) as usize * 3;
                    let mut target = vec![0u8; len];
                    for (byte, target) in memory.view()[ptr as usize..ptr as usize + len]
                        .iter()
                        .map(Cell::get)
                        .zip(target.iter_mut())
                    {
                        *target = byte;
                    }
                    for (idx, color) in target.chunks_exact(3).enumerate() {
                        let y = idx / 8;
                        let x = if y % 2 == 0 { idx % 8 } else { 7 - idx % 8 };
                        stdout
                            .queue(MoveTo(
                                if reverse {
                                    (7 - x as u16) * 2
                                } else {
                                    x as u16 * 2
                                },
                                if reverse {
                                    y as u16
                                } else {
                                    height - 1 - y as u16
                                },
                            ))
                            .unwrap();
                        stdout
                            .queue(PrintStyledContent(style('⬤').with(Color::Rgb {
                                r: color[0],
                                g: color[1],
                                b: color[2],
                            })))
                            .unwrap();
                    }
                    stdout.flush().unwrap();
                } else {
                    println!("{:?}", unsafe { data.data.unbuffered });
                }
                Timer::after(Duration::from_millis(50)).await;
            }
        })
    });

    loop {
        match read().unwrap() {
            Event::Key(event) => {
                if let KeyCode::Char('c') = event.code {
                    if event.modifiers.contains(KeyModifiers::CONTROL) {
                        let mut stdout = stdout();
                        stdout.queue(Show).unwrap();
                        stdout.flush().unwrap();
                        disable_raw_mode().unwrap();
                        println!("\ncaught ^C, quitting");
                        break;
                    }
                }
            }
            _ => {}
        }
    }
}
