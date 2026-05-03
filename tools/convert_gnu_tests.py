#!/usr/bin/env python3
# rgrep — GNU grep ported to Rust (gnu-grep test importer)
# Copyright (c) 2026 Francesco Tinti <francesco.tinti@activemind.it>
#
# AI-assisted port:
#   Architect: Claude Opus 4.7 (1M context, Anthropic)
#   Implementer: Gemini Antigravity (Google)
#
# https://github.com/francescotinti/rgrep
import os
import re
import sys
import glob
import shlex
import subprocess

def parse_gnu_tests():
    test_dir = "../gnu-grep/tests"
    out_dir = "tests/cases"
    manifest = "tests/testsuite.toml"
    
    if not os.path.exists(test_dir):
        print(f"Error: {test_dir} not found.")
        sys.exit(1)
        
    os.makedirs(out_dir, exist_ok=True)
    
    try:
        with open(manifest, 'r') as f:
            manifest_content = f.read()
    except FileNotFoundError:
        manifest_content = "cases = [\n]"
        
    existing = re.findall(r'"cases/(\d+)_.*\.toml"', manifest_content)
    existing += re.findall(r'"cases/gnu_(\d+)_.*\.toml"', manifest_content)
    
    next_idx = max([int(x) for x in existing] + [74]) + 1
    
    exclude_patterns = [
        (r'LC_ALL=(?!C\b|POSIX\b)[^\s]+', 'locale-specific'),
        (r'LANG=(?!C\b|POSIX\b)[^\s]+', 'locale-specific'),
        (r'get-mb-cur-max', 'requires C helper binary'),
        (r'\bcmp\b', 'compares binary output with cmp'),
        (r'\bod\b', 'compares binary output with od'),
        (r'\$\{SHELL\}', 'uses ${SHELL} interpolation'),
        (r'\bIFS=', 'manipulates IFS'),
        (r'fr_FR\.UTF-8', 'locale skip'),
        (r'timeout', 'uses timeout command'),
        (r'require_timeout_', 'requires timeout')
    ]
    
    generated_files = []
    
    for filepath in sorted(glob.glob(os.path.join(test_dir, "*"))):
        if not os.path.isfile(filepath): continue
        if filepath.endswith(".sh") is False and "." in os.path.basename(filepath): 
            if not filepath.endswith(".awk") and not filepath.endswith(".pl"):
                pass
            else:
                continue
                
        with open(filepath, 'r', encoding='utf-8', errors='ignore') as f:
            content = f.read()
            
        filename = os.path.basename(filepath)
        
        skip_reason = None
        for pat, reason in exclude_patterns:
            if re.search(pat, content):
                skip_reason = reason
                break
                
        if skip_reason:
            print(f"[skip] tests/{filename}: {skip_reason}")
            continue
            
        clean_content = re.sub(r'\\\n\s*', ' ', content)
        
        blocks = re.finditer(r'(echo|printf)\s+(.*?)\s*\|\s*grep\s+(.*?)\s*(?:>.*?)?\n\s*(?:if\s+test|test)\s+\$\?\s+-ne\s+(\d+)', clean_content)
        
        count = 0
        for b in blocks:
            cmd_type, stdin_raw, grep_args_raw, exit_code = b.groups()
            
            if stdin_raw.startswith("'") and stdin_raw.endswith("'"):
                stdin_val = stdin_raw[1:-1]
            elif stdin_raw.startswith('"') and stdin_raw.endswith('"'):
                stdin_val = stdin_raw[1:-1]
            else:
                stdin_val = stdin_raw
                
            if cmd_type == 'printf':
                stdin_val = stdin_val.replace('\\n', '\n').replace('\\t', '\t').replace('\\\\', '\\')
            else:
                stdin_val += '\n'
                
            try:
                args = shlex.split(grep_args_raw)
            except ValueError:
                continue
                
            # Run grep to get actual expected_stdout
            try:
                proc = subprocess.run(['grep'] + args, input=stdin_val.encode('utf-8'), stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=2)
                expected_stdout = proc.stdout.decode('utf-8', errors='replace')
                actual_exit_code = proc.returncode
            except Exception as e:
                continue
                
            seq = count + 1
            toml_filename = f"gnu_{next_idx:04d}_{filename}_seq{seq}.toml"
            toml_path = os.path.join(out_dir, toml_filename)
            
            import json
            with open(toml_path, 'w') as f:
                f.write(f'name = {json.dumps("gnu " + filename + " seq" + str(seq))}\n')
                args_str = ", ".join(json.dumps(a) for a in args)
                f.write(f'args = [{args_str}]\n')
                f.write(f'stdin = {json.dumps(stdin_val)}\n')
                f.write(f'expected_stdout = {json.dumps(expected_stdout)}\n')
                f.write(f'expected_exit_code = {actual_exit_code}\n')
                f.write(f'source = {json.dumps("gnu-grep/tests/" + filename)}\n')
                
            generated_files.append(f"cases/{toml_filename}")
            next_idx += 1
            count += 1
            
        if count > 0:
            print(f"[ok]   tests/{filename}: {count} testcase generated")
        else:
            print(f"[skip] tests/{filename}: no simple grep patterns found")
            
    if generated_files:
        cases_idx = manifest_content.find("cases = [")
        if cases_idx != -1:
            end_idx = manifest_content.find("]", cases_idx)
            existing_list = manifest_content[cases_idx+9:end_idx].strip()
            
            new_list = existing_list
            if new_list and not new_list.endswith(','):
                new_list += ','
                
            for g in generated_files:
                new_list += f'\n    "{g}",'
                
            new_list = new_list.rstrip(',')
            
            new_manifest = manifest_content[:cases_idx+9] + "\n    " + new_list.strip() + "\n" + manifest_content[end_idx:]
            with open(manifest, 'w') as f:
                f.write(new_manifest)
                
        print(f"Total {len(generated_files)} cases generated and appended to testsuite.toml.")
    else:
        print("No cases generated.")

if __name__ == "__main__":
    os.chdir(os.path.dirname(os.path.abspath(__file__)))
    os.chdir("..")
    parse_gnu_tests()
