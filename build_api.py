import subprocess

BUILD_SETTINGS = ["--release", "--target-dir", "target"]

print("Building...")
try:
    subprocess.check_output(
        [
            "cargo",
            "build",
        ]
        + BUILD_SETTINGS
    )
except subprocess.CalledProcessError as code:
    print(f"Failed to compile, exited with code {code.returncode}")
    exit(1)
print("Done Building...")
