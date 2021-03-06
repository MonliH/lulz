import os
import json
import subprocess
import matplotlib.pyplot as plt
import argparse
import numpy as np
from glob import glob
from tqdm import tqdm

parser = argparse.ArgumentParser(
    description="Run and graph benchmarks between lulz, lci, and python."
)

subparsers = parser.add_subparsers(dest="command_type")

graph = subparsers.add_parser(
    "graph",
    help="only generate the graph based on previous benchmark data in ./_perfs/",
)
graph.add_argument(
    "benchmark",
    metavar="<benchmark>",
    type=str,
    nargs=1,
    help="the benchmark in ./perfs/ to run",
)

run = subparsers.add_parser(
    "run",
    help="run the same benchmark (in ./perfs/) multiple times, with \
    different input sizes, and composite into a line graph",
)
run.add_argument(
    "benchmark",
    metavar="<benchmark>",
    type=str,
    nargs=1,
    help="the benchmark in ./perfs/ to run",
)

runall = subparsers.add_parser(
    "runall",
    help="run all benchmarks in ./perfs/, and composite into a bar graph",
)

opts = parser.parse_args()

exts = [".lol", ".py", ".lci.lol",]
commands = ["./target/release/lulz", "python", "lci"]

perfs_dir = "_perfs"

if not os.path.isdir(perfs_dir):
    os.makedirs(perfs_dir)


def get_filenames_from_name(name, folder_name):
    filenames = [os.path.join(folder_name, name + ext) for ext in exts]
    temp_filenames = [filename + ".temp" for filename in filenames]

    values = json.load(open(os.path.join(folder_name, "values.json"), "r"))
    ns = values["<<N>>"]

    return (filenames, temp_filenames, ns)


json_name = os.path.join(perfs_dir, "data.json")


def get_results(filenames, temp_filenames, n):
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

    # store useful information
    results = json.load(open(json_name, "r"))

    return results


if opts.command_type == "runall":
    bar_width = 0.25
    fig, ax = plt.subplots()
    data = {command: [] for command in commands}
    benches = []
    for value_filename in glob("./perfs/*/values.json"):
        value_json = json.load(open(value_filename, "r"))
        folder_name = os.path.dirname(value_filename)
        name = os.path.basename(folder_name)
        (filenames, temp_filenames, ns) = get_filenames_from_name(name, folder_name)
        results = get_results(filenames, temp_filenames, value_json["one"])
        benches.append(name)
        for program in results["results"]:
            command = program["command"].split(" ")[0]
            data[command].append((name, program["mean"], program["stddev"]))
    current_xs = np.arange(len(benches))
    for (name, series) in data.items():
        ax.bar(
            current_xs,
            [point[1] for point in series],
            width=bar_width,
            label=name.split("/")[-1],
        )
        current_xs = [x + bar_width for x in current_xs]

    ax.set_xlabel("benchmark", fontweight="bold")
    ax.set_ylabel("time (s)", fontweight="bold")
    plt.xticks([r + bar_width for r in range(len(benches))], benches)
    ax.legend()
    fig.suptitle(f"benchmark results", fontsize=18)
    ax.set_title("lower is better")
    plt.savefig(os.path.join(perfs_dir, f"benches.png"), dpi=300, bbox_inches="tight", pad_inches=0.2)
else:
    name = opts.benchmark[0]
    folder_name = os.path.join("perfs", name)

    if not os.path.isdir(folder_name):
        print(f"invalid program to bench `{name}`")
        exit(1)

    perf_backup = os.path.join(perfs_dir, f"{name}_results.json")

    (filenames, temp_filenames, ns) = get_filenames_from_name(name, folder_name)

    fig, ax = plt.subplots()

    def regen_single_graph(ns, name, data):
        # plot the data and write to file
        fig, ax = plt.subplots()

        for (file, points) in data.items():
            ax.errorbar(
                ns,
                [point[0] for point in points],
                [point[1] for point in points],
                label=file.split("/")[-1],
                marker=".",
                capsize=3,
            )

        ax.set_xlabel("input size")
        ax.set_ylabel("time (s)")
        ax.legend()
        fig.suptitle(f"{name} benchmark results", fontsize=18)
        ax.set_title("lower is better (log scale)")
        ax.set_yscale("log")

        plt.savefig(os.path.join(perfs_dir, f"{name}.png"), dpi=300)

    if opts.command_type == "graph":
        data = json.load(open(os.path.join(perf_backup), "r"))
        regen_single_graph(ns, name, data)
    elif opts.command_type == "run":
        data = {command: [] for command in commands}

        for n in tqdm(ns):
            tqdm.write(f"=== Size {n} ===")
            results = get_results(filenames, temp_filenames, n)
            for program in results["results"]:
                command = program["command"].split(" ")[0]
                data[command].append((program["mean"], program["stddev"]))

        # clean up generated files
        for temp in temp_filenames:
            os.remove(temp)

        os.remove(json_name)

        json.dump(data, open(perf_backup, "w"))
        regen_single_graph(ns, name, data)
