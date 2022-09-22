# raspi-rust-api

## Setup

### Environment variables

`.env/.envrc`
```
TIBBER_API_TOKEN=<replaceme>

SHELLY_USERNAME=<replaceme>
SHELLY_PASSWORD=<replaceme>

SHELLY_PLUGS=<name>@<local_ip>,<name>@<local_ip>

TIME_ZONE=<replaceme>
```

### Schedule config

`schedule.json`

`level: "CHEAP" | "NORMAL" | "EXPENSIVE"`


```
[
  {
    "level": "CHEAP",
    "days":["MON", "TUE", "WED", "THU", "FRI"],
    "hours":[
      {"from": "16:00:00", "to": "21:00:00"}
    ]
  },
  ...
```