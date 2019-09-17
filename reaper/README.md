# BUGOUT reaper service

This service is designed to shut down the expensive
compute host used by the JVM services.  It waits
for user activity on kafka to die down, ensures
that an appropriate amount of time has passed,
deems the system inactive, then issues an AWS stop
instance request.

## How it works

If there is no activity detected on system streams, the service will eventually shut down.  The system will wait `ALLOWED_IDLE_SECS` before shutting down the specified instance.

The service will search for the `INSTANCE_TAG_NAME` specified in the `.env` file, and terminate all instances with the given `Name` tag.

## AWS configuration

Within AWS, the instance(s) that you wish to terminate must have a tag with key `Name` and a given value.  This value must be configured in a `.env` file which will be baked into the docker container build for `reaper`.  See the next section for an example.

You need a role attached to your EC2 instance which gives it the EC2 `StopInstances` and `DescribeInstances` permissions via an IAM policy.  [See AWS documentation for more detail.](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/iam-roles-for-amazon-ec2.html)

You may optionally specify an AWS region in the `.env` file as well.

## Example .env file

```text
ALLOWED_IDLE_SECS=150
INSTANCE_TAG_NAME=Whatever-you-like
AWS_REGION=eu-west-1
```
