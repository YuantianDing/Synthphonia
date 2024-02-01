#![feature(int_roundings)]
#![feature(unchecked_math)]
#![feature(thread_local)]
#![feature(trait_upcasting)]
#![feature(noop_waker)]
#![feature(map_try_insert)]
#![feature(async_fn_in_trait)]
#![feature(cell_update)]
#![feature(trait_alias)]

pub mod galloc;
pub mod log;
pub mod utils;
pub mod parser;
pub mod value;
pub mod expr;
pub mod forward;
pub mod backward;
use std::fs;

use clap::Parser;
use expr::{cfg::Cfg, context::Context};
use forward::executor::Executor;
use galloc::AllocForAny;

use crate::parser::problem::PBEProblem;
#[derive(Debug, Parser)]
#[command(name = "stren")]
struct Cli {
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
    path: String,
    #[arg(short, long)]
    cfg: Option<String>,
}



fn main() -> Result<(), Box<dyn std::error::Error>>{
    let args = Cli::parse();
    log::set_log_level(args.verbose + 2);

    let s = fs::read_to_string(args.path).unwrap();
    let problem = PBEProblem::parse(s.as_str()).unwrap();
    let cfg = Cfg::from_synthfun(problem.synthfun());
    let ctx = Context::from_problem(&problem);
    let exec = Executor::new(ctx, cfg).galloc();
    let result = exec.block_on(exec.spawn_task(0, exec.ctx.output));
    println!("{:?}", result);
    Ok(())
}