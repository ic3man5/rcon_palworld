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
  -d, --debug_level <LOG_LEVEL_VERBOSITY>
          
  -P, --port <25575>
          Port of the palworld server, defaults to 25575 or 22 if not specified
  -p, --password <PASSWORD>
          Password of the palworld server (RCON or SSH)
  -j, --json
          output in json format
  -l, --list
          Get player name, Unique ID, and SteamID
  -v, --server_version
          Get server version
  -s, --save
          Tell the server to save
  -S, --shutdown <30>
          Tell the server to shutdown with a delay in seconds
  -b, --broadcast <BROADCAST>
          Broadcast a message to the server
  -r, --replace-broadcast-space <REPLACE_BROADCAST_SPACE>
          Broadcast space replacement String
  -c, --command <COMMAND>
          Send a command to the server, result is sent to stdout
  -m, --memory
          Get memory usage of the server
  -M, --memory_ssh
          Get memory usage of the server through SSH
  -u, --username <USERNAME>
          Username to use with an SSH connection
  -h, --help
          Print help
  -V, --version
          Print version
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