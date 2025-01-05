# IsOk

IsOk is a tool that allows you to run checks on your infrastructure, and allows you to perform actions based on the 
results of these checks. It is designed to be distributed, and able to have assertions from multiple locations for 
a single check.

## Getting started

You need to have a running Kafka broker in order to proceed. We'll go through the run of the agent and the broker 
in the following sections.

### Broker

It is responsible for receiving the results of the checks, and forwarding them to the kafka cluster. Its
configuration is simple and can be found in [broker.example.yaml](isok-broker/assets/config/broker.example.yaml).

If your kafka cluster has a listener on `9092`, then you can run the example without change:

```bash
cargo run --bin isok-broker -- --config ./isok-broker/assets/config/broker.example.yaml
```

### Agent

It is responsible for running the checks, and sending the results to the broker. Its configuration example
can be found in [agent.example.yaml](isok-agent/assets/config/agent.example.yaml).

```bash
cargo run --bin isok-agent -- --config ./isok-agent/assets/config/agent.example.yaml
```