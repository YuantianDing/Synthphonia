use std::{iter, ops::Range};

use iset::IntervalMap;
use itertools::{Itertools};
use radix_trie::{Trie, TrieCommon};

use crate::closure;
use ahash::AHashMap as HashMap;
/// An enumeration representing a recursive interval tree that can contain either an inner node with associated intervals or a terminal leaf with a stored value. 
/// 
/// 
/// The inner node variant encapsulates an interval map associating ranges (identified by usize keys) to boxed nested trees, enabling hierarchical and ordered organization of intervals. 
/// In contrast, the leaf variant holds a direct value, serving as the base case in this nested tree structure.
pub enum NestedIntervalTree<T>{
    Node(IntervalMap<usize, Box<NestedIntervalTree<T>>>),
    Leaf(T)
}

impl<T: Clone> NestedIntervalTree<T> {
    /// Builds a nested interval tree by recursively processing an iterator of range iterators and embedding a terminal value when no more ranges remain. 
    /// 
    /// 
    /// Recursively processes the outer iterator such that if a current iterator exists, it iterates over its ranges and creates subtrees for each by cloning the remaining iterators and value; otherwise, it produces a leaf containing the provided value. 
    /// This design constructs a hierarchical structure where each node associates intervals with nested subtrees, culminating in leaves holding the synthesized value.
    pub fn build(mut ranges: impl Iterator<Item=impl Iterator<Item=Range<usize>>> + Clone, value: T) -> Self {
        if let Some(head) = ranges.next() {
            let mut maps = IntervalMap::new();
            for range in head {
                let inner = Self::build(ranges.clone(), value.clone());
                maps.insert(range, inner.into());
            }
            Self::Node(maps)
        } else {
            Self::Leaf(value)
        }
    }
    /// Inserts elements into a nested interval structure by recursively traversing an iterator over iterators of range indices to either update existing leaf values or create new branches with default values.
    /// 
    /// The function operates by taking an iterator of iterators of ranges, a function to update leaf values, and a default value. 
    /// For an internal node, it iterates over each range from the provided iterator: if a corresponding child exists, it recurses into that child with cloned iterators; if not, it creates a new branch using the default value. 
    /// When a leaf node is reached and there are no more ranges, it applies the update function to modify the leaf's contained value. 
    /// The function will panic if the provided range structure does not match the depth of the tree.
    pub fn insert_using_iter<'a: 'b, 'b>(&'a mut self, mut ranges: impl Iterator<Item=impl Iterator<Item=Range<usize>> + 'b> + Clone + 'b, update: &impl Fn(&mut T), default: T) {
        let head = ranges.next();
        match (self, head) {
            (NestedIntervalTree::Node(maps), Some(head)) => {
                for range in head {
                    if let Some(r) = maps.get_mut(range.clone()) {
                        r.insert_using_iter(ranges.clone(), update, default.clone());
                    } else {
                        maps.insert(range, Self::build(ranges.clone(), default.clone()).into());
                    }
                }
            }
            (NestedIntervalTree::Leaf(v), None) => {
                update(v);
            }
            _ => panic!("DeepIntervalTree have a different number of ranges indices."),
        }
    }
    /// Inserts a value at a nested tree leaf corresponding to a multi-dimensional sequence of interval ranges. 
    /// 
    /// 
    /// Uses a vector of interval range vectors to represent the nesting levels, delegating recursive insertion to a helper that traverses and builds intermediate nodes as needed, while using a no-op update function for branch nodes.
    pub fn insert_multiple<'a: 'b, 'b>(&'a mut self, ranges: &Vec<Vec<Range<usize>>>, value: T) {
        self.insert_using_iter(ranges.iter().map(|x| x.iter().cloned()), &|_| (), value)
    }
    /// Inserts a value into the nested interval tree using a provided slice of ranges. 
    /// This function transforms the slice of ranges into an iterator of one-item iterators and then delegates to the internal insertion procedure that supports multi-level range specifications.
    pub fn insert<'a: 'b, 'b>(&'a mut self, ranges: &[Range<usize>], value: T) {
        self.insert_using_iter(ranges.iter().map(|x| iter::once(x.clone())), &|_| (), value)
    }
}

