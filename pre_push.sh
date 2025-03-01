#!/bin/bash

cargo check --no-default-features
if [[ $? -ne 0 ]]; then; exit 1; fi

cargo check --target x86_64-pc-windows-gnu --no-default-features
if [[ $? -ne 0 ]]; then; exit 1; fi

echo "tests ok"