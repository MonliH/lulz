import os
import json
import subprocess
import matplotlib.pyplot as plt
import argparse
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

graph = subparsers.add_parser(
    "run",
    help="run the same benchmark (in ./perfs/) multiple times, with \
    different input sizes, and composite into a line graph",
)
graph.add_argument(
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

exts = [".lol", ".lci.lol", ".py"]
commands = ["./target/release/lulz", "lci", "python"]

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
    data = {language: [] for language in commands}
    for value_filename in glob("./perfs/*/values.json"):
        value_json = json.load(open(value_filename, "r"))
        folder_name = os.path.dirname(value_filename)
        name = os.path.basename(folder_name)
        (filenames, temp_filenames, ns) = get_filenames_from_name(name, folder_name)
        results = get_results(filenames, temp_filenames, value_json["one"])
        for program in results["results"]:
            command = program["command"].split(" ")[0]
            data[command].append((name, program["mean"], program["stddev"]))
    print(data)
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

