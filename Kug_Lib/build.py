#!/usr/bin/python3
import sys
import os

def build(platform: str, release: bool):
    release_flag = ""

    if release:
        release_flag = "--release"

    if platform.startswith("win"): # Windows
        if sys.platform.startswith("linux"): # Check and see if we are trying to compile a Windows lib on Linux.
            build_flag = "x86_64-pc-windows-gnu"
        else:
            build_flag = "x86_64-pc-windows-msvc"

        file = "kug_lib.dll"
        cmd_name = "powershell.exe -c cp"
    elif platform.startswith("linux"):
        build_flag = "x86_64-unknown-linux-gnu"
        file = "libkug_lib.so"
        cmd_name = "cp"
    else:
        print(f"Platform {platform} is not currently supported. Allowed platforms are win32 and linux.")
        sys.exit(1)

    os.system(f"cargo build --target {build_flag} {release_flag}")

    if not os.path.exists(f"../Kug_App/lib"):
        os.mkdir(os.path.join("..", "Kug_App", "lib"))

    os.system(f"{cmd_name} target/{build_flag}/debug/{file} ../Kug_App/lib/{file}")

def main(*args):
    if len(args) == 0:
        print("No args specified. Valid args: `build`, `build-win`, `build-linux`")
        sys.exit(-1)

    match args[0]:
        case "build": # Build file for current platform
            build(sys.platform, False)
        case "build-win":
            build("win32", False)
        case "build-linux":
            build("linux", False)
        case "build-all":
            build("win32", False)
            build("linux", False)
        case _: print(f"Unknown command: `{args[0]}`")

if __name__ == "__main__":
    sys.argv.pop(0)
    main(*sys.argv)

