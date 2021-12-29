let relaychainBasePort = 30300;
let relaychainBaseRPCPort = 9900;
let relaychainBaseWSPort = 9914;

let parachainBasePort = 30400;
let parachainBaseRPCPort = 9930;
let parachainBaseWSPort = 9944;

const parachainAlicePrometheusPort = 9610;


const commonFlags = [
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
];

const executable = (name) => {
    const envvar = `${name}_executable`.toUpperCase();
    const exec = process.env[envvar];
    if (exec) {
        return exec;
    }
    throw new Error(`Missing required envvar: ${envvar}`);
}
                           
const relaychain = {
    //"bin": "../polkadot/target/release/polkadot",
    bin: executable("relaychain"),
    chain: "rococo-local",
    nodes: [
        {
            name: "alice",
            wsPort: relaychainBaseWSPort++,
            port: relaychainBasePort++,
            rpcPort: relaychainBaseRPCPort++,
            flags: [
                ...relaychainFlags,
                "--prometheus-external",
            ]
        },
        ...[
            "bob",
            "charlie",
            "dave",
            // "eve",
            // "ferdie"
        ].map((name, idx) => ({
            name,
            wsPort: relaychainBaseWSPort + idx,
            rpcPort: relaychainBaseRPCPort + idx,
            port: relaychainBasePort + idx,
            flags: [...relaychainFlags]
        }))
    ],
    genesis: {
        // runtime: {
        //     runtime_genesis_config: {
        //         configuration: {
        //             config: {
        //                 validation_upgrade_frequency: 1,
        //                 validation_upgrade_delay: 1
        //             }
        //         }
        //     }
        // }
    }
};

const imbue_collator = {
    // bin: "../imbue/target/release/imbue-collator",
    bin: executable("imbue_collator"),
    id: "2102",
    balance: "1000000000000000000000",
    nodes: [
        {
            name: "alice",
            wsPort: parachainBaseWSPort++,
            port: parachainBasePort++,
            rpcPort: parachainBaseRPCPort++,
            flags: [
                `--prometheus-port=${parachainAlicePrometheusPort}`,
                ...parachainNodeFlags,
            ]
        },
        ...[
            "bob",
            "charlie",
            "dave",
            // "eve",
            // "ferdie",
            // // "alice",
            // // "bob"
        ].map((name, idx) => ({
            name,
            wsPort: parachainBaseWSPort + idx,
            rpcPort: parachainBaseRPCPort + idx,
            port: parachainBasePort + idx,
            flags: parachainNodeFlags,
        }))
    ]
};

module.exports = {
    relaychain,
    parachains: [imbue_collator],
    simpleParachains: [],
    hrmpChannels: [],
    types: {
        HrmpChannelId: {
            sender: "u32",
            receiver: "u32"
        }
    },
    finalization: false
};