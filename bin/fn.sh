#!/usr/bin/env bash

here=`cd $(dirname $BASH_SOURCE); pwd`
source $here/env.sh

# test-module
`cd $MODULE_DIR && ln -f -s $MODULE_ORIGINAL $MODULE_RENAMED`

FN=$1
MODULE="$MODULE_DIR/$MODULE_RENAMED"
TEST="$PROJECT_ROOT/test-module/src/test.el"

$EMACS -batch -l $MODULE -l $TEST -f $FN