#!/usr/bin/env -S deno run --allow-run --allow-net --allow-read --allow-env
// SPDX-License-Identifier: MIT
import { runOrExit, parseProcessOutput, awsEc2Cmd } from "./procs.ts";
import { config as loadEnv } from "https://deno.land/x/dotenv@v0.3.0/mod.ts";

console.log(loadEnv({ safe: true, export: true }));

const KEY_NAME = Deno.env.get("KEY_NAME");

console.log(`Acting on key ${KEY_NAME}`);

let instsDescd = runOrExit(
  { cmd: awsEc2Cmd("describe-instances"), stdout: "piped" },
);

let addrsDescd = runOrExit(
  { cmd: awsEc2Cmd("describe-addresses"), stdout: "piped" },
);

const { Reservations } = await parseProcessOutput(await instsDescd);

let instancesToTerminate = [];
for (let { Instances } of Reservations) {
  for (let { InstanceId, KeyName } of Instances) {
    if (KEY_NAME === KeyName) {
      instancesToTerminate.push(InstanceId);
    }
  }
}

const { Addresses } = await parseProcessOutput(await addrsDescd);

let addressesToRelease = [];
for (let { InstanceId, AllocationId, AssociationId } of Addresses) {
  if (instancesToTerminate.includes(InstanceId)) {
    addressesToRelease.push({ AllocationId, AssociationId });
  }
}

if (instancesToTerminate.length > 0) {
  console.log(
    `Instances to terminate: ${JSON.stringify(instancesToTerminate)}`,
  );
}

if (addressesToRelease.length > 0) {
  console.log(`Addresses to release  : ${JSON.stringify(addressesToRelease)}`);
}
