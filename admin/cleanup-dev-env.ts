#!/usr/bin/env -S deno run --allow-run --allow-net --allow-read --allow-env
// SPDX-License-Identifier: MIT
import { runOrExit, parseProcessOutput, awsEc2Cmd } from "./procs.ts";
import { config as loadDotEnv } from "https://deno.land/x/dotenv@v0.3.0/mod.ts";

console.log(loadDotEnv({ safe: true }));

const KEY_NAME = Deno.env.get("KEY_NAME");

let instsDescd = runOrExit(
  { cmd: awsEc2Cmd("describe-instances"), stdout: "piped" },
);

let addrsDescd = runOrExit(
  { cmd: awsEc2Cmd("describe-addresses"), stdout: "piped" },
);

const { Reservations } = await parseProcessOutput(await instsDescd);

console.log("Instances:");
for (let { Instances } of Reservations) {
  for (let { InstanceId, KeyName } of Instances) {
    console.log(`\t${InstanceId}`);
    if (KEY_NAME === KeyName) {
      console.log(`\t...Clean up on ${KeyName}`);
    }
  }
}

const { Addresses } = await parseProcessOutput(await addrsDescd);

console.log("Addresses:");
for (let { InstanceId, AllocationId, AssociationId } of Addresses) {
  console.log(
    `\t${InstanceId} has alloc ${AllocationId} and assoc ${AssociationId}`,
  );
}
