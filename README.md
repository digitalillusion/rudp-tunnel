rudp-tunnel
===========

An utility to upgrade an UDP connection to reliable UDP by mean of a tunnel relying on the [Aeron](https://github.com/real-logic/aeron) driver.
The command acts as a proxy between the origin and the destination of the initial, non-reliable UDP connection.
It listens on a port on the client side and sends to a given network address on the server side.
The tunnel also works backward, whether the server communicates with the client.

Usage
-----

On both parties the Aeron Media Driver must be running:

    aeronmd

Or, if you run the Java version:

    aeron-driver

Server-side proxy is the first to launch. It doesn't matter if the service which is the destination of the tunnel has not exposed its port yet.

    rudp-tunnel -r REMOTE -d SERVICE_HOST:SERVICE_PORT

The above command defines opens a channel toward the remote client and defines a given service as destination of the tunnel.

On the client side, the command opens another channel toward the remote server and defines a given service as origin of the tunnel.
Additionally, it's possible to specify the network interface where to route traffic.

    rudp-tunnel -r REMOTE -i INTERFACE -o SERVICE_HOST:SERVICE_PORT 

*The tunnel operates by default on port 40123, it must be forwarded if you are behind a NAT.*


**Options**

        -h, --help          Show this usage message.
        -p, --fport FPORT   The port on which forward channel operates. Defaults
                            to 40123
        -q, --bport BPORT   The port on which backward channel operates. Defaults
                            to FPORT
        -o, --origin ORIGIN Ip address to bind the client onto, origin of the
                            tunnel. Mutually exclusive with -d
        -d, --destination DESTINATION
                            Ip address where server sends packets, destination of
                            the tunnel. Mutually exclusive with -o
        -r, --remote REMOTE Public network address of the remote side. Defaults to
                            0.0.0.0
        -i, --interface INTERFACE
                            Routing interface. Defaults to 0.0.0.0
        -f, --forward FORWARD
                            Forward channel, client to server.
        -b, --backward BACKWARD
                            Backward channel, server to client.





Building
--------

Build requires to have Rust, Cargo and Cross installed. They can be installed by [rustup](https://rustup.rs/)

**Building a development version**

    cargo build

**Building a Linux release**

    cargo build --release

**Building a Windows release**

    cross build --target x86_64-pc-windows-gnu --release


Example
-------

IPX is a network layer protocol used by DOS games to play on LAN.
Emulators like DOSBox provide an implementation over UDP of such protocol called [IPXNET](https://www.dosbox.com/wiki/Connectivity#IPX_emulation). 
However, since the ethernet has a lower error rate than the UDP protocol on the internet, 
some games may not behave correctly in the presence, for instance, of packet loss.

`rudp-tunnel` can be used to provide a reliable connection for DOSBox IPXNET, allowing stable gameplay.
Typically, IPXNET binds a server on a given network address and the IPXNET clients can connect to such address.
Instead, to create a tunnel one would do as follows.

Before starting, make sure that the Aeron driver is running. 

Start the server-side proxy, open a channel with the remote client and listen on the IPX server address (destination)

    rudp-tunnel -r 97.22.247.211 -d 127.0.0.1:19900

Then, the client-side proxy opens a channel with the remote server on a given routing interface and listens to an IPX connection (origin)  
 
    rudp-tunnel -r 65.53.156.219 -i 192.168.1.208/8 -o 127.0.0.1:19901
 
Afterwards, IPXNET binds a server on the server side, inside DOSBox:

    ipxnet startserver 19900

Finally, on the client side, IPXNET client (always inside DOSBox) connects to the tunneled IPXNET server:

    ipxnet connect 127.0.0.1 19901

License (See LICENSE file for full license)
-------------------------------------------

Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at

https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.