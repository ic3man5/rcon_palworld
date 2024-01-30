Palworld server management
---

How to use command line tool:
---

```
$ ./palworldcli --help
Arguments:
  [localhost]  Host of the palworld server, defaults to localhost if not specified

Options:
  -P, --port <25575>
          Port of the palworld server, defaults to 25575 if not specified
  -p, --password <PASSWORD>
          Password of the palworld server
  -j, --json
          output in json format
  -l, --list
          Get player name, Unique ID, and SteamID
  -v, --server_version
          Get server version
  -s, --save
          Tell the server to save
  -s, --shutdown <30>
          Tell the server to shutdown with a delay in seconds
  -b, --broadcast <BROADCAST>
          Broadcast a message to the server
  -r, --replace-broadcast-space <REPLACE_BROADCAST_SPACE>
          Broadcast space replacement String
  -c, --command <COMMAND>
          Send a command to the server, result is sent to stdout
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