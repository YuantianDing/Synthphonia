#![allow(unused_imports)] 
#![allow(unused_mut)] 
#![feature(int_roundings)]
#![feature(unchecked_math)]
#![feature(thread_local)]
#![feature(trait_upcasting)]
#![feature(noop_waker)]
#![feature(map_try_insert)]
#![feature(async_fn_in_trait)]
#![feature(hash_raw_entry)]
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
// pub mod text;
use std::{borrow::BorrowMut, cell::Cell, cmp::min, fs, process::exit};

use clap::Parser;
use expr::{cfg::Cfg, context::Context, Expr};
use forward::executor::Enumerator;
use futures::{stream::FuturesUnordered, StreamExt};
use galloc::AllocForAny;
use itertools::Itertools;
use mapped_futures::mapped_futures::MappedFutures;
use parser::check::CheckProblem;
use solutions::new_thread;
use value::ConstValue;

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
    #[arg(long)]
    extract_constants: bool,
    path: String,
    #[arg(short, long)]
    cfg: Option<String>,
    #[arg(long)]
    sig: bool
}

#[thread_local]
pub static DEBUG: Cell<bool> = Cell::new(false);

pub static COUNTER: spin::Mutex<[usize; 6]> = spin::Mutex::new([0usize; 6]);

fn main() -> Result<(), Box<dyn std::error::Error>>{
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

        if args.extract_constants {
            let constants = problem.examples.extract_constants();
            for nt in cfg.iter_mut() {
                if nt.ty == Type::Str {
                    for c in constants.iter() {
                        nt.rules.push(ProdRule::Const(ConstValue::Str(c)));
                    }
                }
            }
        }

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
            let mut exec = Enumerator::new(ctx, cfg).galloc_mut();
            info!("Deduction Configuration: {:?}", exec.deducers);
            smol::block_on(async {
                let result = exec.solve_top_blocked();
                let func = DefineFun { sig: problem.synthfun().sig.clone(), expr: result};
                println!("{}", func);
                // exit(0);
            })
        } else {
            let mut solutions = Solutions::new(cfg.clone(), ctx.clone());

            solutions.create_cond_search_thread();
            for i in 0..min(args.thread, ctx.len) {
                solutions.create_new_thread();
            }

            smol::block_on(async {
                let result = solutions.solve_loop().await;
                let func = DefineFun { sig: problem.synthfun().sig.clone(), expr: result};
                println!("{}", func);
                // exit(0);
            })
        }
    }
    Ok(())
}



#[cfg(test)]
mod test {
    use std::{thread, time::Duration};
    use smol::future;

    use smol::{Executor, Timer};

    struct A();
    impl Drop for A {
        fn drop(&mut self) {
            println!("Dropped");
        }
    }
    #[test]
    fn test() {
        let ex = Executor::new();
    
        // Spawn a deamon future.
        let task = ex.spawn(async {
            let a = A();
            loop {
                println!("Even though I'm in an infinite loop, you can still cancel me!");
                Timer::after(Duration::from_secs(1)).await;
            }
        });
        
        // Run an executor thread.
        thread::spawn(move || future::block_on(ex.run(future::pending::<()>())));
        
        future::block_on(async {
            Timer::after(Duration::from_secs(3)).await;
            task.cancel().await;
        });
    }
}
