#!/usr/bin/env -S deno run --allow-run --allow-net --allow-read --allow-env
// SPDX-License-Identifier: MIT
import { runOrExit, parseProcessOutput, awsEc2Cmd } from "./procs.ts";
import { config as loadEnv } from "https://deno.land/x/dotenv@v0.3.0/mod.ts";

console.log(loadEnv({ safe: true, export: true }));

const KEY_NAME = Deno.env.get("KEY_NAME");

console.log(`Acting on ${KEY_NAME}`);

let instsDescd = runOrExit(
  { cmd: awsEc2Cmd("describe-instances"), stdout: "piped" },
);

let addrsDescd = runOrExit(
  { cmd: awsEc2Cmd("describe-addresses"), stdout: "piped" },
);

const { Reservations } = await parseProcessOutput(await instsDescd);

console.log("Instances:");

let instancesToTerminate = [];
for (let { Instances } of Reservations) {
  for (let { InstanceId, KeyName } of Instances) {
    console.log(`\t${InstanceId} ${KeyName}`);
    if (KEY_NAME === KeyName) {
      console.log(`\t...Clean up on ${KeyName}`);
      instancesToTerminate.push(InstanceId);
    }
  }
}

const { Addresses } = await parseProcessOutput(await addrsDescd);

console.log("Addresses:");
let addressesToRelease = [];
for (let { InstanceId, AllocationId, AssociationId } of Addresses) {
  console.log(
    `\t${InstanceId} has alloc ${AllocationId} and assoc ${AssociationId}`,
  );
  if (instancesToTerminate.includes(InstanceId)) {
    addressesToRelease.push({ AllocationId, AssociationId });
  }
}

console.log("\n");
console.log(`Instances to terminate: ${JSON.stringify(instancesToTerminate)}`);
console.log(`Addresses to release  : ${JSON.stringify(addressesToRelease)}`);
