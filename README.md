# RIK
A rustlang based cloud orchestrator

## RIK SDN

A Software-defined Networking technology is an approach to network management that enables dynamic, programmatically efficient network configuration in order to improve network performance and monitoring. In this part, we will speak about services that can deserve this component.

### Propal of services

In this part of rik project, we think that it can be very useful to have this services on a rik main SDN daemon  :

- DHCP
- DNS
- Routing between nodes (with encapsulation)

In addition, we want to add a RIK secondary SDN daemon on each node. This daemon will have a Discovery Service. The communication between the main daemon and secondaries daemon must be encrypted.

Also, we are questioning ourselves if the RIK main SDN daemon must purpose a load balancer/ingress. We think that it can be a plus for the project.

### Propal of architecture

The main SDN will give configuration to the secondary SDN. If the secondary doesn't have the information, he asks the main SDN.
