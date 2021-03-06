#!/bin/bash

set -o errexit 

CURRENTDIR=$(pwd)
echo Current dir is $CURRENTDIR
CHECKPROG=$(pwd)/../target/debug/dpr
CONFFILE=$(pwd)/my.cfg

VERSION=$($CHECKPROG --version)

for i in $(ls -d */); do
    echo Testing $i
    cd $i
    ./go.sh "$CHECKPROG" "$CONFFILE" > /tmp/output.txt
    diff expected-output.txt /tmp/output.txt
    rm -f /tmp/output.txt .jobcounter .jobfile
    cd ..
done
