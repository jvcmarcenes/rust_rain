use std::collections::HashSet;
use std::error::Error;
use std::io::{self, Write};
use std::time::Duration;

use crossterm::style::Color;
use crossterm::{cursor, event, style, terminal, QueueableCommand};
use rand::Rng;
use uuid::Uuid;

mod char_sets;

#[derive(Clone, Copy)]
enum Direction {
    Up,
    Down,
}

struct Config {
    char_set: &'static str,
    direction: Direction,
    delay: Duration,
    color: Color,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            char_set: char_sets::ASCII,
            direction: Direction::Down,
            delay: Duration::from_millis(60),
            color: Color::DarkGreen,
        }
    }
}
struct State {
    config: Config,
    terminal_size: (u16, u16),
}

impl State {
    /// Generates a random character that's contained in [`RANDOM_CHARACTERS`]
    fn rand_char(&self) -> char {
        let len = self.config.char_set.chars().count();
        let i = rand::thread_rng().gen_range(0..len);
        self.config.char_set.chars().nth(i).unwrap()
    }

    /// Moves a particle according to the current `Direction`
    fn move_particle(&self, (x, y): (u16, u16)) -> Option<(u16, u16)> {
        match self.config.direction {
            Direction::Up => Some((x, y.checked_sub(1)?)),
            Direction::Down => {
                Some((x, y.checked_add(1).filter(|&pos| pos < self.terminal_size.1)?))
            }
        }
    }
}

/// The kind for a particle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ParticleKind {
    /// Falls down, generating random characters in its path.
    Rain(char),
    /// Changes the color of the character at its position.
    Trail(char),
    /// Falls down, clearing the characters in its path.
    Clear,
}

/// A particle emitter.
#[derive(Debug, Clone)]
struct ParticleEmitter {
    last: ParticleKind,
}

/// A particle will move through the screen and modify characters on it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Particle {
    /// A unique identifier for this particle, for removing it.
    id: Uuid,
    /// The position of this particle on the screen
    pos: (u16, u16),
    /// The kind for this particle.
    kind: ParticleKind,
}

/// The perthousand probability that a given column will spawn a raining particle.
const RAIN_PROB: u16 = 50;
/// The perthousand probability that a given column will spawn a clearing particle.
const CLEAR_PROB: u16 = 50;

/// How much we fill the columns. default is 2 (every other column).
const COLUMN_GAP: usize = 2;

fn main() -> Result<(), Box<dyn Error>> {
    let mut config = Config::default();

    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--set" => {
                let Some(arg) = args.next() else {
                    println!("invalid usage, expected character set");
                    return Ok(());
                };
                config.char_set = match arg.as_str() {
                    "ascii" => char_sets::ASCII,
                    "jp" => char_sets::JAPONESE,
                    _ => {
                        println!("unknown character set");
                        return Ok(());
                    }
                };
            }
            "-r" => {
                config.direction = Direction::Up;
            }
            "--delay" => {
                let Some(arg) = args.next() else {
                    println!("invalid usage, expected delay duration (in ms)");
                    return Ok(());
                };
                let Ok(delay) = arg.parse::<u64>() else {
                    println!("could not parse delay");
                    return Ok(());
                };
                config.delay = Duration::from_millis(delay);
            }
            "--color" => {
                let Some(arg) = args.next() else {
                    println!("invalid usage, expected color");
                    return Ok(());
                };
                config.color = match arg.as_str() {
                    "green" => Color::DarkGreen,
                    "red" => Color::DarkRed,
                    "blue" => Color::DarkBlue,
                    "cyan" => Color::DarkCyan,
                    "magenta" => Color::DarkMagenta,
                    "white" => Color::White,
                    _ => {
                        println!("unknown color");
                        return Ok(());
                    }
                };
            }
            opt => {
                println!("unknown option: {opt}");
                return Ok(());
            }
        }
    }

    let mut stdout = io::stdout();
    let mut rng = rand::thread_rng();

    terminal::enable_raw_mode()?;

    stdout.queue(terminal::Clear(terminal::ClearType::All))?.queue(cursor::Hide)?;

    let terminal_size @ (width, _) = terminal::size()?;

    let state = State { config, terminal_size };

    let mut columns =
        vec![ParticleEmitter { last: ParticleKind::Clear }; width as usize / COLUMN_GAP];

    let mut rain = Vec::<Particle>::new();

    loop {
        // update particle emitters
        for (x, col) in columns.iter_mut().enumerate() {
            let x = x as u16;

            let switch_prob = match col.last {
                ParticleKind::Rain(_) => CLEAR_PROB,
                ParticleKind::Clear => RAIN_PROB,
                _ => unreachable!(),
            };

            if rng.gen_range(0..=1000) <= switch_prob {
                let kind = match col.last {
                    ParticleKind::Rain(_) => ParticleKind::Clear,
                    ParticleKind::Clear => ParticleKind::Rain('*'),
                    _ => unreachable!(),
                };

                col.last = kind;

                let y = match state.config.direction {
                    Direction::Up => state.terminal_size.1 - 1,
                    Direction::Down => 0,
                };

                rain.push(Particle {
                    id: Uuid::new_v4(),
                    pos: (x * COLUMN_GAP as u16, y),
                    kind,
                });
            }
        }
        //

        // update particles
        let mut to_remove = HashSet::<Uuid>::new();
        let mut to_add = HashSet::<Particle>::new();

        for particle in rain.iter_mut() {
            let (x, y) = particle.pos;
            stdout.queue(cursor::MoveTo(x, y))?;

            match particle.kind {
                ParticleKind::Rain(ref mut c) => {
                    stdout.queue(style::SetForegroundColor(Color::White))?;
                    *c = state.rand_char();
                    write!(stdout, "{c}")?;

                    to_add.insert(Particle {
                        id: Uuid::new_v4(),
                        pos: (x, y),
                        kind: ParticleKind::Trail(*c),
                    });

                    match state.move_particle(particle.pos) {
                        Some(pos) => particle.pos.1 = pos.1,
                        None => {
                            to_remove.insert(particle.id);
                        }
                    }
                }
                ParticleKind::Trail(c) => {
                    stdout.queue(style::SetForegroundColor(state.config.color))?;
                    write!(stdout, "{c}")?;

                    to_remove.insert(particle.id);
                    // continue;
                }
                ParticleKind::Clear => {
                    write!(stdout, " ")?;

                    match state.move_particle(particle.pos) {
                        Some(pos) => particle.pos.1 = pos.1,
                        None => {
                            to_remove.insert(particle.id);
                        }
                    }
                }
            }
        }

        stdout.flush()?;

        rain.retain(|part| !to_remove.contains(&part.id));

        rain.extend(to_add);
        //

        // read input
        if event::poll(state.config.delay)? {
            let event = event::read()?;
            if let event::Event::Key(event) = event {
                let q_pressed = event.code == event::KeyCode::Char('q');

                let ctrl_c_pressed = {
                    let ctrl_pressed = event.modifiers.contains(event::KeyModifiers::CONTROL);
                    let c_pressed = event.code == event::KeyCode::Char('c');
                    ctrl_pressed && c_pressed
                };

                if q_pressed || ctrl_c_pressed {
                    break;
                }
            }
        }
        //
    }

    // cleanup
    stdout.queue(style::SetForegroundColor(style::Color::White))?;
    stdout.queue(terminal::Clear(terminal::ClearType::All))?;
    stdout.queue(cursor::MoveTo(0, 0))?;
    //

    Ok(())
}
