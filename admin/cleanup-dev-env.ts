#!/usr/bin/env -S deno run --allow-run --allow-net --allow-read --allow-env
// SPDX-License-Identifier: MIT
import { runOrExit, parseProcessOutput, awsEc2Cmd } from "./procs.ts";
import { config as loadDotEnv } from "https://deno.land/x/dotenv@v0.3.0/mod.ts";

console.log(loadDotEnv({ safe: true }));

let described = await runOrExit(
  { cmd: awsEc2Cmd("describe-instances"), stdout: "piped" },
);

const { Reservations } = await parseProcessOutput(described);

for (let { Instances } of Reservations) {
  for (let { InstanceId, KeyName } of Instances) {
    console.log(`Hello ${InstanceId} ${KeyName}`);
  }
}
