use ethers::types::Address;
use std::str::FromStr;

pub mod base {
    pub mod mainnet {
        pub mod dex {
            pub mod uniswap {
                pub const ROUTER_V2_ADDRESS: &str = "0x4752ba5DBc23f44D87826276BF6Fd6b1C372aD24";
                pub const ROUTER_V3_ADDRESS: &str = "0x2626664c2603336E57B271c5C0b26F421741e481";
                pub const FACTORY_V2_ADDRESS: &str = "0x8909Dc15e40173Ff4699343b6eB8132c65e18eC6";
            }
        }
    }
}

pub mod arb {
    pub mod mainnet {
        pub mod dex {
            pub mod uniswap {
                pub const ROUTER_V2_ADDRESS: &str = "0x1b02dA8Cb0d097eB8D57A175b88c7D8b47997506";
                pub const ROUTER_V3_ADDRESS: &str = "0xE592427A0AEce92De3Edee1F18E0157C05861564";
                pub const FACTORY_V2_ADDRESS: &str = "0xc35DADB65012eC5796536bD9864eD8773aBc74C4";
            }
        }
    }
}

pub mod ethereum {
    pub mod mainnet {
        pub mod dex {
            pub mod uniswap {
                pub const ROUTER_V2_ADDRESS: &str = "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D";
                pub const ROUTER_V3_ADDRESS: &str = "0xE592427A0AEce92De3Edee1F18E0157C05861564";
                pub const FACTORY_V2_ADDRESS: &str = "0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f";
            }
        }
        pub mod token {
            /// mainnet WETH address
            pub const WETH_ADDRESS: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
            /// mainnet USDC address
            pub const USDC_ADDRESS: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
            /// mainnet USDT address
            pub const USDT_ADDRESS: &str = "0xdAC17F958D2ee523a2206206994597C13D831ec7";
            /// mainnet DAI address
            pub const DAI_ADDRESS: &str = "0x6B175474E89094C44Da98b954EedeAC495271d0F";
            /// mainnet WBTC address
            pub const WBTC_ADDRESS: &str = "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599";
        }
    }
}

pub mod bsc {
    pub mod mainnet {
        pub mod dex {
            pub mod uniswap {
                pub const ROUTER_V2_ADDRESS: &str = "0x10ED43C718714eb63d5aA57B78B54704E256024E";
                pub const ROUTER_V3_ADDRESS: &str = "0xB971eF87ede563556b2ED4b1C0b0019111Dd85d2";
                pub const FACTORY_V2_ADDRESS: &str = "0xcA143Ce32Fe78f1f7019d7d551a6402fC5350c73";
            }
        }
    }
}

pub mod polygon {
    pub mod mainnet {
        pub mod tokens {
            pub const WMATIC: &str = "0x0d500B1d8E8eF31E21C99d1Db9A6444d3ADf1270";
            pub const USDC: &str = "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174";
            pub const USDT: &str = "0xc2132D05D31c914a87C6611C10748AEb04B58e8F";
        }
    }
}

pub mod optimism {
    pub mod mainnet {
        pub mod tokens {
            pub const WETH: &str = "0x4200000000000000000000000000000000000006";
            pub const USDC: &str = "0x7F5c764cBc14f9669B88837ca1490cCa17c31607";
        }
    }
}

pub fn parse_address(address_str: &str) -> Result<Address, Box<dyn std::error::Error>> {
    Ok(Address::from_str(address_str)?)
}

pub struct PrecomputedAddresses;

impl PrecomputedAddresses {
    pub fn v2_ethereum_addresses_provider() -> Address {
        Address::from_str("0xB53C1a33016B2DC2fF3653530bfF1848a515c8c5")
            .expect("Invalid V2 Ethereum addresses provider address")
    }

    pub fn v3_ethereum_addresses_provider() -> Address {
        Address::from_str("0x2f39d218133AFaB8F2B819B1066c7E434Ad94E9e")
            .expect("Invalid V3 Ethereum addresses provider address")
    }

    pub fn weth() -> Address {
        Address::from_str("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2")
            .expect("Invalid WETH address")
    }

    pub fn usdc() -> Address {
        Address::from_str("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")
            .expect("Invalid USDC address")
    }
}
