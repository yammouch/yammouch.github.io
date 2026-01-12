#!/bin/bash
pushd $(dirname $0)
wasm-pack build --target web
echo -e "class TextDecoder{ decode() {} }\n" | cat - pkg/reson.js > pkg/glue.js
