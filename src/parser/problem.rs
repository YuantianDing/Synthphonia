use derive_more::Display;
use itertools::Itertools;
use pest::{
    iterators::{Pair, Pairs},
    Parser,
};

pub use pest::Position;
pub use pest::Span;

use crate::{
    galloc::{self},
    value::Type,
};

use super::{ioexamples::IOExamples, prod::ProdRule, config::{Config, self}};
use derive_more::DebugCustom;

pub type Error = pest::error::Error<Rule>;

/// Creates a new custom error instance associated with a specific span. 

pub fn new_custom_error_span<'i>(msg: String, span: Span<'i>) -> Error { Error::new_from_span(pest::error::ErrorVariant::CustomError { message: msg }, span) }
/// Constructs and returns an error with a custom error message and position. 

pub fn new_costom_error_pos<'i>(msg: String, pos: Position<'i>) -> Error { Error::new_from_pos(pest::error::ErrorVariant::CustomError { message: msg }, pos) }

#[derive(DebugCustom, PartialEq, Eq, Hash, Clone)]
#[debug(fmt = "{} : {:?} -> {:?}", _0, _1, _2)]
/// A struct that encapsulates a non-terminal symbol in the context of string synthesis. 
/// 
/// This struct includes a `String` representing the non-terminal's identifier, a `Type` indicating the expected data type or category the non-terminal belongs to, and a `Vec<ProdRule>` which comprises a collection of production rules associated with this non-terminal. 
/// Additionally, it contains a `Config` to store any configuration settings specific to the non-terminal's behavior or rule application within the parsing process. 
/// Together, these components define the operational and structural context for a non-terminal during synthesis. 
/// 
/// 
pub struct NonTerminal(pub String, pub Type, pub Vec<ProdRule>, pub Config);

impl NonTerminal {
    /// Parses a `Pair` into a `NonTerminal`. 

    pub fn parse(pair: Pair<'_, Rule>) -> Result<NonTerminal, Error> {
        let mut vec = pair.into_inner().collect_vec();
        let config = vec.last().unwrap().clone();
        let config = if config.as_rule() == Rule::config {
            vec.pop();
            Config::parse(config.clone())?
        } else {
            Config::new()
        };
        let [symbol, typ, prods]: [_; 3] = vec.try_into().unwrap();
        let prods: Vec<_> = prods.into_inner().map(|x| ProdRule::parse(x)).try_collect()?;
        Ok(NonTerminal(symbol.as_str().into(), Type::parse(typ)?, prods, config))
    }
}

#[derive(DebugCustom, Clone)]
#[debug(fmt = "{:?} [{:?}]", "self.inner", "self.config")]
/// A struct that represents a grammar configuration for the synthesis problem. 
/// 
/// It contains three fields: `start`, `inner`, and `config`. 
/// The `start` field is a `String` representing the initial non-terminal symbol of the grammar. 
/// The `inner` field is a vector of `NonTerminal`, detailing the additional non-terminals involved in the synthesis process. 
/// The `config` field is an instance of `Config`, which provides specific settings or parameters that modify or define the grammar's behavior within the synthesis context. 
/// Together, these fields collectively describe the essential components and settings required for configuring the grammar used in string synthesis tasks. 
/// 
/// 
pub struct Cfg {
    pub start: String,
    pub inner: Vec<NonTerminal>, 
    pub config: Config
}

impl Cfg {
    /// Parses a `Pair` using a context-free grammar (Cfg) representation and returns a `Cfg`. 
    /// 
    /// The function takes a `Pair` object that adheres to an outlined grammatical rule (`Rule`), transforming it into a collection of `NonTerminal` elements while also handling configuration through optional `Config` parsing. 
    /// 
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, Error> {
        let mut cfgvec = pair.into_inner().collect_vec();
        let config = cfgvec.last().unwrap().clone();
        let config = if config.as_rule() == Rule::config {
            cfgvec.pop();
            Config::parse(config.clone())?
        } else {
            Config::new()
        };
        let mut cfgiter = cfgvec.into_iter().peekable();
        let start = NonTerminal::parse(cfgiter.peek().unwrap().clone())?;
        let start = if let [ProdRule::Var(s, _)] =  start.2.as_slice() { cfgiter.next(); s } else { &start.0 };
        let start = start.clone();
        let mut inner: Vec<_> = cfgiter.map(|x| NonTerminal::parse(x)).try_collect()?;
        let mut cfg = Cfg{start, inner, config};
        cfg.reset_start();
        Ok(cfg)
    }
    /// This function resets the position of the start non-terminal in the control flow graph. 

    pub fn reset_start(&mut self) {
        let start_index = self.inner.iter().position(|x| x.0 == self.start).unwrap();
        let start_nt = self.inner.remove(start_index);
        self.inner.insert(0, start_nt);
    }
    /// Retrieves the name of the non-terminal by type. 
    /// 

