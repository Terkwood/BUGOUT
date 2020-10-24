#!/usr/bin/env -S deno run --allow-run --allow-net --allow-read --allow-env
// SPDX-License-Identifier: MIT
import { runOrExit, parseProcessOutput, awsEc2Cmd } from "./procs.ts";
import { config as loadEnv } from "https://deno.land/x/dotenv@v0.3.0/mod.ts";

console.log(loadEnv({ safe: true, export: true }));

const KEY_NAME = Deno.env.get("KEY_NAME");

if (!KEY_NAME) {
  console.log("Must set KEY_NAME in env");
  Deno.exit(1);
}

console.log(`Acting on key ${KEY_NAME}`);

let instsDescd = runOrExit({
  cmd: awsEc2Cmd("describe-instances"),
  stdout: "piped",
});

let addrsDescd = runOrExit({
  cmd: awsEc2Cmd("describe-addresses"),
  stdout: "piped",
});

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

if (addressesToRelease.length > 0) {
  console.log(`Addresses to release  : ${JSON.stringify(addressesToRelease)}`);

  for (let { AssociationId, AllocationId } of addressesToRelease) {
    await runOrExit({
      cmd: awsEc2Cmd(`disassociate-address --association-id ${AssociationId}`),
    });

    await runOrExit({
      cmd: awsEc2Cmd(`release-address --allocation-id ${AllocationId}`),
    });
  }
}

if (instancesToTerminate.length > 0) {
  console.log(
    `Instances to terminate: ${JSON.stringify(instancesToTerminate)}`,
  );

  await runOrExit({
    cmd: awsEc2Cmd(
      `terminate-instances --instance-ids ${instancesToTerminate.join(" ")}`,
    ),
  });
}

Deno.exit(0);
