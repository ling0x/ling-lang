#!/bin/bash

OUTPUT_DIR="tests/test_compiled"
export LING_OUTPUT_DIR="$OUTPUT_DIR"

echo "==================================="
echo "Testing Ling Language Compiler"
echo "==================================="
echo "Compiled output → $OUTPUT_DIR/"

mkdir -p "$OUTPUT_DIR"

PASS=0
FAIL=0

for file in tests/test_programs/*.ling; do
    echo ""
    echo "Testing: $file"
    echo "-----------------------------------"
    cargo run --release "$file"
    
    if [ $? -eq 0 ]; then
        echo "✓ PASS: $file"
        PASS=$((PASS + 1))
    else
        echo "✗ FAIL: $file"
        FAIL=$((FAIL + 1))
    fi
    echo ""
done

echo "==================================="
echo "Results: $PASS passed, $FAIL failed"
echo "==================================="
