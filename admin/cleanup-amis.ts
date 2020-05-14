#!/usr/bin/env -S deno run --allow-run

// SPDX-License-Identifier: MIT

// STOP!
// DO NOT RUN THIS SCRIPT UNLESS YOU WANT TO DELETE YOUR
// AWS ACCOUNT SNAPSHOTS AND MACHINE IMAGES!

// Per https://terkwood.farm/tech/aws_cli.html#destroy-your-amis-and-their-snapshots,
// we want to provide auto-cleanup for all images and snapshots.

//
// Helper Functions
//

/** This side-effecting procedure will cause the program to quit 
 * if the subprocess returns a non-zero code.
 */
const runOrExit = async (
  cmd: string[],
  stdout: number | "piped" | "inherit" | "null" | undefined,
) => {
  const p = Deno.run({
    cmd,
    stdout,
  });

  const { code } = await p.status();

  if (code !== 0) {
    const rawError = await p.stderrOutput();
    const errorString = new TextDecoder().decode(rawError);
    console.log(errorString);
    Deno.exit(code);
  }

  return p;
};

const parseProcessOutput = async (p: Deno.Process) =>
  JSON.parse(new TextDecoder().decode(await p.output()));

// Primary Program

const idp = await runOrExit(
  ["/usr/bin/aws", "ec2", "describe-images", "--owners", "self"],
  "piped",
);

const { Images } = await parseProcessOutput(idp);

for (let { ImageId } of Images) {
  await runOrExit(
    ["/usr/bin/aws", "ec2", "deregister-image", "--image-id", ImageId],
    undefined,
  );
}

const dsp = await runOrExit(
  ["/usr/bin/aws", "ec2", "describe-snapshots", "--owner", "self"],
  "piped",
);

const { Snapshots } = await parseProcessOutput(dsp);

for (let { SnapshotId } of Snapshots) {
  await runOrExit(
    ["/usr/bin/aws", "ec2", "delete-snapshot", "--snapshot-id", SnapshotId],
    undefined,
  );
}

Deno.exit(0);
