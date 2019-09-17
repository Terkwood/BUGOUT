# BUGOUT reaper service

This service is designed to shut down the expensive
compute host used by the JVM services.  It waits
for user activity on kafka to die down, ensures
that an appropriate amount of time has passed,
deems the system inactive, then issues an AWS stop
instance request.

## AWS configuration

Within AWS, the instance(s) that you wish to terminate must have a tag with key `Name` and a given value.  This value must be configured in a `.env` file which will be baked into the docker container build for `reaper`.  See the next section for an example.

You need a role attached to your EC2 instance which gives it the EC2 `StopInstances` and `DescribeInstances` permissions via an IAM policy.  [See AWS documentation for more detail.](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/iam-roles-for-amazon-ec2.html)

You may optionally specify an AWS region in the `.env` file as well.

### Example .env file

```text
INSTANCE_TAG_NAME=Whatever-you-like
AWS_REGION=eu-west-1
```
