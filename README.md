# swarm-website-visualization
Visualization Software for Swarm Website

### Setup
Install Rust: https://www.rust-lang.org/tools/install  
Install wasm-pack: https://rustwasm.github.io/wasm-pack/installer/

### Building
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
This must be run within an actual web server. Something like nodejs' `live-server`<sup>1</sup> or python3's `http.server`<sup>2</sup> can be used to easily test the output.  
The included `testsite.html` is a simple html file that loads the generated javascript/WASM file(s).

<sup>1</sup>  `live-server` is a nodejs/npm package  
nodejs: https://nodejs.org/en/  
live-server: https://www.npmjs.com/package/live-server  
`live-server` will open `testsite.html` in your browser automatically  
```shell
// Install live-server
npm install -g live-server
// Run the following line in the root project directory
live-server testsite.html
```

<sup>2</sup>  `http.server` is part of python3's standard library  
http.server docs: https://docs.python.org/3/library/http.server.html#module-http.server  
Run the following command and then go to http://localhost:8000/testsite.html  
```shell
// Run the following in the root project directory
python -m http.server
```