# Tilted

Tilted works with the Tilt Hydrometer to collect homebrew meter
readings and forward them to other services.

It should work on a raspberry pi or any similar device, or and old PC,
as long as it has a compatible bluetooth chip (i.e. 4.0 or
newer). It's designed to be fast and lightweight, while still being
flexible to help you do exactly what you want, and get out of your
way.

# Install
Tilted is written in the rust programming language, and you can install it with cargo:

 cargo install tilted

You can also use the docker image:

 docker pull ozamosi/tilted

# Usage
Running tilted after installing is as simple as `tilted --config
tilted.toml`.

To run it inside docker, you need to run something like the
following - note that `--privileged` is required to give the container
access to your bluetooth device:

 docker run --privileged ozamosi/tilted -m config:/etc/tilted

To run this, you need a config file that defines one or more emitter -
here's an example:
```toml
[log]
emitter = "log"

[brewfather]
emitter = "http"
url = "http://log.brewfather.net/stream?id=xz83XTFteh"

[brewfather.payload]
name = "Tilt {{color}}"
gravity = "{{gravity}}"
temperature = "{{temperature}}"
```

The config file is in [TOML](https://toml.io) format. Each section
defines one emitter. Each emitter type can take different config
options. There are three emitters - `log`, `http`, and `prometheus`.

## Log emitter
The log emitter simply logs info level log messages, which you can use
for either debugging, or for forwarding to a log service.

There are no options.

## Http emitter
The HTTP(s) emitter allows you to format a payload to send to a cloud
service. It takes the following options:
|Name|Required?|Default|Description|Example|
|----|---------|-------|-----------|-------|
|url|✔|N/A|The full URL to send the request to|`url = "http://log.brewfather.net/stream?id=xz83XTFteh"`|
|method| |POST|HTTP method. Probably one of POST, GET, PUT.|`method = "POST"`|
|content-type| |application/json|The content type to send to the server. Note: this does not affect the serialization format, see the `format` key for that.|`content-type = "application/json"`|
|format| |json|The serialisation format. One of `json`, `query` and `form`, for a json encoded body, query parameters, and form encoded body, respectively.|format = "query"|
|min-interval| |5m|The minimum interval to wait between sending data to the service, for rate limiting. The default value is "5m", meaning 5 minutes.|`min-interval="1h5m20s"`|
|payload|✔|N/A|What to put into the payload to send to the server. This is a table where all keys and values are sent through a mustache encoder that has the variables `color`, `gravity` and `temperature` available.|`payload={"device": "tilt", "color": "{{ color }}", "temperature": "{{ temperature }}", "gravity": "{{ gravity }}"}`|

## Prometheus emitter
The prometheus emitter uses the pushgateway to submit metrics. It
associates the color as a label for each metric. It takes the
following options:
|Name|Required?|Default|Description|Example|
|----|---------|-------|-----------|-------|
|address|✔|N/A|The address of the prometheus push gateway, with or without protocol|`address="localhost:9091"`|
|temp-gauge-name|✔|N/A|The gauge name to use for the temperature.|`temp-gauge-name="tilted_temperature_f"`|
|gravity-gauge-name|✔|N/A|The gauge name to use for the gravity.|`gravity-gauge-name="tilted_gravity_sg"`|

# License
Licensed under either of

    Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
    MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

# Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
