Palworld server management
---

How to use command line tool:
---

```
$ ./palworldcli --help
Usage: palworldcli [OPTIONS] --password <PASSWORD> [localhost]

Arguments:
  [localhost]  Host of the palworld server, defaults to localhost if not specified

Options:
  -P, --port <25575>         Port of the palworld server, defaults to 25575 if not specified
  -p, --password <PASSWORD>  Password of the palworld server
  -j, --json                 output in json format
  -l, --list                 Get player name, Unique ID, and SteamID
  -v, --server_version       Get server version
  -h, --help                 Print help
  -V, --version              Print version
```

TODO:
---
- [x] RCON commands
- [ ] Add the rest of the RCON commands 
- [ ] Memory watchdog
- [ ] backup
- [ ] Website
- [ ] Complete License

LICENSE
---
MIT