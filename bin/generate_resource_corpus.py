#!/usr/bin/env python3
"""Generate deterministic resource workloads in a new directory, optionally with Go."""
import argparse
import hashlib
import json
from pathlib import Path
import tarfile
import urllib.request

GO_COMMIT = '6e676ab2b809d46623acb5988248d95d1eb7939c'


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('directory',type=Path)
    parser.add_argument('--include-go',action='store_true')
    args=parser.parse_args()
    root=args.directory.resolve()
    root.mkdir(parents=True,exist_ok=False)
    block=hashlib.shake_256(b'paq-resource-benchmark-2026-09-07').digest(1024*1024)
    rows=[]

    def write(path,size):
        with path.open('wb') as file:
            while size:
                n=min(size,len(block));file.write(block[:n]);size-=n

    for name,count,size,iterations,wide in [
        ('empty',0,0,500,False),('tiny-10',10,10,300,False),
        ('tiny-99',99,10,100,False),('tiny-1000',1000,10,20,False),
        ('tiny-10000',10000,10,5,False),('buffered-99',99,65536,20,False),
        ('buffered-1000',1000,65536,5,False),('large-8',8,16*1024*1024,3,False),
        ('wide-empty-100000',100000,0,1,True),('empty-200000',200000,0,1,False),
    ]:
        directory=root/name;directory.mkdir()
        for i in range(count):
            parent=directory
            if wide:
                parent=directory/f'd{i//10:05}';parent.mkdir(exist_ok=True)
            write(parent/f'f{i:06}',size)
        rows.append(dict(name=name,path=str(directory),files=count,bytes=count*size,library_iterations=iterations))
        print(name,flush=True)
    small=root/'single-10';small.write_bytes(b'alpha-body')
    rows.insert(1,dict(name='single-10',path=str(small),files=1,bytes=10,library_iterations=500))
    single=root/'single-64m'
    with single.open('wb') as file:
        for i in range(64):
            file.write(hashlib.shake_256(b'paq-large-file'+i.to_bytes(4,'little')).digest(1024*1024))
    rows.append(dict(name='single-64m',path=str(single),files=1,bytes=64*1024*1024,library_iterations=5))
    mixed=root/'mixed-256';mixed.mkdir()
    sizes=[0,10,1024,1025,32768,163840,1048576,8*1048576]
    for i in range(256):write(mixed/f'f{i:05}',sizes[i%len(sizes)])
    rows.append(dict(name='mixed-256',path=str(mixed),files=256,bytes=32*sum(sizes),library_iterations=3))
    if args.include_go:
        archive=root/'go-pinned.tar.gz'
        urllib.request.urlretrieve(f'https://codeload.github.com/golang/go/tar.gz/{GO_COMMIT}',archive)
        with tarfile.open(archive) as tar:tar.extractall(root,filter='data')
        source=root/f'go-{GO_COMMIT}'
        files=[p for p in source.rglob('*') if p.is_file()]
        rows.append(dict(name='go-pinned',path=str(source),files=len(files),bytes=sum(p.stat().st_size for p in files),library_iterations=3,commit=GO_COMMIT))
    (root/'workloads.json').write_text(json.dumps(rows,indent=2)+'\n')
    print(root/'workloads.json')


if __name__=='__main__':main()
