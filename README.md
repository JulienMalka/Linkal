# Linkal
[![Build Status](https://ci.julienmalka.me/api/badges/JulienMalka/Linkal/status.svg?ref=refs/heads/main)](https://ci.julienmalka.me/JulienMalka/Linkal)
[![made-with-rust](https://img.shields.io/badge/Made%20with-Rust-1f425f.svg)](https://www.rust-lang.org/)
[![built with nix](https://img.shields.io/static/v1?logo=nixos&logoColor=white&label=&message=Built%20with%20Nix&color=41439a)](https://builtwithnix.org)
![GitHub repo size](https://img.shields.io/github/repo-size/JulienMalka/Linkal?label=Size)
![Lines of code](https://img.shields.io/tokei/lines/github/JulienMalka/Linkal?color=26b79b)

Linkal is a public-calendar aggregator server. Given a set a public calendars links, it can make a CalDav client believe all these calendars are part of the same calendar collection. It makes it easy to source public calendars from multiples users and locations and easily distribute them to your end user.
It works by exposing the same endpoints as a real CalDav server, emulating responses when needed and otherwise forwarding the requests to the upstream servers.

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

### ⚙️ Configuration

Linkal is configured using a json file describing your calendars. The file has to follow this structure :
```json
{
  "calendars": {
    "https://calendar1.link/public-calendar/path": {
      "name": "Calendar 1 name",
    },
    "https://calendar2.link/public-calendar/path": {
      "name": "Calendar 2 name",
      "color": "#c63b52",
    }
  }
}
```
The calendars have to be **public**. Linkal does not perform any authentification. The ``color`` field is optionnal. If provided, the color of the calendar will be overriden by Linkal. This is useful if several of the calendars you're aggregating have the same color.

### Running

```bash
linkal --calendar-file <FILE>
```

This command will start the Linkal Server on port ``8443``.

### Add a linkal calendar in a CalDav client

To add a linkal aggregated calendar to your client, use as url either the ip of your server with port ``4145`` or the url that is set in your reverse proxy. The id and password that your client is asking can be set with any value.

Supported clients are :
- ✅ Thunderbird
- ✅ Apple calendar

If your calendar client is supported and not on this list, please open a pr/issue. If your favorite calendar client is not supported, open an issue. 

## 🚧 Roadmap

Linkal is in development phase and can be succeptible to bugs. Identified elements for upcoming developments are :
- Parallel requests to answer to /cals requests
- Enable https
- More abstract handling of propfind requests
- More reasonable format of the config file
- Allowing to override more calendar fields 
- Better support of the [RFC 4791](https://datatracker.ietf.org/doc/html/rfc4791) (Long term)
- Filtering protocol, if possible actionnable from the calendar client or a web interface (Long term)


