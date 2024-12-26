# IsOK Agent

## Getting started

You can run the agent without any external broker, or complex infrastructure.
It is able to run itself and output results to stdout. You just have to provide
the appropriate configuration.

```bash
cargo run --bin isok-agent -- -config-path ./agent.yaml
```

```yaml
# agent.yaml
result_sender_adapter:
  type: "stdout"

check_config_adapter:
  name: "static"
  checks:
    - type: "http"
      pretty_name: "10s google"
      endpoint: "https://google.com"
      interval: 10
    - type: "http"
      pretty_name: "5s failing endpoint"
      endpoint: "https://my_endpoint.com/api/v1/healthy?system_only=true"
      interval: 5

    - type: "tcp"
      pretty_name: "10s tcp fail"
      endpoint: "my_tcp_endpoint:9123"
      interval: 10
```

## Configuration

### Dissociate check file

You dissociate checks configuration from the agent itself:

```yaml
check_config_adapter:
  name: "file"
  path: "asserts/config/checks.example.yml"
```

### Job results to broker

The agent can send job results to a broker, you just have to provide the broker
configuration:

```yaml
result_sender_adapter:
  type: "broker"
  # Target broker configuration, we'd recommend to have at least one fallback broker.
  main_broker: "http://broker.eu-fr-par1.localhost"
  fallback_brokers:
    - "http://broker.us-east-ny1.localhost"

  zone: "dev"
  region: "localhost"

  # Max number of message that can be buffered, if full, send to broker
  batch: 100
  # Maximum interval to which we send a batch
  batch_interval: 10
```

### Job results to stdout

If you don't want to send job results to a broker, you can output them to stdout:

```yaml
result_sender_adapter:
  type: "stdout"
```