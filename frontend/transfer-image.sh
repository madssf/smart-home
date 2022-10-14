docker image save <image>:latest | gzip | pv | ssh <user@host> docker load
