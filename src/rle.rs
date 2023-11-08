use anyhow::{anyhow, Result};
use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

#[derive(Debug)]
enum RLEToken {
    Dead(i64),
    Alive(i64),
    EOL,
    EOF,
}

#[derive(Debug)]
enum RLELine {
    Comment(String),
    Header(i64, i64),
    Data(Vec<RLEToken>),
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn parse_header(header: &str) -> Result<RLELine> {
    let re: Regex = Regex::new(r"x[ ]*=(\d+),[ ]*y[ ]*=(\d+).*")?;
    let captures = re.captures(header).ok_or(anyhow!("Invalid header line"))?;

    let x = captures.get(1).ok_or(anyhow!("Header missing x"))?;
    let y = captures.get(2).ok_or(anyhow!("Header missing y"))?;

    Ok(RLELine::Header(
        x.as_str().parse::<i64>()?,
        y.as_str().parse::<i64>()?,
    ))
}

fn parse_data(line: &str) -> Result<RLELine> {
    todo!()
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

pub fn load_rte(filename: &str) -> Result<Vec<Vec<bool>>> {
    let lines = read_lines(filename)?;

    for line in lines {
        let line = line?;
    }

    todo!()
}
