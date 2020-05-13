#!/usr/bin/env deno

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

const idp = Deno.run({
  cmd: ["/usr/bin/aws", "ec2", "describe-images", "--owners", "self"],
  stdout: "piped",
});

const { code: idpCode } = await idp.status();

if (idpCode !== 0) {
  const rawError = await idp.stderrOutput();
  const errorString = new TextDecoder().decode(rawError);
  console.log(errorString);
  Deno.exit(idpCode);
}

const imagesDescribed = new TextDecoder().decode(await idp.output());
console.log(imagesDescribed);
Deno.exit(0);
