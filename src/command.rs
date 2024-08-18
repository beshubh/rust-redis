use crate::parser::RespData;

pub enum RedisCommand {
    Ping,
    Echo(String),
    Set(String, String, Option<u64>),
    Get(String),
    Info,
    ReplConfListenPort(String, u16),
    ReplConfCapaPsync2,
    Psync,
}

fn parse_px(args: &[RespData]) -> Option<u64> {
    match args {
        [RespData::BulkString(px), RespData::BulkString(ms)] if px.to_uppercase() == "PX" => {
            ms.parse().ok()
        }
        _ => None,
    }
}

pub fn parse_command(data: &RespData) -> Option<RedisCommand> {
    let array = match data {
        RespData::Array(arr) => arr,
        _ => return None,
    };

    let (cmd, args) = array.split_first().unwrap();
    let cmd = match cmd {
        RespData::BulkString(s) => s,
        _ => return None,
    };
    match cmd.to_uppercase().as_str() {
        "PING" => match args {
            [] => Some(RedisCommand::Ping),
            _ => None,
        },
        "ECHO" => match args {
            [RespData::BulkString(message)] => Some(RedisCommand::Echo(message.clone())),
            _ => None,
        },
        "SET" => match args {
            [RespData::BulkString(key), RespData::BulkString(value), rest @ ..] => {
                let px = parse_px(rest);
                Some(RedisCommand::Set(key.clone(), value.clone(), px))
            }
            _ => None,
        },
        "GET" => match args {
            [RespData::BulkString(key)] => Some(RedisCommand::Get(key.clone())),
            _ => None,
        },
        "INFO" => match args {
            [RespData::BulkString(role)] if role == "replication" => Some(RedisCommand::Info),
            _ => None,
        },
        "REPLCONF" => match args {
            [RespData::BulkString(info), RespData::BulkString(port)]
                if info == "listening-port" =>
            {
                Some(RedisCommand::ReplConfListenPort(
                    info.to_string(),
                    port.parse().ok()?,
                ))
            }
            _ => Some(RedisCommand::ReplConfCapaPsync2),
        },
        "PSYNC" => match args {
            // PSYNC ? -1
            [RespData::BulkString(repl_request), RespData::BulkString(init_offset)]
                if repl_request == "?" && init_offset == "-1" =>
            {
                Some(RedisCommand::Psync)
            }
            _ => None,
        },
        _ => None,
    }
}