impl<T> Default for NestedIntervalTree<T> {
    /// Returns a default instance by invoking the constructor for a new instance. 
    /// This implementation enables the use of default() to create an object with initial empty configuration without the need for manual initialization.
    fn default() -> Self {
        Self::new()
    }
}

impl<T> NestedIntervalTree<T> {
    /// Creates an empty nested interval tree by initializing it as a node containing an empty interval map. 
    /// This function serves as a constructor for the tree structure, providing a starting point for inserting ranges and associated values in later operations.
    pub fn new() -> Self {
        Self::Node(IntervalMap::new())
    }
    /// Retrieves a stored value by following a sequence of interval ranges across a nested data structure. 
    /// 
    /// This function accepts a slice of range indices that represent a traversal path through nested intervals. 
    /// It navigates the structure by, for non-empty range sequences, using the first range to determine which subtree to search and recursing with the remaining ranges, while returning the leaf’s value if there are no ranges left. 
    /// If the provided path does not match the structure, it results in a None.
    /// 
    pub fn get(&self, ranges: &[Range<usize>]) -> Option<&T> {
        match self {
            NestedIntervalTree::Node(maps) if !ranges.is_empty() => {
                let (head, tail) = (&ranges[0], &ranges[1..]);
                maps.get(head.clone()).and_then(|x| x.get(tail))
            }
            NestedIntervalTree::Leaf(v) if ranges.is_empty() => Some(v),
            _ => None,
        }
    }
    /// Returns an iterator over all values in the tree that are reachable via a chain of intervals where each enclosing interval is a superrange of the corresponding query range. 
    /// 
    /// This method recursively traverses a nested interval structure using an iterator of iterators of ranges; in internal nodes it filters subintervals that fully contain the query interval and descends recursively, while at a leaf node with no remaining query ranges it yields the stored value. 
    /// It panics if the depth of provided range sequences does not match the tree structure.
    /// 
    pub fn superrange_using_iter<'a: 'b, 'b>(&'a self, mut ranges: impl Iterator<Item=impl Iterator<Item=Range<usize>> + 'b> + Clone + 'b) -> Box<dyn Iterator<Item=&'b T> + 'b> {
        let head = ranges.next();
        match (self, head) {
            (NestedIntervalTree::Node(maps), Some(head)) => {
                let it = head.flat_map(move |head| {
                    maps.iter(head.clone())
                        .filter(move |(r, _)| head.start >= r.start && r.end >= head.end) 
                        .flat_map(closure![clone ranges; move |(_, t)| t.superrange_using_iter(ranges.clone())])
                });
                Box::new(it)
            }
            (NestedIntervalTree::Leaf(v), None) => Box::new(Some(v).into_iter()),
            _ => panic!("DeepIntervalTree have a different number of ranges indices."),
        }
    }
    /// Returns an iterator over subrange-matching values from a nested interval tree structure based on a sequence of range iterators. 
    /// 
    /// 
    /// Processes a cloned iterator yielding iterators over range indices, where at each node it filters for entries whose ranges are completely contained within the provided range, and recursively applies the same filtering on the resulting subtrees. 
    /// In the leaf case when no further range iterator is provided, it yields the leaf value, while differing iterator depths result in a runtime panic.
    pub fn subrange_using_iter<'a: 'b, 'b>(&'a self, mut ranges: impl Iterator<Item=impl Iterator<Item=Range<usize>> + 'b> + Clone + 'b) -> Box<dyn Iterator<Item=&'b T> + 'b> {
        let head = ranges.next();
        match (self, head) {
            (NestedIntervalTree::Node(maps), Some(head)) => {
                let it = head.flat_map(move |head| {
                    maps.iter(head.clone())
                        .filter(move |(r, _)| r.start >= head.start && head.end >= r.end) 
                        .flat_map(closure![clone ranges; move |(_, t)| t.subrange_using_iter(ranges.clone())])
                });
                Box::new(it)
            }
            (NestedIntervalTree::Leaf(v), None) => Box::new(Some(v).into_iter()),
            _ => panic!("DeepIntervalTree have a different number of ranges indices."),
        }
    }
    /// Returns an iterator over all values whose associated intervals contain the specified super ranges.
    /// 
    /// Delegates to a lower-level iterator implementation by mapping the provided vector of range vectors into an appropriate iterator form for traversing the nested interval structure, ultimately yielding references to values that satisfy the super-range condition.
    pub fn superrange_multiple<'a: 'b, 'b>(&'a self, ranges: &'b Vec<Vec<Range<usize>>>) -> Box<dyn Iterator<Item=&'b T> + 'b> {
        self.superrange_using_iter(ranges.iter().map(|x| x.iter().cloned()))
    }
    /// Returns an iterator over elements from nested intervals that are enclosed by the specified multiple subrange lists provided as a vector of vectors. 
    /// 
    /// 
    /// Invokes an internal subrange retrieval function by mapping the provided vector of range lists into an iterator of cloned ranges, thereby allowing the caller to iterate over all matching elements in the tree that fall within the prescribed nested subranges.
    pub fn subrange_multiple<'a: 'b, 'b>(&'a self, ranges: &'b Vec<Vec<Range<usize>>>) -> Box<dyn Iterator<Item=&'b T> + 'b> {
        self.subrange_using_iter(ranges.iter().map(|x| x.iter().cloned()))
    }
    /// Return an iterator yielding references to values whose associated intervals form a superrange of the provided sequence of index ranges. 
    /// This function converts a vector of ranges into the expected single-element iterators and delegates the lookup to the underlying superrange retrieval logic, thereby providing access to all elements that satisfy the defined superrange condition.
    pub fn superrange<'a: 'b, 'b>(&'a self, ranges: Vec<Range<usize>>) -> Box<dyn Iterator<Item=&'b T> + 'b> {
        self.superrange_using_iter(ranges.into_iter().map(std::iter::once))
    }
    /// Returns a boxed iterator over references to values matching the provided subrange specification. 
    /// 
    /// 
    /// Maps the given vector of ranges into the expected iterator form and delegates to an underlying iterator-based subrange retrieval method to extract matching elements from the nested structure.
    pub fn subrange<'a: 'b, 'b>(&'a self, ranges: Vec<Range<usize>>) -> Box<dyn Iterator<Item=&'b T> + 'b> {
        self.subrange_using_iter(ranges.into_iter().map(std::iter::once))
    }
}

