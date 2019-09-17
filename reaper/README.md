# BUGOUT reaper service

This service is designed to shut down the expensive
compute host used by the JVM services.  It waits
for user activity on kafka to die down, ensures
that an appropriate amount of time has passed,
deems the system inactive, then issues an AWS stop
instance request.

## HOW IT WORKS

You need a role attached to your instance which gives you the EC2 `StopInstances` and `DescribeInstances` permissions via .  If you launch this container on a host

```scala
TODO()
TODO()
TODO()
TODO()
TODO()
TODO()
TODO()
TODO()
TODO()
TODO()
TODO()
TODO()
```
