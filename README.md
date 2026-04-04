A simple rust one-off to join a CS2 server, because the steam one is clunky to use

```bash
Usage: cs2-server-joiner [OPTIONS] <IP>

Arguments:
  <IP>  

Options:
      --port <PORT>        Join when less than this amount of players
  -p, --players <PLAYERS>  Join when less than this amount of players
  -c, --check <CHECK>      How often to check if there's a slot available
  -h, --help               Print help
```