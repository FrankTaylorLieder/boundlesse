use anyhow::{anyhow, Result};
use log::*;
use regex::Regex;
use std::cmp::max;
use std::fs::File;
use std::io::{BufRead, BufReader};

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

// fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
// where
//     P: AsRef<Path>,
// {
//     let file = File::open(filename)?;
//     Ok(io::BufReader::new(file).lines())
// }

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
pub fn load_rle(filename: &str) -> Result<Vec<Vec<bool>>> {
    let lines = read_lines(filename)?;
    let lines: Vec<RLELine> = lines
        .iter()
        .map(|l| parse_line(l.as_str()))
        .collect::<Result<Vec<RLELine>>>()?;
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

    let x: usize;
    let y: usize;

    match lines.next() {
        Some(RLELine::Header(hx, hy)) => {
            x = *hx;
            y = *hy;
        }
        Some(_) => return Err(anyhow!("Missing header")),
        _ => return Err(anyhow!("Unexpected EOF reading header")),
    }

    let mut data = Vec::with_capacity(y);
    let mut line = Vec::with_capacity(x);
    for dl in lines {
        match dl {
            RLELine::Comment(_) => return Err(anyhow!("Comment found in data")),
            RLELine::Header(_, _) => return Err(anyhow!("Header found in data")),
            RLELine::Data(tokens) => {
                for t in tokens {
                    match t {
                        RLEToken::Dead(c) => {
                            for _ in 0..*c {
                                line.push(false);
                            }
                        }
                        RLEToken::Alive(c) => {
                            for _ in 0..*c {
                                line.push(true);
                            }
                        }
                        RLEToken::EOL(c) => {
                            for _ in line.len()..x {
                                line.push(false);
                            }

                            data.push(line.clone());

                            // Push empty lines for repeated $
                            for _ in 0..c - 1 {
                                data.push(vec![false; x as usize]);
                            }

                            line = Vec::with_capacity(x);
                        }
                        RLEToken::EOF => {
                            for _ in line.len()..x {
                                line.push(false);
                            }

                            data.push(line.clone());

                            if data.len() < y {
                                return Err(anyhow!("Too few lines"));
                            };
                        }
                    }
                }
            }
        }
    }

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::assert_eq;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn load_single() -> Result<()> {
        init();
        let data = load_rle("patterns/single.rle")?;

        assert_eq!(data.len(), 1);
        assert_eq!(data[0].len(), 1);
        assert_eq!(data[0][0], true);

        Ok(())
    }
}
