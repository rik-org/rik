# Node Metrics

This library is used by riklet to get metrics about the node which is running riklet.

## Getting started

```rs
use node_metrics::Metrics;

fn main() {
    let metrics = Metrics::new();
    let json = metrics.to_json().unwrap();
    println!("{}", json);
}
```

This code prints this kind of json :

```json
{
  "cpu": {
    "total": 8,
    "free": 40.906418
  },
  "memory": {
    "total": 34054199296,
    "free": 24982971392
  },
  "disks": [
    {
      "disk_name": "/dev/nvme0n1p3",
      "total": 496896393216,
      "free": 26035392512
    },
    {
      "disk_name": "/dev/nvme0n1p1",
      "total": 824180736,
      "free": 765476864
    },
    {
      "disk_name": "overlay",
      "total": 496896393216,
      "free": 26035392512
    }
  ]
}
```

## Examples

Code examples can be found [here](examples)

## TODOs

- tests
