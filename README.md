
# Synthphonia: DryadSynth Solver for Strings

Synthphonia is the name for the internal algorithm used by [DryadSynth](https://github.com/purdue-cap/DryadSynth) string solver.

## Build

Simply build with `cargo build --release`.

## Usage

```js
Usage: synthphonia [OPTIONS] <PATH>

Arguments:
  <PATH>  Path to the input file: enriched sygus-if (.sl) for synthesis or smt2 (.smt2) to check the result

Options:
  -v, --verbose...
          Log level
  -c, --cfg <CFG>
          Path to the context-free grammar configuration (enriched sygus-if)
  -j, --thread <THREAD>
          Number of threads [default: 4]
      --no-ite
          No ITE Mode: Generate results without `ite` operator
      --ite-limit-rate <ITE_LIMIT_RATE>
          Set the rate limit of ITE (in milliseconds), i.e.,
          how much time (without new solutions) does it take
          for the `ite_limit` to increment by one [default: 4000]
      --no-deduction
          Disable deduction, i.e., Enumeration + ACS
      --with-all-example-thread
          Enable all-example thread (Using one thread for all-example thread)
      --extract-constants
          Enable constant extraction
  -d, --debug
          Debug Mode (More assertions)
      --showex
          Show examples (debugging)
      --sig
          Show Signature (Just Print the signature without solving)
  -h, --help
          Print help
```


## Enriched Sygus-If

Synthphonia uses an enriched grammar of sygus-if to specify the grammar by algorithm. `test/` directory lists several examples of such a grammar, For example:

```js
(synth-fun f ((name String)) String
    (
      (Start String (ntString))
      (ntString String ("" name
            (str.++ ntString ntString) 
            (str.head ntString ntInt #cost:4) // The `cost` hints the weight for each operator
            (str.tail ntString ntInt #cost:4)

            (list.at ntList ntInt) 
            (str.join ntList ntString) 
            (int.to.str ntInt #cost:2)

            // `retain**` is a function to only keep a specific
            //       unicode category (https://www.compart.com/en/unicode/category)
            //       of characters in a string.
            // 
            // e.g. `retainLl` means to only keep all unicode lower-case (`l`) letters (`L`).
            (str.retainLl ntString #cost:4)
            (str.retainLc ntString #cost:4)
            (str.retainL ntString #cost:4)
            (str.retainN ntString #cost:4)
            (str.retainLN ntString #cost:4)
            (str.uppercase ntString #cost:4)
            (str.lowercase ntString #cost:4)

            (ite ntBool ntString ntString)
      ) )
      (ntInt Int (-1 1 2 3 4 5
            (+ ntInt ntInt #cost:4)
            (int.neg ntInt)
            (list.len ntString)
            (str.count ntString ntString #cost:2)
            (str.to.int ntString #cost:2)
      ))
      (ntBool Bool (
            (int.is0 ntInt)
            (int.is+ ntInt)
            (int.isN ntInt) // is natural number, i.e., >=0
      ))
      (ntList (List String) (
            (str.split ntString ntString)
            (list.map ntList)
      ))
))
```