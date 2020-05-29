#!/usr/bin/env -S deno run --allow-run

// SPDX-License-Identifier: MIT

// STOP!
// DO NOT RUN THIS SCRIPT UNLESS YOU WANT TO DELETE YOUR
// AWS ACCOUNT SNAPSHOTS AND MACHINE IMAGES!

// Per https://terkwood.farm/tech/aws_cli.html#destroy-your-amis-and-their-snapshots,
// we want to provide auto-cleanup for all images and snapshots.

import { runOrExit, parseProcessOutput, awsEc2Cmd } from "./procs.ts";

const idp = await runOrExit({
  cmd: awsEc2Cmd("describe-images --owners self"),
  stdout: "piped",
});

const { Images } = await parseProcessOutput(idp);

for (let { ImageId } of Images) {
  await runOrExit({
    cmd: awsEc2Cmd(`deregister-image --image-id ${ImageId}`),
    stdout: undefined,
  });
}

const dsp = await runOrExit({
  cmd: awsEc2Cmd("describe-snapshots --owner self"),
  stdout: "piped",
});

const { Snapshots } = await parseProcessOutput(dsp);

for (let { SnapshotId } of Snapshots) {
  await runOrExit({
    cmd: awsEc2Cmd(`delete-snapshot --snapshot-id ${SnapshotId}`),
    stdout: undefined,
  });
}

Deno.exit(0);
