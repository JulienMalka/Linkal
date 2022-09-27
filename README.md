# Linkal
[![Build Status](https://ci.julienmalka.me/api/badges/JulienMalka/Linkal/status.svg?ref=refs/heads/main)](https://ci.julienmalka.me/JulienMalka/Linkal)
[![made-with-rust](https://img.shields.io/badge/Made%20with-Rust-1f425f.svg)](https://www.rust-lang.org/)
[![built with nix](https://img.shields.io/static/v1?logo=nixos&logoColor=white&label=&message=Built%20with%20Nix&color=41439a)](https://builtwithnix.org)
![GitHub repo size](https://img.shields.io/github/repo-size/JulienMalka/Linkal?label=Size)
![Lines of code](https://img.shields.io/tokei/lines/github/JulienMalka/Linkal?color=26b79b)

Linkal is a public-calendar aggregator server. Given a set a public calendars links, it can make a CalDav client believe all these calendars are part of the same calendar collection. It makes it easy to source public calendars from multiples users and locations and easily distribute them to your end user.


## 🔧 How to build

### ❄ Nix users

```bash
nix build
```

### ❄ 👴🏼 Nix legacy users

```bash
nix-build
```

### 🐳 Docker users

Please be serious.

### Others

Having cargo installed, run ``cargo build``.

## Usage

### Configuration

Linkal is configured using a json file describing your calendars. The file has to follow this structure :
```json
{
  "calendars": {
    "https://calendar1.link": {
      "name": "Calendar 1 name",
    },
    "https://calendar2.link": {
      "name": "Calendar 2 name",
      "color": "#c63b52",
    }
  }
}
```
The calendars have to be **public**. Linkal does not perform any authentification. The ``color`` field is optionnal. If provided, the color of the calendar will be overriden by Linkal. This is useful if several of the calendars you're aggregating have the same color.


An example of configuration file can be found in [examples/calendars.json](https://github.com/JulienMalka/Linkal/tree/main/exemples/calendars.json).

### Running

```bash
linkal --calendar-file <FILE>
```

This command will start the Linkal Server on port ``4145``. It is advised to run Linkal behind a reverse-proxy like Nginx.
