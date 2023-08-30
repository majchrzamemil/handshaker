# handshaker
The handshaker is a tool used for establishing handshake with provided bitcoin node. Bitcoin protocol defines handshake as shown bellow![btc_handshake](https://github.com/majchrzamemil/handshaker/assets/17731933/23f54a3f-8337-406e-8bd4-a364d4274f4c)
## Configuration
In order to choose a node to connect to user needs to configure config file: `config.json` or provide own file as a command line argument. Structure of config file:
`
{
  "dest_addr": "94.130.79.4:8333",
  "network_type": "main"
}
`<br />
Handshaker allow both IPv4 and IPv6 addrresses. Allowed network types: main, testnet, signet, regtest.<br />
This project has been tested on arm MacOs with nodes from main net and IPv4 addresses.
## Validation of handshake
Successful handshake should result in output: </br>
```
Sending Version message
Message sent
Received: Version message
Sending Verack message
Message sent
Received: Verack message
Hanshake with node: 94.130.79.4:8333 completed
```
Failed hanshake will result with appropriate error message. There is no timeout for sending message so when no output is visible choosen node is inactive, in that case send SIGINT and try again with diffrent node.
