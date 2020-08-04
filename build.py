import os
import argparse
import rjsmin

BUILD_REL = "wasm-pack build --out-dir build --no-typescript --target no-modules"
BUILD_DEV = BUILD_REL + " --dev -- --features \"debug\""

JS_PATH_IN = "./build/swarm_website_visualization.js"
JS_PATH_OUT = "./build/swarm_website_visualization.min.js"

parser = argparse.ArgumentParser(description="Project build script")
parser.add_argument("-r", "--release", help="Create a release build. Dev/debug build is assumed otherwise", action="store_true")
parser.add_argument("--no-minjs", help="Disable the creation of a *.min.js", action="store_true")
args = parser.parse_args();

if args.release:
    print("Building release build...")
    os.system(BUILD_REL)
else:
    print("Building dev build...")
    os.system(BUILD_DEV)
print("Done")

if not args.no_minjs:
    print("Minifying JS...")
    minjs = open("./build/swarm_website_visualization.min.js", "w")
    minjs.write(rjsmin.jsmin(JS_PATH_IN))
    minjs.close()
    print("Done")
