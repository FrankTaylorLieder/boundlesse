use anyhow::{anyhow, Result};
use log::*;
use regex::Regex;
use std::cmp::max;
use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::grid::GridCoord;

pub trait Inject {
    fn inject(&mut self, coord: GridCoord, alive: bool) -> anyhow::Result<()>;
}

#[derive(Debug)]
enum RLEToken {
    Dead(u32),
    Alive(u32),
    EOL(u32),
    EOF,
}

#[derive(Debug)]
enum RLELine {
    Comment(String),
    Header(usize, usize),
    Data(Vec<RLEToken>),
}

fn read_lines(path: &str) -> Result<Vec<String>> {
    let input = File::open(path)?;
    let buffered = BufReader::new(input);

    let lines: Vec<String> = buffered
        .lines()
        .collect::<Result<Vec<String>, std::io::Error>>()?;

    Ok(lines)
}

fn parse_header(header: &str) -> Result<RLELine> {
    let re: Regex = Regex::new(r"^x[ ]*=[ ]*(\d+),[ ]*y[ ]*=[ ]*(\d+).*$")?;
    let captures = re.captures(header).ok_or(anyhow!("Invalid header line"))?;

    let x = captures.get(1).ok_or(anyhow!("Header missing x"))?;
    let y = captures.get(2).ok_or(anyhow!("Header missing y"))?;

    Ok(RLELine::Header(
        x.as_str().parse::<usize>()?,
        y.as_str().parse::<usize>()?,
    ))
}

fn parse_data(line: &str) -> Result<RLELine> {
    let mut tokens: Vec<RLEToken> = vec![];

    let mut count: u32 = 0;

    for c in line.chars() {
        match c {
            'o' => {
                tokens.push(RLEToken::Alive(max(count, 1)));
                count = 0;
            }
            'b' => {
                tokens.push(RLEToken::Dead(max(count, 1)));
                count = 0;
            }
            '$' => {
                tokens.push(RLEToken::EOL(max(count, 1)));
                count = 0;
            }
            '!' => tokens.push(RLEToken::EOF),
            c => {
                if c.is_digit(10) {
                    count *= 10;
                    count += c.to_digit(10).ok_or(anyhow!("Invalid run length"))?;
                } else {
                    return Err(anyhow!("Malformed data: {c}"));
                }
            }
        }
    }

    Ok(RLELine::Data(tokens))
}

fn parse_line(line: &str) -> Result<RLELine> {
    let trimmed = line.trim();
    if trimmed.starts_with("#") {
        Ok(RLELine::Comment(trimmed.to_owned()))
    } else if trimmed.starts_with("x") {
        parse_header(trimmed)
    } else {
        parse_data(trimmed)
    }
}

#[allow(unused)]
pub fn load_rle(filename: &str, inject: &mut impl Inject, skip_blank: bool) -> anyhow::Result<()> {
    let lines = read_lines(filename)?;
    let lines: Vec<RLELine> = lines
        .iter()
        .map(|l| parse_line(l.as_str()))
        .collect::<Result<Vec<RLELine>>>()?;

    trace!("Loaded {} RLELines", lines.len());

    let mut lines = lines.iter().peekable();

    while lines
        .next_if(|&l| match l {
            RLELine::Comment(s) => {
                debug!("Ignoring coment: {s}");
                true
            }
            _ => false,
        })
        .is_some()
    {}

    let max_x: i64;
    let max_y: i64;

    match lines.next() {
        Some(RLELine::Header(hx, hy)) => {
            max_x = *hx as i64;
            max_y = *hy as i64;
        }
        Some(_) => return Err(anyhow!("Missing header")),
        _ => return Err(anyhow!("Unexpected EOF reading header")),
    }

    // Offset the pattern to center as 0,0
    let offset_x = -(max_x / 2);
    let offset_y = -(max_y / 2);

    let mut x: i64 = 0;
    let mut y: i64 = 0;
    for dl in lines {
        match dl {
            RLELine::Comment(_) => return Err(anyhow!("Comment found in data")),
            RLELine::Header(_, _) => return Err(anyhow!("Header found in data")),
            RLELine::Data(tokens) => {
                for t in tokens {
                    match t {
                        RLEToken::Dead(c) => {
                            if skip_blank {
                                x += *c as i64;
                            } else {
                                for _ in 0..*c {
                                    inject.inject(
                                        GridCoord::Valid(x + offset_x, y + offset_y),
                                        false,
                                    );
                                    x += 1;
                                }
                            }
                        }
                        RLEToken::Alive(c) => {
                            for _ in 0..*c {
                                inject.inject(GridCoord::Valid(x + offset_x, y + offset_y), true);
                                x += 1;
                            }
                        }
                        RLEToken::EOL(c) => {
                            if !skip_blank {
                                for _ in x..max_x {
                                    inject.inject(
                                        GridCoord::Valid(x + offset_x, y + offset_y),
                                        false,
                                    );
                                    x += 1;
                                }
                            }

                            x = 0;
                            y += 1;

                            // Push empty lines for repeated $
                            for _ in 0..c - 1 {
                                if !skip_blank {
                                    for i in 0..max_x {
                                        inject.inject(
                                            GridCoord::Valid(i as i64 + offset_x, y + offset_y),
                                            false,
                                        );
                                    }
                                }

                                y += 1;
                            }
                        }
                        RLEToken::EOF => {
                            if !skip_blank {
                                for _ in x..max_x {
                                    inject.inject(
                                        GridCoord::Valid(x + offset_x, y + offset_y),
                                        false,
                                    );
                                    x += 1;
                                }
                            }

                            y += 1;

                            if y < max_y {
                                return Err(anyhow!("Too few lines"));
                            };
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{assert_eq, collections::HashMap};

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    struct TestData {
        pub injects: usize,
        pub coords: HashMap<GridCoord, bool>,
    }

    impl TestData {
        pub fn new() -> Self {
            TestData {
                injects: 0,
                coords: HashMap::new(),
            }
        }
    }

    impl Inject for TestData {
        fn inject(&mut self, coord: GridCoord, alive: bool) -> anyhow::Result<()> {
            self.injects += 1;
            if alive {
                self.coords.insert(coord, alive);
            }

            Ok(())
        }
    }
    #[test]
    fn load_single() -> Result<()> {
        init();
        let mut data = TestData::new();
        load_rle("patterns/single.rle", &mut data, true)?;

        let hm = data.coords;

        assert_eq!(data.injects, 1);
        assert_eq!(hm.len(), 1);
        assert_eq!(
            *hm.get(&GridCoord::Valid(0, 0)).expect("Missing cell"),
            true
        );

        Ok(())
    }

    #[test]
    fn load_enormous() -> Result<()> {
        init();

        let mut data = TestData::new();
        load_rle("patterns/gemini.rle", &mut data, true)?;

        // TODO Check the result... for now we'll just check it loads.

        Ok(())
    }
}
