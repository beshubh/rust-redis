use nom::branch::alt;
use nom::bytes::complete::{tag, take, take_until};
use nom::character::complete::char;
use nom::multi::count;
use nom::sequence::delimited;
use nom::IResult;
use std::vec::Vec;

pub struct RespMessage {
    pub raw_string: String,
}

impl RespMessage {
    pub fn new(raw_string: String) -> Self {
        Self { raw_string }
    }

    pub fn build_reply(&self) -> String {
        let commands_vec = self
            .raw_string
            .split(' ')
            .map(String::from)
            .collect::<Vec<_>>();
        let mut command_strign = String::new();
        for command in &commands_vec {
            command_strign.push_str(format!("${}\r\n{}\r\n", command.len(), command).as_str())
        }
        format!("*{}\r\n{}", commands_vec.len(), command_strign)
    }
}

#[derive(Debug, PartialEq)]
pub enum RespData {
    SimpleString(String),
    Error(String),
    BulkString(String),
    BulkStringNull,
    Array(Vec<RespData>),
}

fn parse_simple_string(input: &str) -> IResult<&str, RespData> {
    let (input, data) = delimited(char('+'), take_until("\r\n"), tag("\r\n"))(input)?;
    Ok((input, RespData::SimpleString(data.to_string())))
}

fn parse_bulk_string(input: &str) -> IResult<&str, RespData> {
    let (input, str_len) = delimited(char('$'), take_until("\r\n"), tag("\r\n"))(input)?;
    let str_len = str_len.parse::<i64>().map_err(|_| {
        nom::Err::Failure(nom::error::Error::new(input, nom::error::ErrorKind::Digit))
    })?;
    if str_len == -1 {
        Ok((input, RespData::BulkStringNull))
    } else {
        let (input, data) = take(str_len as usize)(input)?;
        let (input, _) = tag("\r\n")(input)?;
        Ok((input, RespData::BulkString(data.to_string())))
    }
}

fn parse_array(input: &str) -> IResult<&str, RespData> {
    let (input, array_len) = delimited(char('*'), take_until("\r\n"), tag("\r\n"))(input)?;
    let array_len = array_len.parse::<i64>().map_err(|_| {
        nom::Err::Failure(nom::error::Error::new(input, nom::error::ErrorKind::Digit))
    })?;
    let (input, elements) = count(parse_resp, array_len as usize)(input)?;
    Ok((input, RespData::Array(elements)))
}

fn parse_error(input: &str) -> IResult<&str, RespData> {
    let (input, data) = delimited(char('-'), take_until("\r\n"), tag("\r\n"))(input)?;
    Ok((input, RespData::Error(data.to_string())))
}

pub fn parse_resp(input: &str) -> IResult<&str, RespData> {
    alt((
        parse_simple_string,
        parse_error,
        parse_bulk_string,
        parse_simple_string,
        parse_array,
    ))(input)
}
