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
pub mod text;
use std::{borrow::BorrowMut, cell::Cell, cmp::min, fs, os, process::exit};

use clap::Parser;
use expr::{cfg::Cfg, context::Context, Expr};
use forward::executor::{Executor, STOP_SIGNAL};
use futures::{stream::FuturesUnordered, StreamExt};
use galloc::AllocForAny;
use itertools::Itertools;
use mapped_futures::mapped_futures::MappedFutures;
use parser::check::CheckProblem;
use solutions::{new_thread, CONDITIONS};
use tokio::task::JoinHandle;
use value::ConstValue;

use crate::{backward::Problem, expr::cfg::{NonTerminal, ProdRule}, parser::{check::DefineFun, problem::PBEProblem}, solutions::{cond_search_thread, Solutions}, value::Type};
#[derive(Debug, Parser)]
#[command(name = "synthphonia")]
struct Cli {
    /// Log level
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
    /// Path to the context-free grammar configuration (enriched sygus-if)
    #[arg(short, long)]
    cfg: Option<String>,
    
    /// Number of threads
    #[arg(short='j', long, default_value_t=4)]
    thread: usize,
    
    /// No ITE Mode: Generate results without `ite` operator
    #[arg(long)]
    no_ite: bool,
    
    /// Set the rate limit of ITE (in milliseconds), i.e., how much time (without new solutions) does it take for the `ite_limit` to increment by one.
    #[arg(long, default_value_t=4000)]
    ite_limit_rate: usize,
    
    /// Disable deduction, i.e., Enumeration + ACS.
    #[arg(long, default_value_t=false)]
    no_deduction: bool,
    
    /// Enable all-example thread (Using one thread for all-example thread)
    #[arg(long)]
    with_all_example_thread: bool,

    /// Enable constant extraction.
    #[arg(long)]
    extract_constants: bool,
    
    /// Path to the input file: enriched sygus-if (.sl) for synthesis or smt2 (.smt2) to check the result.
    path: String,
    
    /// Debug Mode (More assertions)
    #[arg(short, long)]
    debug: bool,
        
    /// Show examples (debugging)
    #[arg(long)]
    showex: bool,

    /// Show Signature (Just Print the signature without solving)
    #[arg(long)]
    sig: bool
}

#[thread_local]
pub static DEBUG: Cell<bool> = Cell::new(false);

pub static COUNTER: spin::Mutex<[usize; 6]> = spin::Mutex::new([0usize; 6]);

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
            let sygus_if = fs::read_to_string(s).unwrap();
            cfg = enrich_configuration(sygus_if.as_str(), cfg);
        } else {
            let ctx = Context::from_examples(&problem.examples);
            if text::parsing::detector(&ctx) {
                let sygus_if = include_str!("../test/test.sl");
                cfg = enrich_configuration(sygus_if, cfg);
            } else {
                let sygus_if = include_str!("../test/test2map.sl");
                cfg = enrich_configuration(sygus_if, cfg);
            }
        }

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
        debg!("Examples: {:?}", ctx.output);
        if args.showex {
            for i in ctx.inputs() {
                println!("{:?}", i);
            }
            println!("{:?}", ctx.output);
            return Ok(());
        }
        cfg.config.no_deduction = args.no_deduction;
        cfg.config.ite_limit_rate = args.ite_limit_rate;
        if args.no_ite {
            if args.no_ite {
                cfg.config.cond_search = true;
            }
            let exec = Executor::new(ctx, cfg);
            info!("Deduction Configuration: {:?}", exec.deducers);
            let result = exec.solve_top_blocked();
            let func = DefineFun { sig: problem.synthfun().sig.clone(), expr: result};
            println!("{}", func);
        } else {
            let mut solutions = Solutions::new(cfg.clone(), ctx.clone());

            // solutions.create_cond_search_thread();
            let mut nthread = min(args.thread, ctx.len);
            if nthread > 1  && args.with_all_example_thread {
                solutions.create_all_search_thread();
                nthread -= 1;
            }
            for _ in 0..nthread {
                solutions.create_new_thread();
            }

            let result = solutions.solve_loop().await;
            let func = DefineFun { sig: problem.synthfun().sig.clone(), expr: result};
            // let nsols = solutions.count();
            // let ncons = CONDITIONS.lock().as_ref().unwrap().len();
            // eprintln!("nsols: {nsols}, ncons: {ncons}");
            STOP_SIGNAL.store(true, std::sync::atomic::Ordering::Relaxed);
            
            println!("{}", func);

            if solutions.threads.len() != 0 {
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            exit(0);
        }
    }
    Ok(())
}

fn enrich_configuration(sygus_if: &str, mut cfg: Cfg) -> Cfg {
    let problem = PBEProblem::parse(sygus_if).unwrap();
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
    return cfg1;
}

