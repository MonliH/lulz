import subprocess

BUILD_SETTINGS = ["-O3"]

print("Building...")
try:
    subprocess.check_output(
        [
            "rpython",
            "src/main.py",
        ]
        + BUILD_SETTINGS
    )
except subprocess.CalledProcessError as code:
    print(f"Failed to compile, exited with code {code.returncode}")
    exit(1)
print("Done Building...")
