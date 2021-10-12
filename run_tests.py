import os
import textwrap
import json
import subprocess
import difflib
from subprocess import Popen, PIPE, STDOUT
from glob import iglob
import build_api

GREEN = "\033[32m"
RED = "\033[91m"
BLUE = "\033[34m"
YELLOW = "\033[93m"
RESET = "\033[0m"
GRAY = "\033[38;5;243m"


def colored(s: str, c: str) -> str:
    return f"{c}{s}{RESET}"


def run_file(filename, stdin) -> (str, int):
    p = Popen(
        ["./target/release/lulz", filename],
        stdout=PIPE,
        stdin=PIPE,
        stderr=PIPE,
    )
    out, err = p.communicate(input=bytes(stdin, encoding="utf8"))
    return (out, p.returncode, err)


rootdir_glob = "tests/**/*.lol"
file_list = (f for f in iglob(rootdir_glob, recursive=True) if os.path.isfile(f))
failed = 0
passed = 0

for filename in file_list:
    print(colored(f"{filename}...", GRAY), end=" ")
    with open(filename, "r") as f:
        header = f.readline()

    if not header:
        print(colored("warning: has an empty header. skipping", YELLOW))
        continue

    header = json.loads("{" + header.split("{")[1])
    stdin = ""
    if "input" in header:
        stdin = header["input"]

    res = run_file(filename, stdin)
    output = res[0].decode("utf-8")
    stderr = res[2].decode("utf-8")

    if "status" in header and header["status"] == "error":
        print(colored("should error.", BLUE), end=" ")
        if res[1] == 0:
            print(colored("test failed.", RED))
            failed += 1
        else:
            print(colored("test passed.", GREEN))
            passed += 1
        continue

    if header["output"] != output:
        print(colored("test failed.", RED))
        esc_output = output.replace("\n", "\\n")
        header_output = header["output"].replace("\n", "\\n")
        print(
            f'  {colored("saw", BLUE)}:\n    "{esc_output}"\n  {colored("expected", BLUE)}:\n    "{header_output}"\n'
        )
        print(f"  stderr:\n")
        print(f"{textwrap.indent(stderr, '    ')}")
        failed += 1
        continue

    print(colored(f"test passed.", GREEN))
    passed += 1

print(
    colored(f"{passed} test{'s' if passed != 1 else ''} passed.", GREEN),
    colored(f"{failed} test{'s' if failed != 1 else ''} failed.", RED),
)

if failed:
    exit(1)
