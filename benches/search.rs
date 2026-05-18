//! rgrep performance benchmarks (Ondata 4).
//!
//! Process-spawn based: every scenario invokes the compiled `rgrep` binary
//! (via `CARGO_BIN_EXE_rgrep`) and — where applicable — also invokes
//! `/usr/bin/grep` (system BSD/GNU grep) and `rg` (ripgrep) for comparison.
//! ripgrep is skipped silently when not present in `$PATH`.
//!
//! See `PERF_REPORT.md` for methodology, results, and interpretation.

use criterion::{Criterion, criterion_group, criterion_main};
use std::process::Command;

const SYS_GREP: &str = "/usr/bin/grep";

fn has_ripgrep() -> bool {
    which::which("rg").is_ok()
}

fn run(cmd: &str, args: &[&str]) {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .expect("failed to spawn benchmarked command");
    // Drop output explicitly so criterion does not optimise the spawn away.
    let _ = output.stdout.len();
    let _ = output.stderr.len();
}

fn bench_literal_match_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("literal_match_count");
    let rgrep = env!("CARGO_BIN_EXE_rgrep");
    let pattern = "include";
    let inputs = [
        "../gnu-grep/src/grep.c",
        "../gnu-grep/src/dfasearch.c",
        "../gnu-grep/src/kwsearch.c",
        "../gnu-grep/src/pcresearch.c",
        "../gnu-grep/src/searchutils.c",
    ];
    group.bench_function("rgrep", |b| {
        b.iter(|| {
            let mut args = vec!["-c", pattern];
            args.extend(inputs.iter().copied());
            run(rgrep, &args);
        });
    });
    group.bench_function("system_grep", |b| {
        b.iter(|| {
            let mut args = vec!["-c", pattern];
            args.extend(inputs.iter().copied());
            run(SYS_GREP, &args);
        });
    });
    if has_ripgrep() {
        group.bench_function("ripgrep", |b| {
            b.iter(|| {
                let mut args = vec!["-c", pattern];
                args.extend(inputs.iter().copied());
                run("rg", &args);
            });
        });
    }
    group.finish();
}

fn bench_regex_simple(c: &mut Criterion) {
    let mut group = c.benchmark_group("regex_simple");
    let rgrep = env!("CARGO_BIN_EXE_rgrep");
    let pattern = "^[a-z]+\\(";
    let file = "../gnu-grep/src/grep.c";
    group.bench_function("rgrep", |b| {
        b.iter(|| run(rgrep, &["-E", pattern, file]));
    });
    group.bench_function("system_grep", |b| {
        b.iter(|| run(SYS_GREP, &["-E", pattern, file]));
    });
    if has_ripgrep() {
        group.bench_function("ripgrep", |b| {
            b.iter(|| run("rg", &[pattern, file]));
        });
    }
    group.finish();
}

fn bench_fixed_string_f(c: &mut Criterion) {
    let mut group = c.benchmark_group("fixed_string_F");
    let rgrep = env!("CARGO_BIN_EXE_rgrep");
    let pattern = "MB_LEN_MAX";
    let dir = "../gnu-grep/src/";
    group.bench_function("rgrep", |b| {
        b.iter(|| run(rgrep, &["-rF", pattern, dir]));
    });
    group.bench_function("system_grep", |b| {
        b.iter(|| run(SYS_GREP, &["-rF", pattern, dir]));
    });
    if has_ripgrep() {
        group.bench_function("ripgrep", |b| {
            b.iter(|| run("rg", &["-F", pattern, dir]));
        });
    }
    group.finish();
}

fn bench_recursive_walk(c: &mut Criterion) {
    let mut group = c.benchmark_group("recursive_walk");
    group.sample_size(20);
    let rgrep = env!("CARGO_BIN_EXE_rgrep");
    let pattern = "TODO";
    let dir = "../gnu-grep/src/";
    group.bench_function("rgrep", |b| {
        b.iter(|| run(rgrep, &["-r", pattern, dir]));
    });
    group.bench_function("system_grep", |b| {
        b.iter(|| run(SYS_GREP, &["-r", pattern, dir]));
    });
    if has_ripgrep() {
        group.bench_function("ripgrep", |b| {
            b.iter(|| run("rg", &[pattern, dir]));
        });
    }
    group.finish();
}

fn bench_invert_match(c: &mut Criterion) {
    let mut group = c.benchmark_group("invert_match");
    let rgrep = env!("CARGO_BIN_EXE_rgrep");
    let pattern = "^$";
    let file = "../gnu-grep/src/grep.c";
    group.bench_function("rgrep", |b| {
        b.iter(|| run(rgrep, &["-v", "-c", pattern, file]));
    });
    group.bench_function("system_grep", |b| {
        b.iter(|| run(SYS_GREP, &["-v", "-c", pattern, file]));
    });
    if has_ripgrep() {
        group.bench_function("ripgrep", |b| {
            b.iter(|| run("rg", &["-v", "-c", pattern, file]));
        });
    }
    group.finish();
}

fn bench_cow_impact(c: &mut Criterion) {
    let mut group = c.benchmark_group("cow_impact");
    let rgrep = env!("CARGO_BIN_EXE_rgrep");
    let pattern = "include";
    let file = "../gnu-grep/src/grep.c";
    group.bench_function("color_never_with_output", |b| {
        b.iter(|| run(rgrep, &["--color=never", pattern, file]));
    });
    group.bench_function("color_always_with_output", |b| {
        b.iter(|| run(rgrep, &["--color=always", pattern, file]));
    });
    group.bench_function("color_never_count_only", |b| {
        b.iter(|| run(rgrep, &["--color=never", "-c", pattern, file]));
    });
    group.bench_function("color_always_count_only", |b| {
        b.iter(|| run(rgrep, &["--color=always", "-c", pattern, file]));
    });
    group.finish();
}

fn bench_mmap_vs_bufread(c: &mut Criterion) {
    let mut group = c.benchmark_group("mmap_vs_bufread");
    let rgrep = env!("CARGO_BIN_EXE_rgrep");
    let pattern = "include";
    let file = "../gnu-grep/src/grep.c";
    group.bench_function("bufread_default", |b| {
        b.iter(|| run(rgrep, &["-c", pattern, file]));
    });
    group.bench_function("mmap_explicit", |b| {
        b.iter(|| run(rgrep, &["--mmap", "-c", pattern, file]));
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_literal_match_count,
    bench_regex_simple,
    bench_fixed_string_f,
    bench_recursive_walk,
    bench_invert_match,
    bench_cow_impact,
    bench_mmap_vs_bufread,
);
criterion_main!(benches);