    pub fn get_nt_by_type(&self, ty: &Type) -> String {
        self.inner.iter().find_map(|x| (x.1 == *ty).then_some(x.0.clone())).unwrap()
    }
    // pub fn sort(&mut self) {
    //     let mut sort = topological_sort::TopologicalSort::<NonTerminal>::new();
    //     for nt in self.inner.iter() {
    //         for rule in nt.2.iter() {
    //             if let ProdRule::Var(name, _) = rule {
    //                 if let Some(r) = self.inner.iter().find(|a| &a.0 == name) {
    //                     sort.add_dependency(*r, *nt);
    //                 }
    //             }
    //         }
    //     }
    //     let mut v = Vec::new();
    //     loop {
    //         let mut a = sort.pop_all();
    //         if a.is_empty() { break; }
    //         v.append(&mut a);
    //     }
    //     self.inner = v;
    // }
}

#[derive(Debug, Display, Clone)]
#[display(fmt = "{} ({}) {:?}", "self.name", r#"self.args.iter().map(|(s, t)| format!("({} {:?})", s, t)).collect_vec().join(" ")"#, "self.rettype")]
/// A struct that represents a function signature. 
/// 
/// This structure stores a function's name alongside its argument list and return type. 
/// The `name` field holds the function's name as a string. 
/// The `args` field is a vector of tuples, each containing a string representing an argument's name and a `Type` specifying the argument's type. 
/// The `rettype` field indicates the return type of the function. 
/// This struct provides a way to comprehensively define and store the components of a function's signature, facilitating operations that involve introspection or manipulation of function definitions within the context of string synthesis problems.
/// 
pub struct FunSig {
    pub name: String,
    pub args: Vec<(String, Type)>,
    pub rettype: Type,
}

impl FunSig {
    /// Returns the index of a named argument within the function signature's argument list. 

    pub fn index(&self, argname: &str) -> Option<usize> {
        self.args.iter().position(|x| x.0 == argname)
    }
}

#[derive(Debug, Clone)]
/// A struct that encapsulates a synthesis function's core attributes. 
/// 
/// It contains the `sig`, which holds the function's signature, defining the input and output types and any constraints applicable to the function. 
/// The `cfg` field represents the configuration grammar, likely detailing specific rules or patterns that govern how the function operates or is derived during synthesis. 
/// The `subproblem` field is a boolean flag indicating whether this function represents a subproblem within a larger synthesis task, potentially distinguishing it from primary synthesis functions or marking it for special processing.
/// 
pub struct SynthFun {
    pub sig: FunSig,
    pub cfg: Cfg,
    pub subproblem: bool
}

impl SynthFun {
    /// Parses a `synthfun` rule from a given input and constructs a `SynthFun` instance. 

    pub fn parse(synthfun: Pair<'_, Rule>) -> Result<Self, Error> {
        let subproblem = synthfun.as_rule() == Rule::synthsubproblem;
        let [name, arglist, typ, cfg]: [_; 4] = synthfun.into_inner().collect_vec().try_into().unwrap();
        let args: Vec<(String, Type)> = arglist
            .into_inner()
            .map(|x| {
                let [name, typ]: [_; 2] = x.into_inner().collect_vec().try_into().unwrap();
                Ok((name.as_str().to_owned(), Type::parse(typ)?))
            })
            .try_collect()?;
        let rettype = Type::parse(typ)?;
        let cfg = Cfg::parse(cfg)?;
        Ok(Self{sig: FunSig{name: name.as_str().into(), args, rettype}, cfg, subproblem})
    }
    /// Lookup for a non-terminal within the synthesis function's configuration. 

    pub fn lookup_nt(&self, nt: &str) -> Option<usize> {
        self.cfg.inner.iter().find_position(|x| x.0.as_str() == nt).map(|x| x.0)
    }
    /// Searches for an argument name within the function signature and returns its index if found. 
    pub fn lookup_arg(&self, arg: &str) -> Option<usize> {
        self.sig.args.iter().find_position(|x| x.0.as_str() == arg).map(|x| x.0)
    }
}




