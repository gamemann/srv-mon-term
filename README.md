A tool that allows you to monitor game servers inside a terminal. It provides real-time updates on server status, user list, and also monitors latency to the server which supports multiple types of query/packet types (e.g., server queries or ICMP). Will be supporting RCON commands in the future as well!

![Preview as of 7-4-26](./preview/preview01.gif)

## Supported Query Types
| Type | Aliases | Games |
|------|---------|-------|
| `a2s` | `source`, `valve`, `steam`, `goldsrc` | Source/GoldSrc engine servers (CS2, TF2, Garry's Mod, Rust, ...) |
| `quake3` | `q3`, `quake`, `cod`, `cod2`, `cod4`, `codmp`, `et` | Quake 3 engine servers (Call of Duty 1/2/4, Quake 3, OpenArena, Wolfenstein: ET, ...) |
| `minecraft` | `mc`, `minecraft-java`, `java`, `slp` | Minecraft: Java Edition (Server List Ping) |
| `bedrock` | `mcbe`, `mcpe`, `minecraft-bedrock`, `raknet` | Minecraft: Bedrock Edition (RakNet unconnected ping) |
| `gamespy1` | `gs1`, `gamespy` | GameSpy v1 servers (Unreal Tournament, Battlefield 1942, ...) |
| `gamespy3` | `gs3`, `minecraft-query`, `mcquery`, `bf2` | GameSpy v3 servers (Battlefield 2, Minecraft's query listener) |
| `fivem` | `cfx`, `gta5`, `redm`, `citizenfx` | FiveM/RedM (CitizenFX) servers over HTTP |

Run `gmon --list-query-types` to print this list from the program itself.

When `--query` isn't provided, the query type is guessed from the port (e.g., `27015-27030` -> A2S, `25565` -> Minecraft, `19132` -> Bedrock, `28960` -> Quake 3, `30120` -> FiveM).

```bash
# Monitor a Call of Duty 4 server.
gmon -d 192.168.1.10:28960 -q cod4

# Monitor a Minecraft server and remember it.
gmon -d play.example.net:25565 -q minecraft -S

# Print one line per query instead of starting the TUI.
gmon -b -I -d play.example.net:19132 -q bedrock
```

## Adding Servers In The TUI
Servers no longer have to come from the command line. Start the program, then on the dashboard:

| Key | Action |
|-----|--------|
| `n` | Opens the **Add Server** form |
| `↑` `↓` / `Tab` | Moves between fields |
| `←` `→` (or `Space`) | Changes the query type, latency type and save toggle |
| `Enter` | Adds the server |
| `Esc` | Goes back |

The form takes the address, port, query type (left on **Auto** it's guessed from the port), an optional query port, display name, query interval, timeout, latency type and whether to persist the server. The address is checked before the server is added, and adding it starts querying immediately — no restart needed. With **Save** on, the server is written to the store and comes back on the next launch.

Starting without any servers now drops you on an empty dashboard instead of exiting, so a fresh install can be set up entirely from the TUI.

## Building & Installing
You will want to make sure you have Rust and Cargo installed. You can install them from [here](https://www.rust-lang.org/tools/install).

You can clone the project assuming you have `git` installed.

```bash
# Clone the repository
git clone https://github.com/gamemann/gmon.git

# Change directory to the project folder
cd gmon
```

By default, servers are stored using `sqlite` and you will need to have `libsqlite3-dev` installed on your system. You can install it using your package manager. For example, on Ubuntu/Debian you can use `apt`:

```bash
sudo apt install -y libsqlite3-dev
```

To **install** the project, you can use Cargo as well:

```bash
cargo install --path .
```

This should allow you to execute `gmon` from anywhere in your terminal (via the `$PATH`).

## Usage
Servers can be added from the TUI (see [Adding Servers In The TUI](#adding-servers-in-the-tui)) or from the command line with the options below.

| Command | Description | Default |
|---------|-------------| ------- |
| `-s --store` | The storage type to use when persisting server settings. Currently, only `sqlite` is tested, but `json` will be supported in the future. | `sqlite` |
| `--store-path` | The path to the store file without the file extension since that's determined off of the storage type. | `~/.config/gmon/store` |
| `-b --basic` | Uses basic `stdout` messages instead of starting the TUI. Generally used for debugging or minimal output. | - |
| `-v --version` | Displays the current version of the program. | - |
| `-h --help` | Displays the help message. | - |

### General Settings
These are general settings that are stored in whatever store you specify. You can override these settings with command line arguments.

#### Logging
| Command | Description | Default |
|---------|-------------| ------- |
| `l --log` | The log levels to use. Separate multiple levels with a comma. Valid levels are `fatal`, `error`, `warn`, `info`, `debug`, and `trace`. | `info,warn,error,fatal` |
| `-L --log-path` | When specified, all log messages will be written to the specified file as well. Data formats are available! For example: `./logs/%Y-%m-%d.log` | - |
| `-B --log-buffer-size` | The maximum number of log messages to keep in the log buffer before popping out the last one. | `250` |

#### TUI
| Command | Description | Default |
|---------|-------------| ------- |
| `-i --draw-interval` | The interval in milliseconds to redraw the TUI. | `500` |
| `-E --input-poll-interval` | The interval in milliseconds to poll for user input. | `500` |

### Server Settings
Here are general server settings for adding, deleting, or overriding server configurations.

| Command | Description |
|---------|-------------|
| `-d --dst` | The destination IP address or hostname of the server you want to monitor or add/delete. You may also specify a port using the format `IP:PORT`. |
| `-P -port` | The port of the server you want to monitor or add/delete. This is optional if you specify the port in the `--dst` option. |
| `-q --query` | The query type to use when monitoring or adding the server (see [Supported Query Types](#supported-query-types)). If this option is not specified, the program will try to determine the query type based off of the port specified. |
| `-Q --query-port` | The port to use for the query. This is optional and will default to the port specified in `--dst` if not provided. Useful when the query port differs from the port players connect to. |
| `-n --name` | A display name to show for the server instead of the name it reports. |
| `--list-query-types` | Lists every supported query type and exits. |

##### Store Settings
| Command | Description |
|---------|-------------|
| `-S --save` | When set, adds or saves the server or setting to the store. This persists the server or setting for future sessions. |
| `-D --delete` | When set, deletes the server or setting from the store so it will no longer be monitored after the program exits. |

When deleting a server, **only** the destination address and port are required.

#### Overrides
You can specify overrides for a server when adding it to the store. These overrides will be saved and used whenever the server is monitored in the future.

| Command | Description | Default |
|---------|-------------|---------|
| `-t --timeout` | The query timeout in milliseconds. This is the amount of time the program will wait for a response from the server before considering it **Offline**. | `2000` |
| `-c --query-interval` | How often the server is queried, in milliseconds. | `1000` |
| `--latency-type` | How latency is measured: `self-info`, `self-users`, `self-vars` (reuses the timing of the regular query), `query-info`, `query-users`, `query-vars` (sends an extra query) or `icmp`. | `self-info` |
| `--latency-interval` | How often latency is measured, in milliseconds. | Query interval |

## Monitor Settings
You can change how the program monitors servers with the following commands.

| Command | Description |
|---------|-------------|
| `--query-monitor -M` | When set, will only monitor this specific query from the server. The values for this are `info`, `users`, and `vars`. If this option is not set, the program will monitor all queries. |
| `-m --monitor-only` | When set, will only monitor the server query specified above. |
| `-I --isolate` | When set, will only monitor the server specified through the command line. 