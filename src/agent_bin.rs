//! ONLY FOR THE AI-TOURNAMENT SERVER
//! NOT INTENDED TO BE RAN MANUALLY

use std::{
    io::{Read, Write},
    net::{Ipv4Addr, SocketAddrV4, TcpStream},
    str::FromStr,
    time::Duration,
};

use clap::Parser;

use crate::board::Board;

mod args;
mod board;
mod engine;
mod mcts;
mod minimax;

fn parse_duration(arg: &str) -> anyhow::Result<Duration> {
    Ok(Duration::from_micros(arg.parse()?))
}

#[derive(Parser, Debug)]
struct EvalArgs {
    port_number: u16,
    #[arg(value_parser = parse_duration)]
    time_budget: Duration,
    #[arg(value_parser = parse_duration)]
    action_timeout: Duration,

    #[clap(flatten)]
    engine_args: args::EngineArgs,
}

fn main() {
    let EvalArgs {
        port_number,
        time_budget: _time_budget, // unused for checkers
        action_timeout,
        engine_args,
    } = EvalArgs::parse();

    // Connect to server
    let addr = SocketAddrV4::new(Ipv4Addr::from_str("127.0.0.1").unwrap(), port_number);
    let mut stream = TcpStream::connect(addr).expect("connection error");

    let mut engine = engine::create_engine(&engine_args);

    loop {
        let mut buf = [0; 4096];
        let n = stream.read(&mut buf).expect("error on stream.read");
        let string = str::from_utf8(&buf[..n]).unwrap();

        let deadline = std::time::Instant::now() + action_timeout;

        // Parse game state, compute action, send it back
        let board = string
            .parse::<Board>()
            .unwrap_or_else(|_| panic!("Got invalid board string: {string}"));
        println!("Game state:\n{board}");
        let action = engine.select(&board, deadline).unwrap();
        println!("Selected action: {action}\n\n");
        stream
            .write_all(action.to_string().as_bytes())
            .expect("could not send (write error)");
    }
}
