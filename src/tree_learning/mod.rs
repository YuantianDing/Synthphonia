use std::{cell::RefCell, borrow::{Borrow, BorrowMut}};

use bumpalo::Bump;


pub mod bits;

use bits::BoxSliceExt;
pub use bits::Bits;

use crate::{debg, debg2, expr::Expr};

/// An enum representing subproblems within a decision tree learning process for string synthesis. 
pub enum SubProblem<'a> {
    Unsolved(Bits, f32),
    Accept(usize),
    Ite{ expr: usize, entropy: f32, t: SubProb<'a>, f: SubProb<'a> }
}

impl<'a> SubProblem<'a> {
    #[inline]
    /// Adds subproblems for exploration in the decision tree. 
    pub fn add_subproblems(&self, subproblem: &mut Vec<(SubProb<'a>, bool)>) {
        if let SubProblem::Ite { expr, entropy, t, f } = self {
            subproblem.push((f, true));
            subproblem.push((t, true));
        }
    }
}

pub type SubProb<'a> = &'a RefCell<SubProblem<'a>>;

/// A struct encapsulating the state and parameters for a decision tree learning process in string synthesis. 
pub struct TreeLearning<'a, 'b> {
    pub size: usize,
    root: SubProb<'a>,
    pub subproblems: Vec<SubProb<'a>>,
    limit: usize,
    pub conditions: &'b [(&'static Expr, Bits)],
    pub options: Vec<(&'static Expr, Bits)>,
    pub bump: &'a Bump,
    pub solved: bool,
}

/// An enum that captures the outcomes of decision-making processes for solving subproblems in decision trees. 
/// 
/// This enum comprises three variants: `Accept`, `Ite`, and `Failed`. 
/// 
/// 
/// The `Accept` variant represents an outcome where a subproblem is successfully solved, containing a `usize` value typically indicating an index or identifier of the solution. 
/// The `Ite` variant indicates a decision to use a conditional branch, often an 'if-then-else' construct, and includes a `usize` for identification, a `f32` for weight or probability, and two tuples of `(Bits, f32)` representing the branching conditions and associated probabilities. 
/// The `Failed` variant signifies that a subproblem could not be resolved under the current conditions, indicating a failure in the decision-making process.
pub enum SelectResult {
    Accept(usize),
    Ite(usize, f32, (Bits, f32), (Bits, f32)),
    Failed,
}


impl<'a, 'b> TreeLearning<'a, 'b> {
    // pub fn split_infomation(bits: Bits) -> f32 {

