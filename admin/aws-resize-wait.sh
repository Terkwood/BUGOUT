#!/bin/bash

aws ec2 stop-instances --instance-ids $INSTANCE_ID 
aws ec2 wait instance-stopped --instance-ids $INSTANCE_ID
aws ec2 modify-instance-attribute --instance-id $INSTANCE_ID --instance-type $INSTANCE_TYPE 
aws ec2 start-instances --instance-ids $INSTANCE_ID
