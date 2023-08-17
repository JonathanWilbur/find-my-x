# "Find My X" Monorepo

This is a work in progress. This is a set of tools for you to provide your own
"Find My Device"-like service, as provided by iOS and MacOS, except that you can
control your data by running your own server.

## Subprojects

- `fmx-server`: The FindMyX server.
- `fmx-agent`: A desktop agent that periodically pings the server, giving it
  a variety of location data.

In the future, there will be a command-line tool for viewing your location
history, a web app for viewing location history, an Android app, and possibly
an iPhone app. I am also interested in creating agents for IoT devices, but I'm
not sure what that would look like just yet.

## Features

- Record GPS coordinates and velocity
- Record IP addresses
- Record nearby wifi networks and Bluetooth devices
- Record notes
- Announce an emergency
- Remote wipe? (This will depend upon client support, but the server can perform it.)
- Token-based authentication, making it easy to integrate with automation

In the future, there may be more features, such as publicizing where car
accidents or speed traps are, for instance. At such a point, however, this may
cease to be just a "find-my-X" service.

## Uses

- Finding lost or kidnapped people
- Finding missing devices
- Wiping stolen devices
- Silently announcing emergencies
- Home automation (e.g. unlocking your door when you approach your home)

## Implementation

The protocol is defined as a gRPC protocol in `findmydevice.proto`. This means
that the protocol is efficient, fast, and clients can be easily generated in
nearly all major programming languages.

There are really two APIs ("services" in gRPC jargon): the device service and
the user service. The device service is a small and simple API for devices to
transmit location data and listen for requests from the server. The user
service is for reading and purging location history, managing access tokens,
initiating remote wipes, etc. This is split this way to make it easy for
devices to implement the minimal device API, but also allowing clients to be
developed for automation purposes.

Note that the user service does not have to be used by a real human: it may be
used for automation.

There is no persistent storage, currently: all data is only held in memory, but
before 1.0.0, there will be support for a low-latency key-value store, such as
RocksDB or a Rust-based alternative like ReDB.

## Apps / Clients / Agents

I am currently developing a
[cross-platform Android and iOS app](https://github.com/JonathanWilbur/fmx-beacon)
that can submit location data to the FMX server.

## Vision

In the short run, I would like this to be scalable enough for an extended
family to have all of their devices providing updates, but in the long run, I
would like this to become scalable enough that there are a few gigantic servers
on the Internet that people use and trust.

## Development

On a Debian distro, run:

```bash
sudo apt install -y protobuf-compiler libprotobuf-dev
```

(I got these instructions from [here](https://github.com/hyperium/tonic).)