    // }
    /// Creates a new instance with specified parameters including size, conditions, options, memory allocator, and limit. 
    pub fn new_in(size: usize, conditions: &'b [(&'static Expr, Bits)], options: Vec<(&'static Expr, Bits)>, bump: &'a Bump, limit: usize) -> Self {
        let mut this = Self {
            size,
            root: bump.alloc(RefCell::new(SubProblem::Unsolved(bits::boxed_ones(size), 0.0))),
            subproblems: Vec::new(),
            conditions,
            options,
            bump,
            solved: false,
            limit
        };
        let root_entro = this.entropy(& bits::boxed_ones(size));
        if let SubProblem::Unsolved(a, entropy) = &mut *this.root.borrow_mut() {
            *entropy = root_entro;
        }
        this.subproblems.push(this.root);
        this
    }

    #[inline]
    /// Calculates the entropy of a given set of bits within the context of the `TreeLearning` algorithm's options. 
    pub fn entropy(&self, bits: & Bits) -> f32 {
        
        let mut vec: Vec<_> = self.options.iter().enumerate().map(|(i, b)| {
            let mut res = b.1.clone();
            res.conjunction_assign(bits);
            (i, res.count_ones(), res)
        }).collect();
        vec.sort_by_key(|a| u32::MAX - a.1);

        let total = bits.count_ones();
        let mut rest = bits.clone();
        let mut rest_count = rest.count_ones();
        let mut res = 0.0;
        for (_, _, b) in vec {
            rest.difference_assign(&b);
            let count = rest_count - rest.count_ones();
            let p = count as f32 / total as f32;
            if p > 0.0 {
                res += - p * p.log2();
            }
            rest_count = rest.count_ones();
        }
        res
    }
    
    /// Calculates the conditional entropy of a given set of bits based on a specified condition bitset. 
    pub fn cond_entropy(&self, bits: &Bits, condition: &Bits) -> (f32, (Bits, f32), (Bits, f32)) {
        let total = bits.count_ones();
        let mut and_bits = bits.clone();
        and_bits.conjunction_assign(condition);
        let and_entro = self.entropy(&and_bits);
        let and_count = and_bits.count_ones();
        let mut diff_bits = bits.clone();
        diff_bits.difference_assign(condition);
        let diff_entro = self.entropy(&diff_bits);
        let diff_count = diff_bits.count_ones();
        if and_count == 0 || diff_count == 0 {
            (1e10, (and_bits, and_entro), (diff_bits, diff_entro))
        } else {
            (
                (and_entro * and_count as f32 + diff_entro * diff_count as f32) / total as f32,
                (and_bits, and_entro), (diff_bits, diff_entro)
            )
        }
    }
    
    #[inline]
    /// Determines the next action for an unsolved subproblem in the tree learning process. 
    pub fn select(&self, unsolved: &SubProblem<'a>) -> SelectResult {
        if let SubProblem::Unsolved(bits, entro) = unsolved {
            if *entro <= 0.0001 {
                if let Some((i, _)) = self.options.iter().enumerate().find(|(_, x)| bits.subset(&x.1) ) {
                    return SelectResult::Accept(i)
                }
            }
            let (i, (centro, tb, fb)) = self.conditions.iter().enumerate()
                .map(|(i, (e, cb))| {
                    let ce = self.cond_entropy(bits, cb);
                    (i, ce)
                })
                .min_by(|a, b| a.1.0.partial_cmp(&b.1.0).unwrap())
                .expect("At least have one condition.");
            if centro - 0.00001 < *entro {
                SelectResult::Ite(i, centro, tb, fb)
            } else {
                SelectResult::Failed
            }
        } else { panic!("last should be unsolved.") }
    }

    /// Executes the learning algorithm by iterating over the subproblems within the decision tree. 
    pub fn run(&mut self) -> bool {
        let mut counter = 1;
        while let Some(last) = self.subproblems.pop() {
            let sel = self.select(&last.borrow());
            match sel {
                SelectResult::Accept(i) => {
                    *last.borrow_mut() = SubProblem::Accept(i);
                }
                SelectResult::Ite(expr, entropy, t, f) => {
                    let tb = self.bump.alloc(SubProblem::Unsolved(t.0, t.1).into());
                    let fb = self.bump.alloc(SubProblem::Unsolved(f.0, f.1).into());
                    self.subproblems.push(fb);
                    self.subproblems.push(tb);
                    *last.borrow_mut() = SubProblem::Ite{ expr, entropy, t: tb, f: fb };
                    counter += 2;
                    if counter > self.limit { 
                        debg2!("{:?}", self);
                        return false;
                    }
                }
                SelectResult::Failed => {
                    debg2!("{:?}", self);
                    return false;
                }
            }
        }
        self.solved = true;
        debg2!("{:?}", self);
        true
    }

    /// Facilitates the recursive formatting of the decision tree contained within the `TreeLearning` structure for display purposes. 
    fn fmt_recursive(&self, f: &mut std::fmt::Formatter<'_>, node: SubProb<'a>, indent: &mut String) -> std::fmt::Result {
        match &*node.borrow() {
            SubProblem::Unsolved(bits, entropy) => 
                writeln!(f, "{indent}?? {} {:x?}", entropy, bits),
            SubProblem::Accept(i) => 
                writeln!(f, "{indent}{:?}", self.options[*i].0),
            SubProblem::Ite { expr, entropy, t: tb, f: fb } => {
                writeln!(f, "{indent}ite {:?} {:x?}", self.conditions[*expr].0, self.conditions[*expr].1)?;
                indent.push_str("  ");
                self.fmt_recursive(f, tb, indent)?;
                self.fmt_recursive(f, fb, indent)?;
                indent.pop(); indent.pop();
                Ok(())
            }
        }
    }
    /// Determines the size of the decision tree by recursively traversing through its nodes. 
    fn size_recursive(&self, node: SubProb<'a>) -> usize {
        match &*node.borrow() {
            SubProblem::Unsolved(bits, entropy) => 1,
            SubProblem::Accept(i) => 1,
            SubProblem::Ite { expr, entropy, t: tb, f: fb } => 1 + self.size_recursive(tb) + self.size_recursive(fb),
        }
    }
    /// Covers a decision tree recursively starting from a given node and determining the set of bits covered by the tree structure. 
    fn cover_recursive(&self, node: SubProb<'a>) -> Bits {
        match &*node.borrow() {
            SubProblem::Unsolved(bits, entropy) => bits.clone(),
            SubProblem::Accept(i) => self.options[*i].1.clone(),
            SubProblem::Ite { expr, entropy, t: tb, f: fb } => {
                let mut t = self.cover_recursive(tb);
                let mut f = self.cover_recursive(fb);
                let bits = self.conditions[*expr].1.clone();
                t.conjunction_assign(&bits);
                f.difference_assign(&bits);
                t.union_assign(&f);
                t
            }
        }
    }
    /// Returns the expression representation of a given node in the decision tree. 
    fn expr_recursizve(&self, node: SubProb<'a>) -> &'static Expr {
        match &*node.borrow() {
            SubProblem::Unsolved(bits, entropy) => panic!("Still subproblem remain."),
            SubProblem::Accept(i) => self.options[*i].0,
            SubProblem::Ite { expr, entropy, t: tb, f: fb } => {
                let t = self.expr_recursizve(tb);
                let f = self.expr_recursizve(fb);
                let cond = self.conditions[*expr].0;
                cond.ite(t, f)
            }
        }
    }
    /// Recursively traverses through a decision tree to collect bits from unsolved subproblems. 
    fn unsolved_recursive(&self, node: SubProb<'a>, result: &mut Vec<Box<[u128]>>) {
        match &*node.borrow() {
            SubProblem::Unsolved(bits, entropy) => {
                result.push(bits.clone());
            }
            SubProblem::Accept(i) => {}
            SubProblem::Ite { expr, entropy, t: tb, f: fb } => {
                self.unsolved_recursive(tb, result);
                self.unsolved_recursive(fb, result);
            }
        }
    }
    /// Returns a vector of boxed slices containing `u128` values that represent unsolved components of a decision tree. 
    fn unsolved(&self) -> Vec<Box<[u128]>> {
        let mut result = Vec::new();
        self.unsolved_recursive(self.root, &mut result);
        result
    }
    /// Returns the expression associated with the root of the decision tree. 
    /// This function utilizes a recursive approach by invoking `expr_recursizve` on the tree's root node to retrieve the expression efficiently, leveraging the recursive structure to navigate through potentially complex tree configurations within the `TreeLearning` context.
    pub fn expr(&self) -> &'static Expr {
        self.expr_recursizve(self.root)
    }
    
    /// Calculates the result size of a decision tree by recursively determining the size starting from the root node. 
    /// This implementation utilizes the `size_recursive` function on the `root` to compute the cumulative size of the tree structure, which includes all subproblems, branches, and accepted solutions present in the tree.
    pub fn result_size(&self) -> usize {
        self.size_recursive(self.root)
    }
}

impl<'a, 'b> std::fmt::Debug for TreeLearning<'a, 'b> {
    /// Formats the decision tree within the `TreeLearning` instance for display. 
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_recursive(f, self.root, &mut "".into())
    }
}

#[inline(always)]
pub fn tree_learning<'a, 'b>(options: Vec<(&'static Expr, Bits)>, conditions: &'b [(&'static Expr, Bits)], size: usize, bump: &'a Bump, limit: usize) -> TreeLearning<'a, 'b> {
    let mut tl = TreeLearning::new_in(size, conditions, options, bump, limit);
    tl.run();
    tl
}

