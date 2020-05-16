#!/bin/bash
if [[ $# == 0 ]]; then
    echo "need folder"
    exit 0
fi
if [[ $# == 1 ]]; then
    echo "need folder and datafile"
    exit 0
fi
folder=$1
cargo run --release $folder load | tee test/output
sleep 0.1
cat test/output | grep PLOT > $2
gnuplot -p test/plot.gp