/// This structure represents an encoder that maps unique keys to integer codes while storing associated values. 
/// It encapsulates a vector of key-value pairs alongside a hash map that records the corresponding code for each key.
/// 
/// The interface supports operations to insert new key-value associations, retrieve a code given a key, and decode keys and values based on their assigned codes, ensuring efficient bidirectional lookup between keys and integer representations.
pub struct Encoder<K: std::hash::Hash, V> {
    values: Vec<(K, V)>,
    codes: HashMap<K, u32>
}

impl<K: std::hash::Hash + Clone + std::cmp::Eq, V> Default for Encoder<K, V> {
    /// Returns a new instance with default settings by invoking the standard constructor. 
    /// This method provides the default initialization behavior by delegating its work to the dedicated creation function, ensuring consistent instantiation for the type.
    fn default() -> Self {
        Self::new()
    }
}

impl<K: std::hash::Hash + Clone + std::cmp::Eq, V> Encoder<K, V> {
    /// Creates a new encoder instance with no stored key-value pairs. 
    /// 
    /// This constructor initializes the internal storage to be empty, allowing subsequent insertions to assign unique numerical codes to keys while associating corresponding values. 
    /// 
    /// 
    pub fn new() -> Self {
        Encoder { values: Vec::new(), codes: HashMap::new() }
    }
    /// Inserts a new key-value pair into the encoder, assigning and returning a unique numeric code for the key. 
    /// 
    /// 
    /// Generates a code based on the current state of internal storage, clones the key, and stores the pair in parallel structures that maintain both an ordered list of values and a mapping from keys to their corresponding codes, enabling efficient retrieval and translation between keys and their numeric identifiers.
    pub fn insert(&mut self, k: K, v: V) -> u32 {
        let code = self.values.len() as u32;
        self.values.push((k.clone(), v));
        self.codes.insert(k, code);
        code
    }
    /// Retrieves a numeric code corresponding to a given key from the internal mapping. 
    /// 
    /// 
    /// Returns an optional value representing the unique encoding if the key exists, or None otherwise.
    pub fn encode(&self, t: &K) -> Option<u32> {
        self.codes.get(t).cloned()
    }
    /// Retrieves a reference to the key corresponding to the provided encoded numeric value. 
    /// 
    /// 
    /// Returns the key by interpreting the given integer as an index into an internal collection, enabling reverse mapping from a numeric code back to its associated key.
    pub fn decode(&self, t: u32) -> &K {
        &self.values[t as usize].0
    }
    /// Returns a reference to the value associated with the supplied numerical code from the internal storage. 
    /// This method converts the provided u32 code into an index and accesses the corresponding entry, retrieving the second element of the stored key–value pair.
    pub fn value(&self, t: u32) -> &V {
        &self.values[t as usize].1
    }
}

