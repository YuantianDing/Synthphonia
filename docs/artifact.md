
# Artifact for Synthphonia (DryadSynth String Solver)

## System Requirement

| Requirement  | CPUs | Memory | Storage |
| :--:         | :--: | :--:   | :--:    |
| Minimum      | 8    | 8 GB   | 32 GB   |
| Recommended  | 16   | 16 GB  | 32 GB   |
| Reproduction | 32   | 128 GB | 64 GB   |

## Get the Artifact

The easiest way to build this artifact is to use our alpine VM images at [Release Page](https://github.com/YuantianDing/Synthphonia/releases). Simply load this image to VM manager like VirtualBox, and select the desired computational resouces to run this artifact. 

The VM will open an SSH port at 2222. You can simply connect to this port and run `~/artifact/scripts/alpine-setup.sh` to build all the solvers. The process will take nearly 10 minute on a 16 cpus virtual machine. 

You can also download all `artifact.zip` without the VM image at our [Release Page](https://github.com/YuantianDing/Synthphonia/releases), here we scratch the steps to build the artifact. You can also refer to `~/artifact/scripts/alpine-setup.sh` for detailed commands.

* Duet: `solvers/duet/build` script shows how to install duet. You need to install OPAM along with the following packages: `z3.4.8.9`, `containers`, `containers-data`, `batteries`, `ocamlgraph`, `sexplib`.
    Make sure to use the corresponding version of those package supported by the ocaml compiler. Note that `python2` is required to build `z3.4.8.9`.
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
