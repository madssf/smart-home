# smart-home

## Installation

### Prerequisties

- Raspberry Pi connected to network with static IP and the following installed:
  - SSH keys
  - Docker

### Running this

- From another machine:
  `sh transfer.sh`
- On the Raspberry Pi (user root directory):
  `sh run.sh`

## Plug setup

### Shelly Plug S
1. Connect to plug WIFI and go to 192.168.33.1
2. Go to the `Internet & Security` tab.
3. Under `WIFI MODE - CLIENT`, connect the plug to your network and give it a static IP-address.
4. Add a username and password under `RESTRICT LOGIN`
5. Add the plug in the smart-home UI
