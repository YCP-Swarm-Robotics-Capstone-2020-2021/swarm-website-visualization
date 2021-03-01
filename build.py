import os
import argparse
import rjsmin


JS_PATH_IN = "./build/swarm_website_visualization.js"
JS_PATH_OUT = "./build/swarm_website_visualization.min.js"

parser = argparse.ArgumentParser(description="Project build script")
parser.add_argument("-r", "--release", help="Create a release build. Dev/debug build is assumed otherwise", action="store_true")
parser.add_argument("--no-minjs", help="Disable the creation of a *.min.js", action="store_true")
parser.add_argument("--no-modules", help="Use the 'no-modules' build target instead of 'web'", action="store_true")
args = parser.parse_args()

BUILD_REL = "wasm-pack build --out-dir build --no-typescript --target "

if args.no_modules:
    BUILD_REL += "no-modules"
else:
    BUILD_REL += "web"

BUILD_DEV = BUILD_REL + " --dev -- --features \"debug\""


if args.release:
    print("Building release build...")
    os.system(BUILD_REL)
else:
    print("Building dev build...")
    print(BUILD_DEV)
    os.system(BUILD_DEV)
print("Done")

if not args.no_minjs:
    print("Minifying JS...")

    js = open(JS_PATH_IN).read()

    minjs = open("./build/swarm_website_visualization.min.js", "w")
    minjs.write(rjsmin.jsmin(js))
    minjs.close()
    print("Done")
