iRacing Websocket
=================

App to get realtime timings and session status and stream it to multiple web clients to view with [iracing-live](https://github.com/LeoAdamek/iracing-live)


Components
----------

### Exporter

The exporter gets the live data from a running iRacing instance on the local machine and forwards the data to the Server.
The exporter only targets win64.


### Server

The server receives telemetry and session data from the exporter and forwards it to all connected clients.
The server supports all std capable targets.


Both components are built in Rust heavily utilizing the Actix actor framework.