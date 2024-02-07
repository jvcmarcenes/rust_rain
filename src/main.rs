use std::error::Error;
use std::io::{self, Write};

use crossterm::{cursor, style, terminal, QueueableCommand};
use rand::Rng;

static RANDOM_CHARACTERS: &str = concat!(
    "abcdefghijklmnopqrstuvwxyz",
    "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
    "0123456789",
    "!@#$%&*()_-+=[{]}~,.<>;:|\\/?"
);

fn rand_char() -> char {
    let len = RANDOM_CHARACTERS.len();
    let i = rand::thread_rng().gen_range(0..len);
    RANDOM_CHARACTERS.chars().nth(i).unwrap()
}

#[derive(Debug, Clone)]
struct Column(Drop);

#[derive(Debug, Clone, Copy)]
enum Drop {
    Rain(char),
    Stop,
}

#[derive(Debug, Clone)]
struct RainDrop {
    pos: (u16, u16),
    cell: Drop,
}

const RAIN_PROB: u16 = 100;
const STOP_PROB: u16 = 100;

fn main() -> Result<(), Box<dyn Error>> {
    let mut stdout = io::stdout();
    let mut rng = rand::thread_rng();

    stdout.queue(terminal::Clear(terminal::ClearType::All))?;
    stdout.queue(cursor::Hide)?;

    let (width, height) = crossterm::terminal::size()?;

    let mut columns =
        std::iter::repeat(Column(Drop::Stop)).take(width as usize / 2).collect::<Vec<_>>();

    let mut rain = Vec::<RainDrop>::new();

    loop {
        for (x, col) in columns.iter_mut().enumerate() {
            let x = x as u16;
            let (prob, cell) = match col.0 {
                Drop::Rain(_) => (STOP_PROB, Drop::Stop),
                Drop::Stop => (RAIN_PROB, Drop::Rain(rand_char())),
            };

            if rng.gen_range(0..=1000) <= prob {
                rain.push(RainDrop { pos: (x * 2, 0), cell });
                col.0 = cell;
            }
        }

        let mut to_remove = Vec::<usize>::new();
        for (idx, drop) in rain.iter_mut().enumerate() {
            let (x, y) = drop.pos;

            match drop.cell {
                Drop::Rain(ref mut c) => {
                    if y > 0 {
                        stdout.queue(cursor::MoveTo(x, y - 1))?;
                        stdout.queue(style::SetForegroundColor(style::Color::DarkGreen))?;
                        write!(stdout, "{c}")?;
                    }

                    if y < height {
                        stdout.queue(cursor::MoveTo(x, y))?;
                        stdout.queue(style::SetForegroundColor(style::Color::Grey))?;
                        *c = rand_char();
                        write!(stdout, "{c}")?;
                    }
                }
                Drop::Stop => {
                    stdout.queue(cursor::MoveTo(x, y))?;
                    write!(stdout, " ")?;
                }
            }

            drop.pos.1 += 1;

            if drop.pos.1 >= height {
                to_remove.push(idx);
            }
        }

        to_remove.drain(..).for_each(|i| {
            rain.remove(i);
        });

        stdout.flush()?;

        std::thread::sleep(std::time::Duration::from_millis(30));
    }
}
