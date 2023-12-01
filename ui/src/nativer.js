
export const Nativer = new Proxy({}, {
    get: (_, kind) => {
        return (req) => {
            return new Promise((resolve, reject) => {
                native.bridge.call(JSON.stringify({ kind, req }), (err, res) => {
                    err ? reject(new Error(err)) : resolve(JSON.parse(res)?.res)
                })
            })
        }
    }
})
