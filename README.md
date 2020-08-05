# swarm-website-visualization
Visualization Software for Swarm Website

### Setup
- Install Rust: https://www.rust-lang.org/tools/install  
- Install wasm-pack: https://rustwasm.github.io/wasm-pack/installer/

### Building
#### Build Script
Requires [rjsmin](https://pypi.org/project/rjsmin/) (`pip install rjsmin`)
```
# Use --help to see all available arguments
python ./build.py
```
#### Manual Command
Use the following command for a best compatibility debug build
```
wasm-pack build --out-dir build --no-typescript --target no-modules --dev -- --features "debug"
```
Optional changes:
 - Omit `--no-typescript` if you want to typescript files to be generated as well
 - Use `--target web` instead of `--target no-modules` if you want to have `wasm-pack` generate ES modules  
 - Omit `--dev -- --features "debug"` to build a release build
  
All output is within the `./build` directory. See `testsite.html` to see how to include the generated files into a web page  

### Running
- The included `testsite.html` is a simple html file that loads the generated javascript/WASM file(s).  
- `testserver.py` is a python script that will run a web server that supports WASM at http://127.0.0.1:8080/testsite.html  
- Python version 3.7.5 or higher is required. Script is from [here](https://cggallant.blogspot.com/2020/07/extending-pythons-simple-http-server.html)