impl Type {
    /// Parses a type from a given token pair. 
    /// 
    /// This function starts by processing the provided `Pair` to extract a contained symbol. 
    /// It then matches this symbol against several predefined strings representing basic types, converting it into the corresponding `Type` variant. 
    /// These types include `Int`, `String`, `Bool`, and `Float`. 
    /// If the string representation of the pair contains "List", the function attempts to convert the basic type into a list type using `to_list()`, returning an error if this is unsupported. 
    /// The function returns the parsed `Type` or an error if an unknown type is encountered.
    /// 
    pub fn parse(pair: Pair<'_, Rule>) -> Result<Self, Error> {
        let [symbol]: [_; 1] = pair.clone().into_inner().collect_vec().try_into().unwrap();
        if pair.as_str().contains("BitVec") {
            let b = symbol.as_str().parse::<usize>().map_err(|_| new_custom_error_span("Can not parse BitVec".into(), pair.as_span()))?;
            return Ok(Self::BitVector(b));
        }
        let basic = match symbol.as_str() {
            "Int" => Self::Int,
            "String" => Self::Str,
            "Bool" => Self::Bool,
            "Float" => Self::Float,
            _ => panic!("Unknown Type {}", symbol.as_str()),
        };
        if pair.as_str().contains("List") {
            basic.to_list().ok_or(new_custom_error_span("Unsupported list type".into(), pair.as_span()))
        } else {
            Ok(basic)
        }
    }
}
#[derive(Debug)]
/// A struct representing a synthesis problem to be solved. 
/// This structure contains four fields: `logic`, a string specifying the logic or domain in which the synthesis problem is defined; `synthfuns`, a vector of `SynthFun` instances representing the functions to be synthesized as part of solving the problem; `problem_index`, a usize value denoting the particular index or identifier of this problem within a broader set of problems; and `examples`, an `IOExamples` instance that holds input-output exemplars relevant to the synthesis task, to ground the problem solution in practical demonstrations of expected behavior.
pub struct PBEProblem {
    pub logic: String,
    pub synthfuns: Vec<SynthFun>,
    pub problem_index: usize,
    pub examples: IOExamples,
}

impl PBEProblem {
    /// Provides access to a specific `SynthFun` based on the current `problem_index`. 
    /// 
    /// This function retrieves a reference to the `SynthFun` within the `synthfuns` vector of the `PBEProblem` instance. 
    /// The `problem_index` specifies which `SynthFun` to access, allowing direct retrieval of the active synthesis function's details, such as its signature and configuration, from the list of available functions.
    /// 
    pub fn synthfun(&self) -> &SynthFun {
        &self.synthfuns[self.problem_index]
    } 
    
    /// Parses a string input to create an instance of `PBEProblem`. 
    /// 
    /// This method uses the `ProblemParser` to initially parse the input string according to predefined grammar rules, extracting relevant components for logic, synthesis functions, examples, and checks. 
    /// It specifically targets obtaining the logic definition, synthesizing problem configurations, and IO examples used in problem-solving. 
    /// The method processes these components to extract the inner details of logic and synthesis, where synthesis functions are parsed individually. 
    /// A verification step ensures that only one main synthesis function (`synth-fun`) is designated as the primary problem by filtering out those marked as subproblems. 
    /// The synthesis examples are parsed to ensure they match the signature of the main synthesis function. 
    /// It constructs and returns a `PBEProblem` comprising the logic, a vector of synthesis functions, the index of the main problem, and the parsed examples. 
    /// The method will fail if the input does not conform to expected structures or logic, returning an error.
    /// 
    pub fn parse(input: &str) -> Result<PBEProblem, Error> {
        let [file]: [_; 1] = ProblemParser::parse(Rule::file, input)?.collect_vec().try_into().unwrap();
        let [_, logic, synthproblem, examples, checksynth]: [_; 5] = file.into_inner().collect_vec().try_into().unwrap();
        let [logic]: [_; 1] = logic.into_inner().collect_vec().try_into().unwrap();
        let synthfuns: Vec<_> = synthproblem.into_inner().enumerate().map(|(i, pair)| SynthFun::parse(pair)).collect::<Result<Vec<_>, _>>()?;
        let vec = synthfuns.iter().enumerate().filter(|x| !x.1.subproblem).map(|i|i.0).collect_vec();
        let problem_index = if let [a] = vec.as_slice() {*a} else { panic!("There should be only one synth-fun."); };
        let examples = IOExamples::parse(examples, &synthfuns[problem_index].sig, true)?;

        Ok(PBEProblem {
            logic: logic.as_str().to_owned(),
            synthfuns,
            problem_index,
            examples,
        })
    }
}

#[derive(pest_derive::Parser)]
#[grammar = "src/parser/problem.pest"]
/// A unit struct that serves as a parser for synthesis problems. 
/// 
/// This struct, given the absence of any fields or methods, functions as a marker or indicator within the module, potentially signifying a namespace or a type dedicated to parsing tasks related to synthesis problems. 
/// It's expected to be expanded or referenced in functions or traits that implement its parsing functionality.
/// 
pub struct ProblemParser;

#[cfg(test)]
mod tests {
    use std::fs;

    use super::PBEProblem;

    #[test]
    fn parse_test() {
        let s = fs::read_to_string("test/test.sl").unwrap();
        let result = PBEProblem::parse(s.as_str());
        println!("{:?}", result.map(|x| x.synthfun().cfg.clone()));
    }
}
