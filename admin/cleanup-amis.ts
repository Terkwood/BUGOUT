#!/usr/bin/env -S deno run --allow-run

// Per https://terkwood.farm/tech/aws_cli.html#destroy-your-amis-and-their-snapshots,
// we want to provide auto-cleanup for all images and snapshots.
// Yes, this is potentially very destructive :-D
/*
~ $ aws ec2 describe-images --owners self|grep ami
            "ImageId": "ami-0cfdf4c0e19074105",
            "ImageId": "ami-0e32350680e92151c",
~ $ aws ec2 deregister-image --image-id ami-0cfdf4c0e19074105
~ $ aws ec2 deregister-image --image-id ami-0e32350680e92151c
~ $ aws ec2 describe-snapshots --owner self | grep snap-
            "SnapshotId": "snap-0c16d85f13ba6ffa3",
            "SnapshotId": "snap-01dcf9767a040352e",
            
~ $ aws ec2 delete-snapshot --snapshot-id snap-0c16d85f13ba6ffa3
~ $ aws ec2 delete-snapshot --snapshot-id snap-01dcf9767a040352e
*/

/** This SIDE-EFFECTING procedure will cause the program to quit 
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
