#!/usr/bin/env deno
let command = "docker run  -i --rm quay.io/coreos/fcct:release --pretty --strict <  $1 > $1.ign";

console.log("Wrote whatever");
