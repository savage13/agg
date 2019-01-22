#!/bin/sh

# Compiling the many integration tests takes a long time
#  -f -- This copies the test directory and save the specific test back to the
#    test diretory, speeding up the edit/compile/test cycle
#  -u -- Unfocusing on a test, copies all of the tests back to the
#    integration test directory
#
# Usage:
#    # To focus on a test
#      ./focus_on_itest.sh -f tests/test_I_am_working_on.rs
#    # To un-focus on a test
#      ./focus_on_itest.sh -u tests/test_I_am_working_on.rs
#

ARG="$1"
VAL=$(basename "$2")
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
fi
if [ x$ARG == x"-u" ]; then
    echo $VAL
    if [ ! -e $ORG ]; then
        echo "$ORG does not exist, exiting"
        exit -1
    fi
    cp $TDIR/$VAL $ORG
    rm -r $TDIR
    mv $ORG $TDIR
fi
