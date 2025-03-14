
# Artifact for Synthphonia (DryadSynth String Solver)

## Get the Artifact

The easiest way to build this artifact is to use our alpine VM images at [Release Page](https://github.com/YuantianDing/Synthphonia/releases). Simply load this image to VM manager like VirtualBox and `~/artifact/scripts/alpine-setup` does the work.

You can also download all `artifact.zip` without the VM image at our [Release Page](https://github.com/YuantianDing/Synthphonia/releases), here we scratch the steps to build the artifact. You can also refer to `~/artifact/scripts/alpine-setup` for detailed commands.

* Duet: `solvers/duet/build` script shows how to install duet. You need to install OPAM along with the following packages: `z3.4.8.9`, `containers`, `containers-data`, `batteries`, `ocamlgraph`, `sexplib`. Note that you need to use the corresponding version of those package supported by the ocaml compiler. Note that `python2` is required to build `z3.4.8.9`.
* Probe: You need to have JVM, `sbt`, and `cvc4` installed, and simply run `sbt assembly`.
* FlashFill++: You need [.NET 6.0 SDK](https://dotnet.microsoft.com/en-us/download/dotnet/6.0) installed, and simply run `dotnet build`.
* Synthphonia: See [YuantianDing/Synthphonia](https://github.com/YuantianDing/Synthphonia).

The `requirements.txt` inside `text_utils` must be installed to run the `test.py` script.

## The Files of the Artifact

```py
artifact/
    benchmarks/      # Benchmarks of 3 Categories, in different formats.
        duet/
        prose/
        hardbench/
    scripts/         # Scripts to build all the solvers
    solvers/
        duet/
        probe/
        prose/
        synthphonia/
    test_utils/      # A Python module used by `test.py`
    test.py          # Script to run the tests.
    README.md
```

## Run the tests

`test.py` provides a command to run/export/view the primary results from the paper:

```md
Usage: test.py [OPTIONS] COMMAND [ARGS]...

Options:
  --help  Show this message and exit.

Commands:
  csv   Export results of a benchmark suite (duet, hardbench, prose) to csv
  draw  Draw figures in `figures` directory (Need all results collected)
  run   Run all benchmarks in a benchmark suite (all, duet, hardbench, prose)
  xlsx  Export results to xlsx
```

## Experiment More

See [YuantianDing/Synthphonia](https://github.com/YuantianDing/Synthphonia) to play with Synthphonia!
