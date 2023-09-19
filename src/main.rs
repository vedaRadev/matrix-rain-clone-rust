use termion::{input::TermRead, raw::IntoRawMode};
use rand::{ Rng, distributions::Alphanumeric };
use std::{
    thread,
    collections::HashMap,
    time::Duration,
    io::{Write, stdout},
};

struct RainTrail {
    trail: Vec<char>,
    rng: rand::rngs::ThreadRng,
    column: u16,
    top: i16,
    bottom: i16,
    speed: u8,
    alive: bool,
}

impl RainTrail {
    fn new(len: usize, column: u16) -> Self {
        let mut rng = rand::thread_rng();
        let trail = (0..len).map(|_| rng.sample(Alphanumeric) as char).collect();
        let speed = rng.gen_range(1..=3);
        RainTrail {
            trail,
            rng,
            top: -(len as i16 - 1),
            bottom: 0,
            column,
            alive: true,
            speed,
        }
    }

    fn trickle(&mut self) {
        self.bottom += self.speed as i16;
        self.top += self.speed as i16;
        self.trail.rotate_right(1);
        self.trail[0] = self.rng.sample(Alphanumeric) as char;
    }
}

struct Glass {
    width: u16,
    height: u16,
    rain_trails: Vec<RainTrail>,
    rng: rand::rngs::ThreadRng,
}

impl Glass {
    fn new(width: u16, height: u16) -> Glass {
        Glass {
            width,
            height,
            rain_trails: Vec::new(),
            rng: rand::thread_rng(),
        }
    }

    fn update(&mut self) -> String {
        let mut update_string = String::new();
        for rain_trail in self.rain_trails.iter_mut() {
            if rain_trail.top > self.height as i16 {
                rain_trail.alive = false; // not great but I'll come back and rewrite when I'm more familiar with rust
                // rain_trail shouldn't have to know if it's alive or not
            } else {
                update_string.push_str(
                    &rain_trail.trail
                        .iter()
                        .enumerate()
                        .filter(|(glyph_index, _)| {
                            let glyph_row: i16 = rain_trail.bottom - *glyph_index as i16;
                            glyph_row <= self.height as i16 && glyph_row >= 0
                        })
                        .map(|(glyph_index, glyph)| -> String {
                            format!(
                                "{}{}{}",
                                termion::cursor::Goto(rain_trail.column, rain_trail.bottom as u16 - glyph_index as u16),
                                termion::color::Fg(match glyph_index {
                                    0 => termion::color::Rgb(255, 255, 255),
                                    n @ 1..=5 => termion::color::Rgb(255 / (n + 1) as u8, 150, 255 / (n + 1) as u8),
                                    _ => termion::color::Rgb(0, 150, 0)
                                }),
                                *glyph
                            )
                        })
                        .collect::<String>()[..]
                );

                rain_trail.trickle();
            }

            if rain_trail.top > 0 {
                for i in 1..=rain_trail.speed {
                    update_string.push_str(&format!("{} ", termion::cursor::Goto(rain_trail.column, (rain_trail.top - i as i16) as u16))[..]);
                }
            }
        }

        self.rain_trails.sort_by_key(|RainTrail { alive, .. }| *alive);
        while let Some(rain_trail) = self.rain_trails.last() {
            if rain_trail.alive { break; }
            self.rain_trails.pop();
        }

        update_string
    }

    fn available_columns(&self) -> Option<Vec<u16>> {
        if self.rain_trails.is_empty() { return Some((1..=self.width).collect()); }

        let top_most_rain_trails = self.rain_trails.iter().fold(HashMap::new(), |mut hash_map: HashMap<u16, i16>, rain_trail| {
            // Basically what we're doing here is using a hashmap to easily find the top-most
            // RainTrail in a column. We only care abstdout the top-most RainTrail when
            // determining if there is enough space for a new RainTrail to be added.
            match hash_map.get(&rain_trail.column) {
                Some(other_top) if *other_top > rain_trail.top =>  { hash_map.insert(rain_trail.column, rain_trail.top); }
                None => { hash_map.insert(rain_trail.column, rain_trail.top); },
                _ => ()
            };

            hash_map
        });

        let available_columns: Vec<u16> = (1..=self.width)
            .filter(|column| {
                match top_most_rain_trails.get(column) {
                    Some(top) => *top > 5,
                    None => true
                }
            })
            .collect();
            
        if !available_columns.is_empty() { Some(available_columns) } else { None }
    }

    fn create_rain_trail(&mut self, column: u16) {
        self.rain_trails.push(RainTrail::new(
            self.rng.gen_range(5..(self.height as f64 * 0.8) as usize),
            column
        ));
    }
}

fn main() {
    let (term_width, term_height) = termion::terminal_size().expect("Could not get terminal size");
    
    let stdin = termion::async_stdin();
    let mut key_events = stdin.keys();

    let mut stdout = stdout().into_raw_mode().unwrap(); // terminal raw mode
    write!(stdout, "{}", termion::clear::All).unwrap();
    write!(stdout, "{}", termion::cursor::Hide).unwrap();

    let mut rng = rand::thread_rng();
    let mut glass = Glass::new(term_width, term_height);
    loop {
        if let Some(Ok(termion::event::Key::Ctrl('c'))) = key_events.next() { break; }

        if let Some(available_columns) = glass.available_columns() {
            for column in available_columns {
                if rng.gen_bool(0.02) {
                    glass.create_rain_trail(column);
                }
            }
        }

        write!(stdout, "{}", glass.update()).unwrap();
        stdout.flush().unwrap();

        thread::sleep(Duration::from_millis(50));
    }

    write!(
        stdout,
        "{}{}{}",
        termion::cursor::Show,
        termion::clear::All,
        termion::cursor::Goto(1, 1)
    ).unwrap();
}
