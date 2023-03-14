<p align="center">
  <img src="https://i.imgur.com/22sf4x7.png" />
</p>
<img src="https://img.shields.io/github/actions/workflow/status/rik-org/rik/.github/workflows/rust.yml?branch=main&style=for-the-badge" />

## What is RIK ?

RIK (pronounced /rɪk/) is an experimental workload orchestrator written in Rust 
that aims to be able to natively schedule **containers** and **virtual 
machines** workloads. RIK stands for **R**ust **i**n **K**ubernetes.

## Getting started

Please refer to our [documentation](https://rik-org.github.io/rik/).

### Rationale

Cloud adoption is growing every year. A lot of technologies were created to 
solve issues it brings on. One of the most popular technology when dealing 
with cloud is **containers**. 
As stated by the [CNCF annual survey of 2022](https://www.cncf.io/reports/cncf-annual-survey-2022/), 
containers are the new normal. A lot of companies are using containers in 
production to run their applications because of the portability, scalability
and other benefits they offer. But in a world where elasticity is the key 
for building cost-efficient resilient applications, some tools must be used 
to automate management of the containers. This is where orchestrators like 
[Kubernetes](https://github.com/kubernetes/kubernetes),
[HashiCorp Nomad](https://github.com/hashicorp/nomad) or others comes into play. 

RIK is not a Kubernetes replacement and does not implement the Kubernetes 
APIs.  It is a project mainly for educational purposes, and we want to 
provide  a lightweight and simple code base to **help people understand 
underlying concepts of orchestrators**, or simply **to learn Rust** on a  
project related to a day-to-day issue: **orchestrate cloud applications**.

### Status

**The project is not production ready and should not be used in any 
production systems.** 

We are working to make RIK a simple place to start your cloud orchestrator 
journey. If you are eager to learn how an orchestrator work under the hood 
and to contribute to the project without the hassle of a big code base like 
[Kubernetes](https://github.com/kubernetes/kubernetes), check our 
[contributing section](#contributing).

## Contributing

RIK is open-source and contributions are highly welcome. You will find all 
the guidance to contribute to the project in the [CONTRIBUTING.md](./CONTRIBUTING.md) file.

## Conferences

The project has been presented at the following events :

- **Kubernetes Community Days 2023** | Paris, France | [Thomas Gouveia](https://github.com/thomasgouveia) and [Hugo Amalric](https://github.com/hugoamalric)
- **[DevOps DDay 2022](https://www.youtube.com/watch?v=PS5aUSBdF-I)** | Marseille, France | [Thomas Gouveia](https://github.com/thomasgouveia) and [Hugo Amalric](https://github.com/hugoamalric)

## License

RIK is [Apache2 licensed](./LICENSE).
