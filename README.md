# Handshaker

**Handshaker** is a utility designed to streamline the process of establishing a handshake with a provided Bitcoin node. The Bitcoin protocol defines this handshake process, illustrated below:
![btc_handshake](https://github.com/majchrzamemil/handshaker/assets/17731933/23f54a3f-8337-406e-8bd4-a364d4274f4c)

## Configuration

To connect with a specific node, users have the option to either configure the `config.json` file or provide a custom configuration file as a command line argument. The structure of the configuration file is as follows:

```json
{
  "dest_addr": "94.130.79.4:8333",
  "network_type": "main"
}
```

Handshaker supports both IPv4 and IPv6 addresses. Users can select from a variety of allowed network types, including main, testnet, signet, and regtest.

**Note:** This project has been rigorously tested on arm-based macOS systems, utilizing nodes from the main network and IPv4 addresses.

## Handshake Validation

A successful handshake will yield the following output:

```
Sending Version message
Message sent
Received: Version message
Sending Verack message
Message sent
Received: Verack message
Handshake with node: 94.130.79.4:8333 completed
```

In the event of a failed handshake, an appropriate error message will be displayed. It's important to note that there is no timeout for sending messages. If no output is visible, it signifies that the chosen node is inactive. In such cases, send a SIGINT signal and attempt the process again with a different node.
