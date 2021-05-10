#!/bin/bash
aws ec2 run-instances --launch-template LaunchTemplateId=$TEMPLATE_ID --image-id $AMI_ID --subnet-id $SUBNET_ID --placement AvailabilityZone=$AZ