/// A container wrapping a collection of radix tries that map static string slices to lists of string slice arrays. 
/// This structure is designed to group multiple tries, enabling efficient organization and retrieval of sequences of static strings by leveraging trie-based lookups for operations such as prefix search and related string queries.
pub struct RadixTrieN(Vec<Trie<&'static str, Vec<&'static [&'static str]>>>);

impl RadixTrieN {
    /// Creates a new instance of the specialized trie collection with a specified number of inner trie elements.
    /// 
    /// Initializes the generator by taking a length parameter and generating a vector with that many default trie structures. 
    /// Each inner trie is created using its default constructor, and the resulting collection is encapsulated within the instance.
    pub fn new(len: usize) -> Self {
        Self( (0..len).map(|_| Trie::new()).collect_vec() )
    }
    /// Inserts a provided key composed of static string slices into a multi-layer trie structure. 
    /// This function iterates over the key’s components paired with corresponding trie elements and updates each trie by appending the key to an existing entry or inserting a new entry when none exists.
    pub fn insert(&mut self, key: &'static [&'static str]) {
        for (s, v) in key.iter().cloned().zip(self.0.iter_mut()) {
            if let Some(a) = v.get_mut(s) {
                a.push(key);
            } else {
                v.insert(s, vec![key]);
            }
        }
    }
    #[inline]
    /// Returns an iterator over candidate key slices that are recognized as superfixes relative to the provided key.
    /// 
    /// Determines the most significant component of the key based on length and queries the corresponding subtrie, then filters and flattens the resulting values to yield only those keys that satisfy the superfix property with respect to the original key.
    pub fn superfixes(&self, key: &'static [&'static str]) -> impl Iterator<Item=&'static [&'static str]> + '_ {
        let (i, _) = key.iter().cloned().enumerate().max_by_key(|(i, x)| x.len()).unwrap();
        self.0[i].subtrie(key[i]).map(|x| x.values().flat_map(|v| v.iter().cloned().filter(|x| is_prefix(key, x))) ).into_iter().flatten()
    }
    #[inline]
    /// Returns an iterator that produces all stored string sequences from the underlying trie that qualify as prefixes of the provided key sequence.
    /// 
    /// Operates by selecting the key element with the smallest length to determine the appropriate subtrie, then iterates over candidate values from that subtrie. 
    /// The iterator is filtered to include only those sequence elements that satisfy the prefix relationship with the full key as determined by a predicate.
    pub fn prefixes(&self, key: &'static [&'static str]) -> impl Iterator<Item=&'static [&'static str]> + '_ {
        let (i, _) = key.iter().cloned().enumerate().min_by_key(|(i, x)| x.len()).unwrap();
        PrefixIter{ trie: &self.0[i], key: Some(key[i])}.flat_map(|x|  x.iter().cloned().filter(|x| is_prefix(x, key)) )
    }
}

