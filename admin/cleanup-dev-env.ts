#!/usr/bin/env -S deno run --allow-run --allow-net
// SPDX-License-Identifier: MIT
import { runOrExit, parseProcessOutput, awsEc2Cmd } from "./procs.ts";

let described = await runOrExit(
  { cmd: awsEc2Cmd("describe-instances"), stdout: "piped" },
);

const { Reservations } = await parseProcessOutput(described);

for (let { Instances } of Reservations) {
  for (let { InstanceId } of Instances) {
    console.log(`Hello ${InstanceId}`);
  }
}
