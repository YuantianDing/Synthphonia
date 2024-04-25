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
pub mod tree_learning;
pub mod solutions;
pub mod text;
use std::{borrow::BorrowMut, cell::Cell, cmp::min, fs, process::exit};

use clap::Parser;
use expr::{cfg::Cfg, context::Context, Expr};
use forward::executor::Executor;
use futures::{stream::FuturesUnordered, StreamExt};
use galloc::AllocForAny;
use itertools::Itertools;
use mapped_futures::mapped_futures::MappedFutures;
use parser::check::CheckProblem;
use solutions::new_thread;
use tokio::task::JoinHandle;

use crate::{backward::Problem, expr::cfg::{NonTerminal, ProdRule}, parser::{check::DefineFun, problem::PBEProblem}, solutions::{cond_search_thread, Solutions}, value::Type};
#[derive(Debug, Parser)]
#[command(name = "stren")]
struct Cli {
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
    #[arg(short, long)]
    debug: bool,
    #[arg(long)]
    showex: bool,
    #[arg(short='j', long, default_value_t=4)]
    thread: usize,
    #[arg(long)]
    cond_search: bool,
    path: String,
    #[arg(short, long)]
    cfg: Option<String>,
    #[arg(long)]
    sig: bool
}

#[thread_local]
pub static DEBUG: Cell<bool> = Cell::new(false);

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let args = Cli::parse();
    log::set_log_level(args.verbose + 2);
    DEBUG.set(args.debug);
    if args.sig {
        let s = fs::read_to_string(args.path).unwrap();
        let problem = PBEProblem::parse(s.as_str()).unwrap();
        
        println!("{}", problem.synthfun().sig)
    } else if args.path.ends_with(".smt2") {
        let s = fs::read_to_string(args.path).unwrap();
        let problem = CheckProblem::parse(s.as_str()).unwrap();
        let ctx = Context::from_examples(&problem.examples);
        info!("Expression: {:?}", problem.definefun.expr);
        info!("Examples: {:?}", problem.examples);
        let result = problem.definefun.expr.eval(&ctx);
        info!("Result: {:?}", result);
        println!("{}", result.eq_count(&problem.examples.output));
    } else {
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
        let ctx = Context::from_examples(&problem.examples);
        if args.showex {
            for i in ctx.inputs() {
                println!("{:?}", i);
            }
            println!("{:?}", ctx.output);
            return Ok(());
        }
        if args.thread == 1 {
            if args.cond_search {
                cfg.config.cond_search = true;
            }
            let exec = Executor::new(ctx, cfg).galloc();
            info!("Deduction Configuration: {:?}", exec.deducers);
            let result = exec.block_on(exec.spawn_task(Problem::root(0, exec.ctx.output))).expect("Failure");
            let func = DefineFun { sig: problem.synthfun().sig.clone(), expr: result};
            println!("{}", func);
        } else {
            let mut solutions = Solutions::new(cfg.clone(), ctx.clone());

            solutions.create_cond_search_thread();
            for i in 0..min(args.thread, ctx.len) {
                solutions.create_new_thread();
            }

            let result = solutions.solve_loop().await;
            let func = DefineFun { sig: problem.synthfun().sig.clone(), expr: result};
            println!("{}", func);
            exit(0);
        }
    }
    Ok(())
}



