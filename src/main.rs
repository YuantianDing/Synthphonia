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
pub mod text;
use std::fs;

use clap::Parser;
use expr::{cfg::Cfg, context::Context};
use forward::executor::Executor;
use galloc::AllocForAny;
use itertools::Itertools;

use crate::{expr::cfg::{NonTerminal, ProdRule}, parser::problem::PBEProblem, value::Type};
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
    let mut cfg = Cfg::from_synthfun(problem.synthfun());
    if let Some(s) = args.cfg {
        let s = fs::read_to_string(s).unwrap();
        let problem = PBEProblem::parse(s.as_str()).unwrap();
        let mut synthfun = problem.synthfun().clone();
        synthfun.cfg.start = synthfun.cfg.get_nt_by_type(&cfg[0].ty);
        synthfun.cfg.reset_start();
        let mut cfg1 = Cfg::from_synthfun(&synthfun);
        for nt in cfg1.iter_mut() {
            nt.rules.retain(|x| !matches!(x, ProdRule::Var(_)));
        }
        for (nt1, nt) in cfg1.iter_mut().zip(cfg.iter()) {
            for r in nt.rules.iter() {
                if let ProdRule::Const(_) | ProdRule::Var(_) = r {
                    nt1.rules.push(r.clone());
                }
            }
        }
        cfg = cfg1;
    };
    info!("CFG: {:?}", cfg);
    let ctx = Context::from_problem(&problem);
    let exec = Executor::new(ctx, cfg).galloc();
    info!("Deduction Configuration: {:?}", exec.deducers);
    let result = exec.block_on(exec.spawn_task(0, exec.ctx.output));
    println!("{:?}", result.unwrap());
    Ok(())
}