/// Checks whether each element in the first slice is a prefix of its corresponding element in the second slice. 
/// This function iterates over paired elements from both slices and returns true only if every element from the first slice is a starting substring of the corresponding element in the second slice, ensuring a complete prefix match across the pair of slices.
fn is_prefix(x: &[&str], k: &[&str]) -> bool {
    x.iter().cloned().zip(k.iter().cloned()).all(|(a,b)| b.starts_with(a))
}

/// A structure representing an iterator that traverses ancestors in a trie based on a specific key. 
/// It maintains a reference to a trie mapping static string keys to associated values and an optional key used to initiate and guide the iteration process.
/// 
/// This iterator retrieves entries from the trie by progressively reducing the key, enabling prefix-based lookup. 
/// It encapsulates the necessary state to iterate over trie elements whose keys are prefixes, offering a convenient abstraction for navigating hierarchical or nested string data.
pub struct PrefixIter<'a, T>{
    trie: &'a Trie<&'static str, T>,
    key: Option<&'static str>,
}

impl<'a, T> Iterator for PrefixIter<'a, T> {
    type Item=&'a T;

    /// Advances the iterator by retrieving the next element associated with the current prefix state. 
    /// This method queries the underlying trie for an ancestor of the current key and, if found, updates the internal key by trimming its last element or marks the iteration as complete when empty, finally returning the corresponding value.
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(key) = self.key {
            self.trie.get_ancestor(key).and_then(|x| {
                x.key().map(|k| {
                    if k.is_empty() {
                        self.key = None;    
                    } else {
                        self.key = Some(&k[0..k.len()-1]);
                    }
                    x.value().unwrap()
                })
            })
        } else { None }
    }
}

/// A structure encapsulating an interval-based mapping designed to index and query segments of expected string data. 
/// 
/// 
/// It maintains a reference to an array of constant strings along with a vector of interval maps that associate ranges of indices with collections of string slice arrays. 
/// This design facilitates efficient operations for matching, extracting, and navigating through substrings or superstrings based on predefined expectations.
pub struct IntervalTreeN {
    expected: &'static [&'static str],
    maps: Vec<IntervalMap<usize, Vec<&'static [&'static str]>>>
}

