rudp-tunnel
===========

An utility to upgrade an UDP stream to reliable UDP by mean of a tunnel relying on the [Aeron](https://github.com/real-logic/aeron) driver.
The command acts as a proxy between the origin and the destination of the initial, non-reliable UDP stream.

Usage
-----

**Embedded driver**

The executable embeds the Java version of the Aeron driver. In order to work correctly, `java` must be available in the path.

**Startup**

Server-side proxy is the first to launch. It doesn't matter if the service which is the endpoint of the tunnel has not exposed its port yet.

    rudp-tunnel -u SERVER -e SERVICE_HOST:SERVICE_PORT -l

The above command accept as parameter the public address of the server and defines a socket address of a service 
as endpoint of the tunnel, using the `-l` flag to specify that it will be listening to the endpoint instead of connecting.

*The tunnel operates by default on port 40123 and uses control port 32104 for [client NAT traversal through multi-destination-cast](http://www.io7m.com/documents/aeron-guide/#weak_nat).
Each time a new client attempts to connect, two ports subsequents to these ones are assigned, up to a defined max number of simultaneously connected clients.
All these ports (by default the ranges from 32104-32114 and 40123-40133) must be opened/forwarded on the firewall/router behind which the server runs.*

On the client side, the command opens a channel toward the server and defines a socket address of a service 
as the other endpoint of the tunnel.
Additionally, it's possible to specify the network interface where to route traffic.

    rudp-tunnel -s SERVER -i INTERFACE -e SERVICE_HOST:SERVICE_PORT

At this moment, the services at the two endpoints are able to communicate with each other through the reliable UDP tunnel.


**Options**

        -h, --help          Show this usage message.
        -p, --port PORT     The port on which tunnel operates. Defaults to 40123
        -c, --control CONTROL
                            The control port used for client NAT traversal.
                            Defaults to 32104
        -e, --endpoint ENDPOINT
                            Socket address where packets are sent/received,
                            endpoint of the tunnel.
        -s, --server SERVER Public ip address of the server, implicitly defining
                            this node as a client. Defaults to 0.0.0.0
        -u, --public PUBLIC Public ip address of this node, starting as server.
                            Ignored if SERVER is specified. Defaults to 0.0.0.0
        -i, --interface INTERFACE
                            Routing interface.
        -m, --mtu MTU       Packets Maximum Transmission Unit. Defaults to 128
                            (bytes)
        -x, --maxclients MAXCLIENTS
                            Maximum number of simultaneously connected clients.
                            Defaults to 10
        -l, --listen        Defines whether to listen on the endpoint socket
                            address instead of connecting
        -d, --driverless    Run without starting Aeron driver, assuming that it
                            has been started externally.
        -n, --nosharedmem   Avoid using shared memory (/dev/shm) under Linux. Has
                            no effect on other platforms.


Building
--------

Build requires to have Rust, Cargo and Cross installed. They can be installed by [rustup](https://rustup.rs/)

**Building a development version**

    cargo build

**Building a release**

    cargo build --release

**Building a Windows release**

    cross build --target x86_64-pc-windows-gnu --release


Examples
--------

IPX is a network layer protocol used by DOS games to play on LAN.
Emulators like DOSBox provide an implementation over UDP of such protocol called [IPXNET](https://www.dosbox.com/wiki/Connectivity#IPX_emulation). 
However, since IPX on ethernet has a lower error rate than UDP on the internet, 
some games may not behave correctly in the presence, for instance, of packet loss.

`rudp-tunnel` can be used to provide a reliable connection for DOSBox IPXNET, allowing stable gameplay.
Typically, IPXNET server binds a port on a given ip address (65.53.156.219 in the example) and 
the IPXNET clients can connect to such address.
Instead, to create a tunnel one would do as follows.

**Client-Server topology**

Start the server-side proxy, listen to the IPXNET server on localhost (first endpoint)

    rudp-tunnel -u 65.53.156.219 -e 127.0.0.1:19900 -l

Then, the client-side proxy connects to the server on a given routing interface (if there is the need to route through one of several available network interfaces) 
and connects to the IPXNET client (second endpoint)  
 
    rudp-tunnel -s 65.53.156.219 -i 192.168.1.0/24 -e 127.0.0.1:19901
 
Afterwards, IPXNET server starts on the first endpoint. Inside DOSBox, execute:

    ipxnet startserver 19900

Finally, on the client side, IPXNET client (always inside DOSBox) connects to second endpoint:

    ipxnet connect 127.0.0.1 19901

**Star topology**

Sometimes it's convenient to have a preconfigured server so that all clients can connect and be part of the same network without having to handle each others ip addresses and port forwarding.
It's possible to run the server side rudp-tunnel so that it does nothing more than redirecting packages. 
In this configuration, one of the client side rudp-tunnel will run as server for the service (IPXNET in this example) and all the others client side rudp-tunnel will be clients for the service instead.

This means that there is no need that server-side rudp-tunnel and service server are both started on the same host. The publicly accessible server-side rudp-tunnel will simply run:

    rudp-tunnel -u 65.53.156.219

One of the client side rudp-tunnel will be designated to act as IPXNET server; note that while connected to the rudp-tunnel server, it's also listening to the local IPXNET server:

    rudp-tunnel -s 65.53.156.219 -i 192.168.1.0/24 -e 0.0.0.0:19900 -l
    
    ipxnet startserver 19900

Meanwhile, all the other rudp-tunnel clients will connect to the rudp-tunnel server and find themself connected to the IPXNET server as well:

    rudp-tunnel -s 65.53.156.219 -i 192.168.1.0/24 -e 127.0.0.1:19901

    ipxnet connect 127.0.0.1 19901


References
----------

This implementation takes wide inspiration from the work described in [Aeron For The Working Programmer](http://www.io7m.com/documents/aeron-guide/). 


License (See LICENSE file for full license)
-------------------------------------------

Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at

https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.