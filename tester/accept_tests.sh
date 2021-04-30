#!/bin/bash

set -o errexit 

CURRENTDIR=$(pwd)
echo Current dir is $CURRENTDIR
CHECKPROG=$(pwd)/../target/debug/dpr
CONFFILE=$(pwd)/my.cfg

VERSION=$($CHECKPROG --version)

for i in $(ls -d */); do
    echo Accepting $i
    cd $i
    ./go.sh "$CHECKPROG" "$CONFFILE" > expected-output.txt
    rm -f .jobcounter .jobfile
    cd ..
done