impl IntervalTreeN {
    /// Creates a new instance using the provided expected strings and initializes an internal collection of interval maps. 
    /// 
    /// 
    /// Constructs the instance by assigning the given expected strings and generating one interval map per expected element, ensuring a corresponding structure is available for each.
    pub fn new(expected: &'static [&'static str]) -> Self {
        Self { expected, maps: (0..expected.len()).map(|_| IntervalMap::new()).collect_vec()}
    }
    /// Inserts a key into the interval-based mapping structure by iterating over corresponding expected string slices and mutable interval maps. 
    /// 
    /// The method scans each component of the key against its paired expected string to identify all matching substrings through pattern matching. 
    /// For each match, it computes a range based on the match index and the length of the key component. 
    /// If an existing entry for the range is found, it appends the key; otherwise, it creates a new entry with the key.
    /// 
    /// 
    pub fn insert(&mut self, key: &'static [&'static str]) {
        for (k, (e, v)) in key.iter().cloned().zip(self.expected.iter().cloned().zip(self.maps.iter_mut())) {
            for (i, _) in e.match_indices(k) {
                let range = i..(i + k.len());
                if let Some(l) = v.get_mut(range.clone()) {
                    l.push(key);
                } else {
                    v.insert(range, vec![key]);
                }
            }
        }
    }
    /// Inserts the first occurrence of each element of the provided key into the interval tree by mapping the range of its first match in the corresponding expected string. 
    /// 
    /// 
    /// Iterates over each element of the key array alongside the expected strings and mutable maps. 
    /// For each pair, it identifies the first matching index in the expected string and constructs a range covering that occurrence. 
    /// It then updates the map at that range by appending the key if the range already exists or inserting a new entry if it does not.
    pub fn insert_first_occur(&mut self, key: &'static [&'static str]) {
        for (k, (e, v)) in key.iter().cloned().zip(self.expected.iter().cloned().zip(self.maps.iter_mut())) {
            if let Some((i, _)) = e.match_indices(k).next() {
                let range = i..(i + k.len());
                if let Some(l) = v.get_mut(range.clone()) {
                    l.push(key);
                } else {
                    v.insert(range, vec![key]);
                }
            }
        }
    }
    #[inline]
    /// Returns an iterator over stored string sequences that are valid superstrings of the provided key. 
    /// 
    /// 
    /// Selects a dimension based on the longest string in the key and then retrieves intervals from the corresponding expected string. 
    /// These intervals are used to access candidate sequences, which are filtered to retain only those sequences that have the supplied key as a substring, yielding an iterator over matching sequences.
    pub fn superstrings(&self, key: &'static [&'static str]) -> impl Iterator<Item=&'static [&'static str]> + '_ {
        let (i, _) = key.iter().cloned().enumerate().max_by_key(|(_, x)| x.len()).unwrap();
        self.expected[i].match_indices(key[i]).map(move |(k, _)| k..(k+key[i].len())).flat_map(move |range| {
            self.maps[i].iter(range).flat_map(move |(_, v)| {
                v.iter().cloned().filter(|x| is_substring(key, x))
            })
        })
    }
    #[inline]
    /// Returns an iterator over elements from the interval tree whose stored string slices satisfy a substring inclusion condition with respect to the provided key. 
    /// 
    /// 
    /// Selects the key element with the minimum length to guide the search for matching intervals. 
    /// Then, for each occurrence of that element within the expected string, it gathers a range corresponding to that match and scans the associated interval map entries. 
    /// Finally, it filters and returns only those entries whose string slices fulfill the substring checking predicate relative to the entire key.
    pub fn substrings(&self, key: &'static [&'static str]) -> impl Iterator<Item=&'static [&'static str]> + '_ {
        let (i, _) = key.iter().cloned().enumerate().min_by_key(|(_, x)| x.len()).unwrap();
        self.expected[i].match_indices(key[i]).map(move |(k, _)| k..(k+key[i].len())).flat_map(move |range| {
            self.maps[i].iter(range).flat_map(move |(_, v)| {
                v.iter().cloned().filter(|x| is_substring(x, key))
            })
        })
    }
}

/// Determines if each string in one slice occurs as a substring within the corresponding string in another slice. 
/// 
/// 
/// Compares elements from two slices pairwise, returning true only if every element from the first slice is contained within its matching element in the second slice. 
/// The function uses an iterator to zip through both slices and applies the substring check to each pair, ensuring that the condition holds across all compared pairs.
fn is_substring(x: &[&str], k: &[&str]) -> bool {
    x.iter().cloned().zip(k.iter().cloned()).all(|(a,b)| b.contains(a))
}


#[cfg(test)]
mod test {
    use itertools::Itertools;
    use radix_trie::Trie;

    use crate::galloc::AllocForAny;

    use super::RadixTrieN;

    #[test]
    fn test() {
        let mut trie = RadixTrieN::new(2);
        trie.insert(["", "a"].galloc());
        trie.insert(["al", ""].galloc());
        trie.insert(["alpha", "alp"].galloc());
        trie.insert(["", "alp"].galloc());
        trie.insert(["", "alpha"].galloc());
        trie.insert(["", "alphb"].galloc());
        trie.insert(["alpha", "alpha"].galloc());
        
        for k in trie.prefixes(["alpha", "alpha"].galloc()) {
            println!("{:?}", k);
        }

    }
}
