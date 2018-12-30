#!/bin/sh

ARG="$1"
VAL=$(basename "$2")
PPM=$(echo $VAL | sed 's/\.rs/\.ppm/')
ORG=tests.org
TDIR=tests

if [ x$ARG == x"-f" ]; then
    echo $VAL
    if [ -e $ORG ]; then
        echo "$ORG already exists, exiting"
        exit -1
    fi
    mv $TDIR $ORG
    mkdir $TDIR
    cp $ORG/$VAL $TDIR
    cp $ORG/$PPM $TDIR
fi
if [ x$ARG == x"-u" ]; then
    echo $VAL
    if [ ! -e $ORG ]; then
        echo "$ORG does not exist, exiting"
        exit -1
    fi
    cp $TDIR/$VAL $ORG
    cp $TDIR/$PPM $ORG
    rm -r $TDIR
    mv $ORG $TDIR
fi
