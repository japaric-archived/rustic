#!/bin/bash

set -e
set -x

pwd=$(pwd)
export PATH="${PATH}:${pwd}/target/debug"

pushd examples

# No arguments
[[ $(rustic hello.rs) == "Hello, world!" ]]

# With arguments
[[ $(rustic args.rs a b c) == '["a", "b", "c"]' ]]

# Cargo dependencies
[[ $(rustic rand.rs) =~ ^[0-9]+$ ]]

# Shebang
echo "#!${pwd}/target/debug/rustic" > script.rs
cat hello.rs >> script.rs
chmod +x script.rs
[[ $(./script.rs) == "Hello, world!" ]]
rm script.rs

popd
