rudp-tunnel
===========

An utility to upgrade an UDP connection to reliable UDP by mean of a tunnel relying on the [Aeron](https://github.com/real-logic/aeron) driver.
The command acts as a proxy between the origin and the destination of the initial UDP connection.
It listens on a port on the client side and sends to a given network address on the server side.
The tunnel also works backward, whether the server communicates with the client.

Usage
-----

On both parties the Aeron Media Driver must be running:

    aeronmd

Server-side proxy is the first to launch. It doesn't matter if the service it is proxying has not exposed the destination port yet.

    rudp-tunnel -c CHANNEL -t SERVICE_HOST:SERVICE_PORT

The above command sets the hostname of the Aeron channel (_-c_) and defines the destination of the tunnel to be the network addess of a given service (_-t_).

On the client side the command needs to have a slightly different sintax:

    rudp-tunnel -c CHANNEL -p PORT 

The channel host (_-c_) must be the same as before (usually it is the hostname or the ip address of the computer where the server proxy is running).
The proxy port (_-p_) will define the origin of the tunnel.


Building
--------

Build requires to have Rust and Cargo installed. They can be installed by [rustup](https://rustup.rs/)

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

`rudp-tunnel` can be used to provide a more reliable connection for DOSBox IPXNET, allowing stable gameplay.
Typically, IPXNET binds a server on a given network address, say 192.168.122.127:19900, and the IPXNET clients can connect to such address.
Instead, to create a tunnel one would do as follows.

First the server-side proxy is started. Its public ip is specified as the channel's hostname:

    aeronmd    
    rudp-tunnel -c 192.168.122.127 -t 127.0.0.1:19900

Then, the client-side proxy listens on a given port.

    aeronmd    
    rudp-tunnel -c 192.168.122.127 -p 19901

Afterwards, IPXNET binds a server on the server side, inside DOSBox:

    ipxnet startserver 19900

Finally, on the client side, IPXNET client (always inside DOSBox) connects to the tunneled IPXNET server:

    ipxnet connect 127.0.0.1 19901

License (See LICENSE file for full license)
-------------------------------------------

Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at

https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.