#!/usr/bin/env python3
# rgrep — GNU grep ported to Rust (test flag cleaner helper)
# Copyright (c) 2026 Francesco Tinti <francesco.tinti@activemind.it>
#
# AI-assisted port:
#   Architect: Claude Opus 4.7 (1M context, Anthropic)
#   Implementer: Gemini Antigravity (Google)
#
# https://github.com/francescotinti/rgrep

import os
import glob

def clean_expected_to_fail():
    count = 0
    for fpath in glob.glob("tests/cases/gnu_*.toml"):
        with open(fpath, "r") as f:
            content = f.read()
            
        if "expected_to_fail" in content:
            # We must remove lines containing expected_to_fail
            new_lines = []
            for line in content.split('\n'):
                if not line.startswith("expected_to_fail"):
                    new_lines.append(line)
            
            new_content = '\n'.join(new_lines)
            
            with open(fpath, "w") as f:
                f.write(new_content)
            count += 1
            
    print(f"Removed expected_to_fail from {count} testcases.")

if __name__ == "__main__":
    os.chdir(os.path.dirname(os.path.abspath(__file__)))
    os.chdir("..")
    clean_expected_to_fail()
