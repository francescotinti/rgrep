#!/usr/bin/env python3
"""Run from rgrep/ after cargo bench has generated line_scan.c."""
import argparse
import hashlib
import json
import random
import statistics
import subprocess
import time
from pathlib import Path

parser = argparse.ArgumentParser(description="Compare pre/post CLI binaries using alternating pairs.")
parser.add_argument("before", type=Path)
parser.add_argument("after", type=Path)
parser.add_argument("--output", type=Path, default=Path("target/verification/o7-paired.json"))
options = parser.parse_args()
before = str(options.before.resolve())
after = str(options.after.resolve())
file = '../gnu-grep/src/grep.c'
large = 'target/bench_fixtures/line_scan.c'
scenarios = {
 'literal_match_count': ['-c', 'include', *['../gnu-grep/src/' + name for name in ['grep.c','dfasearch.c','kwsearch.c','pcresearch.c','searchutils.c']]],
 'regex_simple': ['-E', r'^[a-z]+\(', file],
 'invert_match': ['-vc', '^$', file],
 'fixed_string_F': ['-rF','MB_LEN_MAX','../gnu-grep/src/'],
 'recursive_walk': ['-r','TODO','../gnu-grep/src/'],
 'color_never_output': ['--color=never','include',file],
 'color_always_output': ['--color=always','include',file],
 'color_never_count': ['--color=never','-c','include',file],
 'color_always_count': ['--color=always','-c','include',file],
 'bufread_default': ['-c','include',file],
 'mmap_explicit': ['--mmap','-c','include',file],
 'quiet_mode': ['-q','include',file],
 'fixed_count': ['-cF','MB_LEN_MAX',file],
 'fixed_quiet': ['-qF','MB_LEN_MAX',file],
 'large_regex_count': ['-cE', r'^[a-z]+\(',large],
 'large_invert_count': ['-vc','^$',large],
 'large_fixed_count': ['-cF','MB_LEN_MAX',large],
}

def run(binary,args):
 start=time.perf_counter_ns()
 result=subprocess.run([binary,*args],capture_output=True)
 elapsed=time.perf_counter_ns()-start
 assert result.returncode in (0,1), result.stderr
 return elapsed,(result.returncode,result.stdout,result.stderr)

report={'method':'31 alternating pre/post pairs, 3 warmups; bootstrap median paired ratio, 10000 resamples, seed 7; negative delta is faster', 'before_sha256':hashlib.sha256(Path(before).read_bytes()).hexdigest(), 'after_sha256':hashlib.sha256(Path(after).read_bytes()).hexdigest(), 'scenarios':{}}
rng=random.Random(7)
for name,args in scenarios.items():
 for _ in range(3):
  run(before,args);run(after,args)
 pre=[];post=[]
 for i in range(31):
  order=[('pre',before),('post',after)] if i%2==0 else [('post',after),('pre',before)]
  results={key:run(binary,args) for key,binary in order}
  assert results['pre'][1]==results['post'][1],name
  pre.append(results['pre'][0]);post.append(results['post'][0])
 ratios=[(b/a-1)*100 for a,b in zip(pre,post)]
 boot=sorted(statistics.median(rng.choices(ratios,k=len(ratios))) for _ in range(10000))
 entry={'args':args,'pre_ns':pre,'post_ns':post,'pre_median_ms':statistics.median(pre)/1e6,'post_median_ms':statistics.median(post)/1e6,'paired_delta_percent':statistics.median(ratios),'paired_bootstrap_95_percent':[boot[250],boot[9749]],'output_parity':True}
 report['scenarios'][name]=entry
 print(name,round(entry['paired_delta_percent'],2),entry['paired_bootstrap_95_percent'],flush=True)
options.output.parent.mkdir(parents=True, exist_ok=True)
options.output.write_text(json.dumps(report,indent=2)+'\n')
