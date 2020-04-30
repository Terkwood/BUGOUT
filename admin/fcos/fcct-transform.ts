#!/usr/bin/env deno

const fileName = Deno.args[0];
const fileBase = fileName.split('.')[0];

let command = `docker run  -i --rm quay.io/coreos/fcct:release --pretty --strict <  ${fileName} > ${fileBase}.ign`;

console.log(`Wrote ${fileBase}.ign`);
