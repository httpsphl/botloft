// `BotloftApi` over the daemon's WebSocket.

import type { BotloftApi } from "./api";
import { RpcConnection, type RpcOptions } from "./rpc";

export interface Client extends BotloftApi {
  close(): void;
}

export function connect(options: RpcOptions): Client {
  const connection = new RpcConnection(options);
  connection.start();
  return {
    call: (method, ...params) => connection.request(method, ...params),
    subscribe: (listener) => connection.onNotification(listener),
    connection: () => connection.state,
    onConnection: (listener) => connection.onState(listener),
    close: () => connection.close(),
  };
}

/** Where the app reaches a daemon listening on `port`. */
export function rpcUrl(port: number): string {
  return `ws://127.0.0.1:${port}/rpc`;
}
