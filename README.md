# RIK

A rustlang based cloud orchestrator

## RIK SDN

A Software-defined Networking technology is an approach to network management that enables dynamic, programmatically efficient network configuration in order to improve network performance and monitoring. In this part, we will speak about services that can deserve this component.

### Proposal of services

Our SDN will implement a kind of DHCP server. If you ask him for an IP address, he will give you one available in the network -> A DEFINIR. The SDN has always the IP -> A DEFINIR

### Proposal of architecture

A SDN daemon is present into each node.

### Possible future features

In this part of the `rik` project, we think that it can be very useful to have a `rik` main SDN daemon that manages SDN daemons in nodes. This main daemon could purpose:

- DNS
- Routing between nodes (with encapsulation)

Also, we are questioning ourselves if the RIK main SDN daemon must purpose a load balancer/ingress. We think that it can be a plus for the project.

### Glossary

- SDN: Software Defined Network
- [DHCP](https://en.wikipedia.org/wiki/Dynamic_Host_Configuration_Protocol): Discover Host Configuration Protocol
- node: Another part of the rik project
- pod: A worker in a node
- [DNS](https://en.wikipedia.org/wiki/Domain_Name_System): Domain Name Server
- [encapsulation](https://en.wikipedia.org/wiki/Encapsulation_(computer_programming)): Refers to the bundling of data
- [load balancer](https://en.wikipedia.org/wiki/Load_balancing_(computing)): Refers to the process of distributing a set of tasks over a set of resources
- [ingress](https://kubernetes.io/docs/concepts/services-networking/ingress/): Refers to kube determination, like a load-balancer, but to the applicative OSI layer
