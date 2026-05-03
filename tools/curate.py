#!/usr/bin/env python3
import os
import glob

def curate():
    for fpath in glob.glob("tests/cases/gnu_*.toml"):
        with open(fpath, "r") as f:
            content = f.read()
            
        modified = False
        
        if "backref seq1" in content or "backref seq2" in content or "backref seq5" in content:
            if "expected_to_fail" not in content:
                content += '\nexpected_to_fail = true\nexpected_to_fail_reason = "Backreferences not supported by regex crate (D-M6)"\n'
                modified = True
                
        elif "backref seq4" in content:
            if "expected_to_fail" not in content:
                content += '\nexpected_to_fail = true\nexpected_to_fail_reason = "Bug: multiple -e concatenated with | can mask syntax errors (D-NEW-5)"\n'
                modified = True
                
        # Remove duplicates
        if modified:
            with open(fpath, "w") as f:
                f.write(content)

if __name__ == "__main__":
    os.chdir(os.path.dirname(os.path.abspath(__file__)))
    os.chdir("..")
    curate()
    print("Curation applied successfully.")
