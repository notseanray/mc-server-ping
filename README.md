### mc-server-ping

Lightweight and small library to query minecraft server information.

#### usage
```
use mc_server_ping::ServerStatus;

fn main() {
    let mut server = ServerStatus::new("mc.hypixel.net", 25565, None, None);
    server.query().unwrap();
    let response = server.to_json().unwrap();
    println!("{}", serde_json::to_string_pretty(&response).unwrap());
}
```
or
`cargo run --example main`

###### example response
```
{
  "description": "                §aHypixel Network §c[1.8-1.19]\n       §b§lNEW: DROPPER §7§l| §6§lSUMMER EVENT§7§l+§e§lSALE",
  "favicon": "data:image/png;base64...",
  "players": {
    "max": 200000,
    "online": 58389,
    "sample": []
  },
  "version": {
    "name": "Requires MC 1.8 / 1.19",
    "protocol": 47
  }
}
```
With smaller servers the sample array is populated with players and their usernames and UUIDs.

The server does NOT need `enable-query` true in `server.properties`, this fetches the same data as the multiplayer screen.
