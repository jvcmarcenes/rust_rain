use std::error::Error;
use std::io::{self, Write};

use crossterm::{cursor, event, style, terminal, QueueableCommand};
use rand::Rng;

/// A static string containing all characters that the program will use.
static RANDOM_CHARACTERS: &str = concat!(
    "abcdefghijklmnopqrstuvwxyz",
    "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
    "0123456789",
    "!@#$%&*()_-+=[{]}~,.<>;:|\\/?"
);

/// Generates a random character that's contained in [`RANDOM_CHARACTERS`]
fn rand_char() -> char {
    let len = RANDOM_CHARACTERS.len();
    let i = rand::thread_rng().gen_range(0..len);
    RANDOM_CHARACTERS.chars().nth(i).unwrap()
}

/// The state of a column, what was the last particle emitted.
#[derive(Debug, Clone)]
struct Column(ParticleKind);

/// The kind for a particle.
#[derive(Debug, Clone, Copy)]
enum ParticleKind {
    /// Falls down, generating random characters in its path.
    Rain(char),
    /// Falls down, clearing the characters in its path.
    Clear,
}

/// A particle will move through the screen and modify characters on it.
#[derive(Debug, Clone)]
struct Particle {
    /// The position of this particle on the screen
    pos: (u16, u16),
    /// The kind for this particle.
    kind: ParticleKind,
}

/// The perthousand probability that a given column will spawn a raining particle.
const RAIN_PROB: u16 = 50;
/// The perthousand probability that a given column will spawn a clearing particle.
const CLEAR_PROB: u16 = 50;

fn main() -> Result<(), Box<dyn Error>> {
    let mut stdout = io::stdout();
    let mut rng = rand::thread_rng();

    terminal::enable_raw_mode()?;

    stdout.queue(terminal::Clear(terminal::ClearType::All))?;
    stdout.queue(cursor::Hide)?;

    let (width, height) = terminal::size()?;

    let mut columns =
        std::iter::repeat(Column(ParticleKind::Clear)).take(width as usize / 2).collect::<Vec<_>>();

    let mut rain = Vec::<Particle>::new();

    loop {
        for (x, col) in columns.iter_mut().enumerate() {
            let x = x as u16;
            let (prob, cell): (_, fn() -> _) = match col.0 {
                ParticleKind::Rain(_) => (CLEAR_PROB, || ParticleKind::Clear),
                ParticleKind::Clear => (RAIN_PROB, || ParticleKind::Rain(rand_char())),
            };

            if rng.gen_range(0..=1000) <= prob {
                let cell = cell();
                rain.push(Particle { pos: (x * 2, 0), kind: cell });
                col.0 = cell;
            }
        }

        let mut to_remove = Vec::<usize>::new();
        for (idx, drop) in rain.iter_mut().enumerate() {
            let (x, y) = drop.pos;

            match drop.kind {
                ParticleKind::Rain(ref mut c) => {
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
                ParticleKind::Clear => {
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

        let wait_time = std::time::Duration::from_millis(30);
        if event::poll(wait_time)? {
            let event = event::read()?;
            if let event::Event::Key(event) = event {
                if matches!(event.code, event::KeyCode::Char('q')) {
                    break;
                }
            }
        }
    }

    stdout.queue(terminal::Clear(terminal::ClearType::All))?;
    stdout.queue(cursor::MoveTo(0, 0))?;

    Ok(())
}
