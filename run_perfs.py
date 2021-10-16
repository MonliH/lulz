import sys
import os
from glob import iglob
import subprocess
import build_api
import timeit
import json

if len(sys.argv) == 2:
    filename = f"./perfs/{sys.argv[1]}.lol"
    if sys.argv[1] == "all":
        filename = f"./perfs/*.lol"
else:
    print("usage: python run_perfs <file> [repetitions]")
    print("       file to benchmark, accessed like `./perfs/<file>.lol`.")
    print("       using `all` as the filename selects all")
    exit(1)

file_list = (f for f in iglob(filename) if os.path.isfile(f))
if not file_list:
    print(f"invalid program to bench `{sys.argv[1]}`")
    exit(1)
for file in file_list:
    print(f"timing {file}", end="... ")
    with open(file, "r") as f:
        header = "{" + f.readline().split("{")[1]
    header = json.loads(header)
    sys.stdout.flush()
    t = timeit.Timer(
        lambda: subprocess.call(
            ["./target/release/lulz", file], stdout=subprocess.DEVNULL
        )
    )
    trials = int(header["trials"])
    reps = int(header["reps"])
    r = t.repeat(trials, reps)
    print(f"done, best of {trials}, run {reps} times each: {min(r)/reps}")
