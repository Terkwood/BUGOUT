#!/usr/bin/env -S deno run --allow-net --allow-run --allow-read --allow-env
// SPDX-License-Identifier: MIT

import { runOrExit, parseProcessOutput, awsEc2Cmd } from "./procs.ts";
import { config as loadEnv } from "https://deno.land/x/dotenv@v0.3.0/mod.ts";

console.log(loadEnv({ safe: true, export: true }));

// This instance needs to have a tag "Name" with a given
// value, so that we can pick it out from the crowd.
const TAG_NAME = Deno.env.get("TAG_NAME");

let instsDescd = runOrExit({
  cmd: awsEc2Cmd("describe-instances"),
  stdout: "piped",
});

const { Reservations } = await parseProcessOutput(await instsDescd);

let instanceFound = undefined;
for (let { Instances } of Reservations) {
  for (let { InstanceId, Tags } of Instances) {
    for (let { Key, Value } of Tags) {
      if (Key === "Name" && TAG_NAME === Value) {
	      console.log(`hello ${InstanceId} ${Value}`);
	      instanceFound = InstanceId;
      }
    }
  }
}

if (!instanceFound) {
	Deno.exit(1);
}
