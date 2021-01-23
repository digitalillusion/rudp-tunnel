rudp-tunnel
===========

An utility to upgrade an UDP stream to reliable UDP by mean of a tunnel relying on the [Aeron](https://github.com/real-logic/aeron) driver.
The command acts as a proxy between the origin and the destination of the initial, non-reliable UDP stream.
It listens on a port on the client side and sends to a given network address on the server side.
The tunnel also works backward, whether the server broadcasts to the connected clients.

Usage
-----

Server-side proxy is the first to launch. It doesn't matter if the service which is the endpoint of the tunnel has not exposed its port yet.

    rudp-tunnel -e SERVICE_HOST:SERVICE_PORT

The above command defines a given network address as endpoint of the tunnel.

On the client side, the command opens another channel toward the server and defines a given service as the other endpoint of the tunnel.
Additionally, it's possible to specify the network interface where to route traffic.

    rudp-tunnel -s SERVER -i INTERFACE -e SERVICE_HOST:SERVICE_PORT 

*The tunnel operates on both sides on port 40123 by default. This port must be forwarded if you are behind a NAT.*

At this moment, the services at the two endpoints are able to communicate with each other through the reliable UDP tunnel.


**Options**

        -h, --help          Show this usage message.
        -p, --port PORT     The port on which tunnel operates. Defaults to 40123
        -e, --endpoint ENDPOINT
                            Network address where to send packets, endpoint of the
                            tunnel.
        -s, --server SERVER Public ip address of the server. Defaults to 0.0.0.0
        -i, --interface INTERFACE
                            Routing interface. Defaults to 0.0.0.0
        -d, --driverless    Run without starting Aeron driver, assuming that it
                            has been started already.


Building
--------

Build requires to have Rust, Cargo and Cross installed. They can be installed by [rustup](https://rustup.rs/)

**Building a development version**

    cargo build

**Building a release**

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

Start the server-side proxy, listen on the IPX server address (first endpoint)

    rudp-tunnel -e 127.0.0.1:19900

Then, the client-side proxy connects to the server on a given routing interface and listens to an IPX connection (second endpoint)  
 
    rudp-tunnel -s 65.53.156.219 -i 192.168.1.208/8 -e 127.0.0.1:19901
 
Afterwards, IPXNET binds to the first endpoint. Inside DOSBox, execute:

    ipxnet startserver 19900

Finally, on the client side, IPXNET client (always inside DOSBox) connects to second endpoint:

    ipxnet connect 127.0.0.1 19901

License (See LICENSE file for full license)
-------------------------------------------

Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at

https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.