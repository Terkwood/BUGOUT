#!/usr/bin/env -S deno --allow-run

let cmd = [`/usr/bin/docker run  -i --rm quay.io/coreos/fcct:release --pretty --strict `];

let ranCmd = await Deno.run({ cmd });

