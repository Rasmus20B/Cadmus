#!/bin/sh

cp template.fmp12 input/blank.fmp12
cp template.fmp12 input/relation.fmp12
cp template.fmp12 input/action_sack.fmp12
cp template.fmp12 input/big_tables.fmp12

osascript generate_inner.applescript
