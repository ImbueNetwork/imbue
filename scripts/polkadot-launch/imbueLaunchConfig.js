let relaychainBasePort = 30300;
let relaychainBaseRPCPort = 9900;
let relaychainBaseWSPort = 9940;

const parachainAlicePrometheusPort = 9610;
const parachainAliceRPCPort = 9930;
let parachainBasePort = 30400;
let parachainBaseWSPort = 9950;


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
            flags: [
                ...relaychainFlags,
                "--prometheus-external",
                `--rpc-port=${relaychainBaseRPCPort++}`,
            ]
        },
        ...["bob", "charlie", "dave", "eve", "ferdie"].map((name, idx) => ({
            name,
            wsPort: relaychainBaseWSPort + idx,
            port: relaychainBasePort + idx,
            flags: [
                ...relaychainFlags,
                `--rpc-port=${relaychainBaseRPCPort + idx}`
            ]
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

const parachains = [
    {
        // bin: "../imbue/target/release/imbue-collator",
        bin: executable("imbue_collator"),
        id: "2010",
        balance: "1000000000000000000000",
        nodes: [
            {
                wsPort: parachainBaseWSPort++,
                port: parachainBasePort++,
                name: "alice",
                flags: [
                    `--prometheus-port=${parachainAlicePrometheusPort}`,
                    `--rpc-port=${parachainAliceRPCPort}`,
                    ...parachainNodeFlags,
                ]
            },
            ...[
                "bob",
                "charlie",
                "dave",
                "eve",
                "ferdie",
                "alice",
                "bob"
            ].map((name, idx) => ({
                name,
                wsPort: parachainBaseWSPort + idx,
                port: parachainBasePort + idx,
                flags: parachainNodeFlags,
            }))
        ]
    }
];

module.exports = {
    relaychain,
    parachains,
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