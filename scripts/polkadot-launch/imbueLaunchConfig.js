const basePathBase = process.env.POLKADOT_LAUNCH_BASE_PATH_BASE || void 0;
const aliceNodeKey = process.env.NODE_KEY_ALICE;
const bobNodeKey = process.env.NODE_KEY_BOB;

let relaychainBasePort = 30300;
let relaychainBaseRPCPort = 9900;
let relaychainBaseWSPort = 9914;

let parachainBasePort = 31400;
let parachainBaseRPCPort = 9980;
let parachainBaseWSPort = 9942;
const parachainAlicePrometheusPort = 9610;

const commonFlags = [
    "--force-authoring",
    "--unsafe-ws-external",
    "--rpc-cors=all",
    "--rpc-external",
    "--rpc-methods=Unsafe",
];

const relaychainFlags = [
    ...commonFlags,
    "--wasm-execution=Compiled",
];

const parachainNodeFlags = [
    ...commonFlags,
    "--prometheus-external",
    "--allow-private-ipv4",
    "--execution=wasm",
    "--",
    "--prometheus-external",
    "--ws-max-out-buffer-capacity=99999",
    "--ws-max-connections=200"
];

const relaychain = {
    "bin": "/polkadot",
    chain: "rococo-dev",
    nodes: [
        {
            name: "alice",
            wsPort: relaychainBaseWSPort++,
            port: relaychainBasePort++,
            rpcPort: relaychainBaseRPCPort++,
            basePath: basePathBase && `${basePathBase}/alice-relaychain`,
            flags: [
                ...relaychainFlags,
                "--prometheus-external",
            ]
        },
        ...[
            "bob",
            "charlie",
            "dave",
            "eve",
            "ferdie",
        ].map((name, idx) => ({
            name,
            wsPort: relaychainBaseWSPort + idx,
            rpcPort: relaychainBaseRPCPort + idx,
            port: relaychainBasePort + idx,
            basePath: basePathBase && `${basePathBase}/${name}-${idx}-relaychain`,
            flags: [...relaychainFlags]
        }))
    ],
    genesis: {
    }
};

const imbue = {
    bin: "/imbue",
    id: "2121",
    balance: "1000000000000000000000",
    chain: "local",
    nodes: [
        {
            name: "alice",
            
            wsPort: parachainBaseWSPort++,
            port: parachainBasePort++,
            rpcPort: parachainBaseRPCPort++,
            basePath: basePathBase && `${basePathBase}/alice-imbue`,
            flags: [
                `--prometheus-port=${parachainAlicePrometheusPort}`,
                `--node-key=${aliceNodeKey}`,
                ...parachainNodeFlags,
            ]
        },
        ...[
            "bob",
        ].map((name, idx) => ({
            name,
            wsPort: parachainBaseWSPort + idx,
            rpcPort: parachainBaseRPCPort + idx,
            port: parachainBasePort + idx,
            basePath: basePathBase && `${basePathBase}/${name}-${idx}-imbue`,
            flags: [
                `--node-key=${bobNodeKey}`,
                ...parachainNodeFlags
            ]
        }))
    ]
};

module.exports = {
    relaychain,
    parachains: [imbue],
    simpleParachains: [],
    hrmpChannels: [],
    types: {
     
    },
    finalization: false
};