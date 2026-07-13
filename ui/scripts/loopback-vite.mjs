import { createServer as createHttpServer } from "node:http"

import { createServer as createViteServer } from "vite"

const LOOPBACK_HOST = "127.0.0.1"
const ADDRESS_IN_USE = "EADDRINUSE"
const MAX_BIND_ATTEMPTS = 3

function isAddressInUse(error) {
  return Boolean(
    error &&
      typeof error === "object" &&
      (error.code === ADDRESS_IN_USE || isAddressInUse(error.cause)),
  )
}

function listen(httpServer) {
  return new Promise((resolve, reject) => {
    const onError = (error) => {
      httpServer.off("listening", onListening)
      reject(error)
    }
    const onListening = () => {
      httpServer.off("error", onError)
      const address = httpServer.address()
      if (!address || typeof address === "string") {
        reject(new Error("Loopback server did not publish a TCP port"))
        return
      }
      resolve(address.port)
    }

    httpServer.once("error", onError)
    httpServer.once("listening", onListening)
    httpServer.listen(0, LOOPBACK_HOST)
  })
}

function closeHttpServer(httpServer) {
  return new Promise((resolve, reject) => {
    if (!httpServer.listening) {
      resolve()
      return
    }
    httpServer.close((error) => (error ? reject(error) : resolve()))
  })
}

export async function startLoopbackVite({ root, logLevel = "error" }) {
  for (let attempt = 1; attempt <= MAX_BIND_ATTEMPTS; attempt += 1) {
    const httpServer = createHttpServer()
    let viteServer

    try {
      viteServer = await createViteServer({
        root,
        appType: "spa",
        logLevel,
        server: {
          host: LOOPBACK_HOST,
          middlewareMode: { server: httpServer },
        },
      })
      httpServer.on("request", viteServer.middlewares)
      const port = await listen(httpServer)
      let closed = false

      return {
        url: `http://${LOOPBACK_HOST}:${port}`,
        async close() {
          if (closed) return
          closed = true
          await viteServer.close()
          await closeHttpServer(httpServer)
        },
      }
    } catch (error) {
      await viteServer?.close()
      await closeHttpServer(httpServer)
      if (!isAddressInUse(error) || attempt === MAX_BIND_ATTEMPTS) throw error
    }
  }

  throw new Error("Unable to start the RunHaven loopback server")
}
