#!/usr/bin/env python3
"""Compare paq binaries with byte equality and Linux per-process rusage.

All measured children run sequentially; configuration order is shuffled for each
round. The native helper excludes Python's pre-exec RSS and harness CPU time.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import random
import signal
import statistics
import subprocess
import tempfile
import time


def measure(helper, report, binary, args, env, expected=None, library=False):
    with subprocess.Popen([str(helper), str(report), str(binary), *args], env=env,
                          stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                          start_new_session=True) as child:
        try:
            stdout, stderr = child.communicate(timeout=300)
        except subprocess.TimeoutExpired:
            os.killpg(child.pid, signal.SIGKILL)
            child.communicate()
            raise RuntimeError(f'{binary}: resource measurement timed out') from None
        if child.returncode:
            raise RuntimeError(f'{binary}: exit {child.returncode}: {stderr!r}')
    metrics = json.loads(report.read_text())
    if library:
        digest, iterations, loop_ms = stdout.decode('ascii').strip().split()
        metrics.update(iterations=int(iterations), loop_wall_ms=float(loop_ms))
        metrics['loop_per_call_ms'] = float(loop_ms) / int(iterations)
        metrics['cpu_per_call_ms'] = metrics['cpu_ms'] / int(iterations)
        metrics['process_per_call_ms'] = metrics['wall_ms'] / int(iterations)
        output = digest.encode('ascii') + b'\n'
    else:
        output = stdout
    if expected is not None and output != expected:
        raise RuntimeError(f'hash/output mismatch: {binary} {args}: {output!r} != {expected!r}')
    return metrics, output


def summarize(rows):
    result = {}
    for key in rows[0]:
        values = [r[key] for r in rows]
        if not isinstance(values[0], (int, float)):
            continue
        ordered = sorted(values)
        result[key] = {'median': statistics.median(values), 'mean': statistics.mean(values),
                       'min': ordered[0], 'max': ordered[-1],
                       'p95': ordered[min(len(ordered)-1, int(0.95*len(ordered)))],
                       'stdev': statistics.stdev(values) if len(values)>1 else 0}
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binaries', type=Path, required=True, help='JSON list with name, cli, library, revision')
    parser.add_argument('--corpus', type=Path, required=True, help='JSON workloads: name, path, library_iterations')
    parser.add_argument('--helper', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--runs', type=int, default=20)
    parser.add_argument('--warmups', type=int, default=3)
    parser.add_argument('--threads', default='default', help='comma separated: default,1,4')
    parser.add_argument('--modes', default='cli', help='cli,library')
    parser.add_argument('--cases', default='', help='comma separated subset of workload names')
    parser.add_argument('--seed', type=int, default=20260907)
    args = parser.parse_args()
    if platform.system() != 'Linux' or args.runs < 1 or args.warmups < 0:
        parser.error('Linux and positive run count required')
    binaries = json.loads(args.binaries.read_text())
    cases = json.loads(args.corpus.read_text())
    if args.cases:
        selected = set(args.cases.split(',')); cases = [x for x in cases if x['name'] in selected]
        if selected != {x['name'] for x in cases}: parser.error('unknown case')
    for binary in binaries:
        for key in ('cli','library'):
            if key in binary:
                p=Path(binary[key]); binary[key]=str(p.resolve())
                binary[key+'_sha256']=hashlib.sha256(p.read_bytes()).hexdigest()
    envbase=os.environ.copy(); envbase.pop('RAYON_NUM_THREADS',None)
    rng=random.Random(args.seed)
    data={'method': 'Linux native fork/exec/wait4; 100% CPU = one logical core; RSS in KiB',
          'cache': 'warm filesystem; no global cache eviction', 'platform':platform.platform(),
          'logical_cpus':os.cpu_count(), 'allowed_cpus':sorted(os.sched_getaffinity(0)),
          'start_utc':time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime()),
          'runs':args.runs,'warmups':args.warmups,'seed':args.seed,
          'binaries':binaries,'workloads':cases,'results':[]}
    args.output.parent.mkdir(parents=True,exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='paq-measure-') as temp:
        report=Path(temp)/'usage.json'
        for mode in args.modes.split(','):
            for threads in args.threads.split(','):
                env=envbase.copy()
                if threads!='default':env['RAYON_NUM_THREADS']=threads
                for case in cases:
                    source=str(Path(case['path']).resolve())
                    ignored=case.get('ignore_hidden',False)
                    cliargs=[source]+(['-i'] if ignored else [])
                    reference=subprocess.run([binaries[0]['cli'],*cliargs],env=env,capture_output=True,check=True).stdout
                    samples={b['name']:[] for b in binaries}
                    for b in binaries:
                        commandargs=cliargs if mode=='cli' else [source,str(case['library_iterations'])]+(['ignore-hidden'] if ignored else [])
                        for _ in range(max(args.warmups,1)):
                            measure(args.helper.resolve(),report,b[mode],commandargs,env,reference,mode=='library')
                    for _ in range(args.runs):
                        order=binaries.copy();rng.shuffle(order)
                        for b in order:
                            commandargs=cliargs if mode=='cli' else [source,str(case['library_iterations'])]+(['ignore-hidden'] if ignored else [])
                            metric,_=measure(args.helper.resolve(),report,b[mode],commandargs,env,reference,mode=='library')
                            metric['machine_cpu_percent']=metric['cpu_percent']/len(data['allowed_cpus'])
                            samples[b['name']].append(metric)
                    for b in binaries:
                        data['results'].append({'mode':mode,'threads':threads,'workload':case['name'],
                            'variant':b['name'],'hash':reference.decode('ascii').strip(),
                            'summary':summarize(samples[b['name']]),'samples':samples[b['name']]})
                    args.output.write_text(json.dumps(data,indent=2)+'\n')
                    metric='wall_ms' if mode=='cli' else 'loop_per_call_ms'
                    print(json.dumps({'mode':mode,'threads':threads,'workload':case['name'],
                        'median_ms':{b['name']:round(summarize(samples[b['name']])[metric]['median'],4) for b in binaries}}),flush=True)
    data['end_utc']=time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime())
    args.output.write_text(json.dumps(data,indent=2)+'\n')


if __name__ == '__main__':
    main()
