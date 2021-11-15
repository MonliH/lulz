import sys
import os
import json
import subprocess
import matplotlib.pyplot as plt
import argparse
from tqdm import tqdm
from shutil import copyfile

import matplotlib.pyplot as plt

parser = argparse.ArgumentParser(
    description="Run and graph benchmarks between lulz, lci, and python."
)

parser.add_argument(
    "benchmark",
    metavar="BENCHMARK",
    type=str,
    nargs=1,
    help="the benchmark in ./perfs/ to run",
)

parser.add_argument(
    "-g",
    "--graph-only",
    dest="no_run",
    action="store_true",
    help="only generate the graph based on previous benchmark data in ./_perfs/",
)

opts = parser.parse_args()

name = opts.benchmark[0]
folder_name = os.path.join("perfs", name)

if not os.path.isdir(folder_name):
    print(f"invalid program to bench `{name}`")
    exit(1)

perfs_dir = "_perfs"

if not os.path.isdir(perfs_dir):
    os.makedirs(perfs_dir)

json_name = os.path.join(perfs_dir, "data.json")
perf_backup = os.path.join(perfs_dir, f"{name}_results.json")

exts = [".lol", ".lci.lol", ".py"]
commands = ["./target/release/lulz", "lci", "python"]
filenames = [os.path.join(folder_name, name + ext) for ext in exts]
temp_filenames = [filename + ".temp" for filename in filenames]

values = json.load(open(os.path.join(folder_name, "values.json"), "r"))
ns = values["<<N>>"]

fig, ax = plt.subplots()

if opts.no_run:
    data = json.load(open(os.path.join(perf_backup), "r"))
else:
    data = {command: [] for command in commands}

    for n in tqdm(ns):
        for (file, temp) in zip(filenames, temp_filenames):
            # copy file to temp location, to overwrite the <<N>> values
            with open(file, "r") as original:
                old_string = original.read()
            with open(temp, "w+") as temp_file:
                temp_file.write(old_string.replace("<<N>>", str(n)))

        process = subprocess.Popen(
            [
                "hyperfine",
                "-s",
                "full",
                "--export-json",
                json_name,
            ]
            + [
                f'{command} "{filename}"'
                for (command, filename) in zip(commands, temp_filenames)
            ],
            shell=False,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
        )

        # write the piped stdout from hyperfine through tqdm
        for line in process.stdout:
            tqdm.write(line.decode("utf-8"), end="")

        tqdm.write("\n------\n")

        # store useful information
        results = json.load(open(json_name, "r"))
        for program in results["results"]:
            command = program["command"].split(" ")[0]
            data[command].append((program["mean"], program["stddev"]))

    # clean up generated files
    for temp in temp_filenames:
        os.remove(temp)

    os.remove(json_name)

    json.dump(data, open(perf_backup, "w"))

# plot the data and write to file
fig, ax = plt.subplots()

for (file, points) in data.items():
    ax.errorbar(
        ns,
        [point[0] for point in points],
        [point[1] for point in points],
        label=file.split("/")[-1],
        marker="*",
        capsize=3,
    )

ax.set_xlabel("input size")
ax.set_ylabel("time")
ax.legend()
fig.suptitle(f"{name} benchmark results", fontsize=18)
ax.set_title("lower is better (log scale)")
ax.set_yscale("log")

plt.savefig(f"{name}.png")
