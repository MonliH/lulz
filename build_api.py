import subprocess
BUILD_SETTINGS = ["--release", "--target-dir", "target", "-q"]

print("Building...")
subprocess.run(
    [
        "cargo",
        "build",
    ]
    + BUILD_SETTINGS
)
print("Done Building...")
