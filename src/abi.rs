use ethers::contract::abigen;

abigen!(
    ISwapRouter,
    r#"[{
        "inputs": [{
            "internalType": "address",
            "name": "tokenIn",
            "type": "address"
        }, {
            "internalType": "address",
            "name": "tokenOut",
            "type": "address"
        }, {
            "internalType": "uint24",
            "name": "fee",
            "type": "uint24"
        }, {
            "internalType": "address",
            "name": "recipient",
            "type": "address"
        }, {
            "internalType": "uint256",
            "name": "deadline",
            "type": "uint256"
        }, {
            "internalType": "uint256",
            "name": "amountIn",
            "type": "uint256"
        }, {
            "internalType": "uint256",
            "name": "amountOutMinimum",
            "type": "uint256"
        }, {
            "internalType": "uint160",
            "name": "sqrtPriceLimitX96",
            "type": "uint160"
        }],
        "name": "exactInputSingle",
        "outputs": [{
            "internalType": "uint256",
            "name": "amountOut",
            "type": "uint256"
        }],
        "stateMutability": "payable",
        "type": "function"
    }, {
        "inputs": [{
            "internalType": "bytes",
            "name": "path",
            "type": "bytes"
        }, {
            "internalType": "address",
            "name": "recipient",
            "type": "address"
        }, {
            "internalType": "uint256",
            "name": "deadline",
            "type": "uint256"
        }, {
            "internalType": "uint256",
            "name": "amountIn",
            "type": "uint256"
        }, {
            "internalType": "uint256",
            "name": "amountOutMinimum",
            "type": "uint256"
        }],
        "name": "exactInput",
        "outputs": [{
            "internalType": "uint256",
            "name": "amountOut",
            "type": "uint256"
        }],
        "stateMutability": "payable",
        "type": "function"
    }, {
        "inputs": [{
            "internalType": "address",
            "name": "tokenIn",
            "type": "address"
        }, {
            "internalType": "address",
            "name": "tokenOut",
            "type": "address"
        }, {
            "internalType": "uint24",
            "name": "fee",
            "type": "uint24"
        }, {
            "internalType": "address",
            "name": "recipient",
            "type": "address"
        }, {
            "internalType": "uint256",
            "name": "deadline",
            "type": "uint256"
        }, {
            "internalType": "uint256",
            "name": "amountOut",
            "type": "uint256"
        }, {
            "internalType": "uint256",
            "name": "amountInMaximum",
            "type": "uint256"
        }, {
            "internalType": "uint160",
            "name": "sqrtPriceLimitX96",
            "type": "uint160"
        }],
        "name": "exactOutputSingle",
        "outputs": [{
            "internalType": "uint256",
            "name": "amountIn",
            "type": "uint256"
        }],
        "stateMutability": "payable",
        "type": "function"
    }, {
        "inputs": [{
            "internalType": "bytes",
            "name": "path",
            "type": "bytes"
        }, {
            "internalType": "address",
            "name": "recipient",
            "type": "address"
        }, {
            "internalType": "uint256",
            "name": "deadline",
            "type": "uint256"
        }, {
            "internalType": "uint256",
            "name": "amountOut",
            "type": "uint256"
        }, {
            "internalType": "uint256",
            "name": "amountInMaximum",
            "type": "uint256"
        }],
        "name": "exactOutput",
        "outputs": [{
            "internalType": "uint256",
            "name": "amountIn",
            "type": "uint256"
        }],
        "stateMutability": "payable",
        "type": "function"
    }, {
        "anonymous": false,
        "inputs": [{
            "indexed": true,
            "internalType": "address",
            "name": "tokenIn",
            "type": "address"
        }, {
            "indexed": true,
            "internalType": "address",
            "name": "tokenOut",
            "type": "address"
        }, {
            "indexed": false,
            "internalType": "uint24",
            "name": "fee",
            "type": "uint24"
        }, {
            "indexed": false,
            "internalType": "uint256",
            "name": "amountIn",
            "type": "uint256"
        }, {
            "indexed": false,
            "internalType": "uint256",
            "name": "amountOut",
            "type": "uint256"
        }],
        "name": "ExactInputSingle",
        "type": "event"
    }, {
        "anonymous": false,
        "inputs": [{
            "indexed": false,
            "internalType": "bytes",
            "name": "path",
            "type": "bytes"
        }, {
            "indexed": false,
            "internalType": "uint256",
            "name": "amountIn",
            "type": "uint256"
        }, {
            "indexed": false,
            "internalType": "uint256",
            "name": "amountOut",
            "type": "uint256"
        }],
        "name": "ExactInput",
        "type": "event"
    }, {
        "anonymous": false,
        "inputs": [{
            "indexed": true,
            "internalType": "address",
            "name": "tokenIn",
            "type": "address"
        }, {
            "indexed": true,
            "internalType": "address",
            "name": "tokenOut",
            "type": "address"
        }, {
            "indexed": false,
            "internalType": "uint24",
            "name": "fee",
            "type": "uint24"
        }, {
            "indexed": false,
            "internalType": "uint256",
            "name": "amountOut",
            "type": "uint256"
        }, {
            "indexed": false,
            "internalType": "uint256",
            "name": "amountIn",
            "type": "uint256"
        }],
        "name": "ExactOutputSingle",
        "type": "event"
    }, {
        "anonymous": false,
        "inputs": [{
            "indexed": false,
            "internalType": "bytes",
            "name": "path",
            "type": "bytes"
        }, {
            "indexed": false,
            "internalType": "uint256",
            "name": "amountOut",
            "type": "uint256"
        }, {
            "indexed": false,
            "internalType": "uint256",
            "name": "amountIn",
            "type": "uint256"
        }],
        "name": "ExactOutput",
        "type": "event"
    }]"#
);

abigen!(
    IUniswapV2Router02,
    r#"[{
        "inputs": [],
        "name": "factory",
        "outputs": [{
            "internalType": "address",
            "name": "",
            "type": "address"
        }],
        "stateMutability": "pure",
        "type": "function"
    }, {
        "inputs": [],
        "name": "WETH",
        "outputs": [{
            "internalType": "address",
            "name": "",
            "type": "address"
        }],
        "stateMutability": "pure",
        "type": "function"
    }, {
        "inputs": [{
            "internalType": "address",
            "name": "tokenA",
            "type": "address"
        }, {
            "internalType": "address",
            "name": "tokenB",
            "type": "address"
        }, {
            "internalType": "uint256",
            "name": "amountADesired",
            "type": "uint256"
        }, {
            "internalType": "uint256",
            "name": "amountBDesired",
            "type": "uint256"
        }, {
            "internalType": "uint256",
            "name": "amountAMin",
            "type": "uint256"
        }, {
            "internalType": "uint256",
            "name": "amountBMin",
            "type": "uint256"
        }, {
            "internalType": "address",
            "name": "to",
            "type": "address"
        }, {
            "internalType": "uint256",
            "name": "deadline",
            "type": "uint256"
        }],
        "name": "addLiquidity",
        "outputs": [{
            "internalType": "uint256",
            "name": "amountA",
            "type": "uint256"
        }, {
            "internalType": "uint256",
            "name": "amountB",
            "type": "uint256"
        }, {
            "internalType": "uint256",
            "name": "liquidity",
            "type": "uint256"
        }],
        "stateMutability": "nonpayable",
        "type": "function"
    }, {
        "inputs": [{
            "internalType": "address",
            "name": "token",
            "type": "address"
        }, {
            "internalType": "uint256",
            "name": "amountTokenDesired",
            "type": "uint256"
        }, {
            "internalType": "uint256",
            "name": "amountTokenMin",
            "type": "uint256"
        }, {
            "internalType": "uint256",
            "name": "amountETHMin",
            "type": "uint256"
        }, {
            "internalType": "address",
            "name": "to",
            "type": "address"
        }, {
            "internalType": "uint256",
            "name": "deadline",
            "type": "uint256"
        }],
        "name": "addLiquidityETH",
        "outputs": [{
            "internalType": "uint256",
            "name": "amountToken",
            "type": "uint256"
        }, {
            "internalType": "uint256",
            "name": "amountETH",
            "type": "uint256"
        }, {
            "internalType": "uint256",
            "name": "liquidity",
            "type": "uint256"
        }],
        "stateMutability": "payable",
        "type": "function"
    }, {
        "inputs": [{
            "internalType": "address",
            "name": "tokenA",
            "type": "address"
        }, {
            "internalType": "address",
            "name": "tokenB",
            "type": "address"
        }, {
            "internalType": "uint256",
            "name": "liquidity",
            "type": "uint256"
        }, {
            "internalType": "uint256",
            "name": "amountAMin",
            "type": "uint256"
        }, {
            "internalType": "uint256",
            "name": "amountBMin",
            "type": "uint256"
        }, {
            "internalType": "address",
            "name": "to",
            "type": "address"
        }, {
            "internalType": "uint256",
            "name": "deadline",
            "type": "uint256"
        }],
        "name": "removeLiquidity",
        "outputs": [{
            "internalType": "uint256",
            "name": "amountA",
            "type": "uint256"
        }, {
            "internalType": "uint256",
            "name": "amountB",
            "type": "uint256"
        }],
        "stateMutability": "nonpayable",
        "type": "function"
    }, {
        "inputs": [{
            "internalType": "address",
            "name": "token",
            "type": "address"
        }, {
            "internalType": "uint256",
            "name": "liquidity",
            "type": "uint256"
        }, {
            "internalType": "uint256",
            "name": "amountTokenMin",
            "type": "uint256"
        }, {
            "internalType": "uint256",
            "name": "amountETHMin",
            "type": "uint256"
        }, {
            "internalType": "address",
            "name": "to",
            "type": "address"
        }, {
            "internalType": "uint256",
            "name": "deadline",
            "type": "uint256"
        }],
        "name": "removeLiquidityETH",
        "outputs": [{
            "internalType": "uint256",
            "name": "amountToken",
            "type": "uint256"
        }, {
            "internalType": "uint256",
            "name": "amountETH",
            "type": "uint256"
        }],
        "stateMutability": "nonpayable",
        "type": "function"
    }, {
        "inputs": [{
            "internalType": "uint256",
            "name": "amountIn",
            "type": "uint256"
        }, {
            "internalType": "uint256",
            "name": "amountOutMin",
            "type": "uint256"
        }, {
            "internalType": "address[]",
            "name": "path",
            "type": "address[]"
        }, {
            "internalType": "address",
            "name": "to",
            "type": "address"
        }, {
            "internalType": "uint256",
            "name": "deadline",
            "type": "uint256"
        }],
        "name": "swapExactTokensForTokens",
        "outputs": [{
            "internalType": "uint256[]",
            "name": "amounts",
            "type": "uint256[]"
        }],
        "stateMutability": "nonpayable",
        "type": "function"
    }, {
        "inputs": [{
            "internalType": "uint256",
            "name": "amountOut",
            "type": "uint256"
        }, {
            "internalType": "uint256",
            "name": "amountInMax",
            "type": "uint256"
        }, {
            "internalType": "address[]",
            "name": "path",
            "type": "address[]"
        }, {
            "internalType": "address",
            "name": "to",
            "type": "address"
        }, {
            "internalType": "uint256",
            "name": "deadline",
            "type": "uint256"
        }],
        "name": "swapTokensForExactTokens",
        "outputs": [{
            "internalType": "uint256[]",
            "name": "amounts",
            "type": "uint256[]"
        }],
        "stateMutability": "nonpayable",
        "type": "function"
    }, {
        "inputs": [{
            "internalType": "uint256",
            "name": "amountOutMin",
            "type": "uint256"
        }, {
            "internalType": "address[]",
            "name": "path",
            "type": "address[]"
        }, {
            "internalType": "address",
            "name": "to",
            "type": "address"
        }, {
            "internalType": "uint256",
            "name": "deadline",
            "type": "uint256"
        }],
        "name": "swapExactETHForTokens",
        "outputs": [{
            "internalType": "uint256[]",
            "name": "amounts",
            "type": "uint256[]"
        }],
        "stateMutability": "payable",
        "type": "function"
    }, {
        "inputs": [{
            "internalType": "uint256",
            "name": "amountOut",
            "type": "uint256"
        }, {
            "internalType": "uint256",
            "name": "amountInMax",
            "type": "uint256"
        }, {
            "internalType": "address[]",
            "name": "path",
            "type": "address[]"
        }, {
            "internalType": "address",
            "name": "to",
            "type": "address"
        }, {
            "internalType": "uint256",
            "name": "deadline",
            "type": "uint256"
        }],
        "name": "swapTokensForExactETH",
        "outputs": [{
            "internalType": "uint256[]",
            "name": "amounts",
            "type": "uint256[]"
        }],
        "stateMutability": "nonpayable",
        "type": "function"
    }, {
        "inputs": [{
            "internalType": "uint256",
            "name": "amountIn",
            "type": "uint256"
        }, {
            "internalType": "uint256",
            "name": "amountOutMin",
            "type": "uint256"
        }, {
            "internalType": "address[]",
            "name": "path",
            "type": "address[]"
        }, {
            "internalType": "address",
            "name": "to",
            "type": "address"
        }, {
            "internalType": "uint256",
            "name": "deadline",
            "type": "uint256"
        }],
        "name": "swapExactTokensForETH",
        "outputs": [{
            "internalType": "uint256[]",
            "name": "amounts",
            "type": "uint256[]"
        }],
        "stateMutability": "nonpayable",
        "type": "function"
    }, {
        "inputs": [{
            "internalType": "uint256",
            "name": "amountOut",
            "type": "uint256"
        }, {
            "internalType": "address[]",
            "name": "path",
            "type": "address[]"
        }, {
            "internalType": "address",
            "name": "to",
            "type": "address"
        }, {
            "internalType": "uint256",
            "name": "deadline",
            "type": "uint256"
        }],
        "name": "swapETHForExactTokens",
        "outputs": [{
            "internalType": "uint256[]",
            "name": "amounts",
            "type": "uint256[]"
        }],
        "stateMutability": "payable",
        "type": "function"
    }, {
        "inputs": [{
            "internalType": "uint256",
            "name": "amountA",
            "type": "uint256"
        }, {
            "internalType": "uint256",
            "name": "reserveA",
            "type": "uint256"
        }, {
            "internalType": "uint256",
            "name": "reserveB",
            "type": "uint256"
        }],
        "name": "quote",
        "outputs": [{
            "internalType": "uint256",
            "name": "amountB",
            "type": "uint256"
        }],
        "stateMutability": "pure",
        "type": "function"
    }, {
        "inputs": [{
            "internalType": "uint256",
            "name": "amountIn",
            "type": "uint256"
        }, {
            "internalType": "uint256",
            "name": "reserveIn",
            "type": "uint256"
        }, {
            "internalType": "uint256",
            "name": "reserveOut",
            "type": "uint256"
        }],
        "name": "getAmountOut",
        "outputs": [{
            "internalType": "uint256",
            "name": "amountOut",
            "type": "uint256"
        }],
        "stateMutability": "pure",
        "type": "function"
    }, {
        "inputs": [{
            "internalType": "uint256",
            "name": "amountOut",
            "type": "uint256"
        }, {
            "internalType": "uint256",
            "name": "reserveIn",
            "type": "uint256"
        }, {
            "internalType": "uint256",
            "name": "reserveOut",
            "type": "uint256"
        }],
        "name": "getAmountIn",
        "outputs": [{
            "internalType": "uint256",
            "name": "amountIn",
            "type": "uint256"
        }],
        "stateMutability": "pure",
        "type": "function"
    }, {
        "inputs": [{
            "internalType": "uint256",
            "name": "amountIn",
            "type": "uint256"
        }, {
            "internalType": "address[]",
            "name": "path",
            "type": "address[]"
        }],
        "name": "getAmountsOut",
        "outputs": [{
            "internalType": "uint256[]",
            "name": "amounts",
            "type": "uint256[]"
        }],
        "stateMutability": "view",
        "type": "function"
    }, {
        "inputs": [{
            "internalType": "uint256",
            "name": "amountOut",
            "type": "uint256"
        }, {
            "internalType": "address[]",
            "name": "path",
            "type": "address[]"
        }],
        "name": "getAmountsIn",
        "outputs": [{
            "internalType": "uint256[]",
            "name": "amounts",
            "type": "uint256[]"
        }],
        "stateMutability": "view",
        "type": "function"
    }]"#
);

abigen!(
    IUniswapV2Factory,
    r#"[{
        "inputs": [{
            "internalType": "address",
            "name": "tokenA",
            "type": "address"
        }, {
            "internalType": "address",
            "name": "tokenB",
            "type": "address"
        }],
        "name": "getPair",
        "outputs": [{
            "internalType": "address",
            "name": "pair",
            "type": "address"
        }],
        "stateMutability": "view",
        "type": "function"
    }, {
        "inputs": [{
            "internalType": "uint256",
            "name": "",
            "type": "uint256"
        }],
        "name": "allPairs",
        "outputs": [{
            "internalType": "address",
            "name": "pair",
            "type": "address"
        }],
        "stateMutability": "view",
        "type": "function"
    }, {
        "inputs": [],
        "name": "allPairsLength",
        "outputs": [{
            "internalType": "uint256",
            "name": "",
            "type": "uint256"
        }],
        "stateMutability": "view",
        "type": "function"
    }, {
        "inputs": [{
            "internalType": "address",
            "name": "tokenA",
            "type": "address"
        }, {
            "internalType": "address",
            "name": "tokenB",
            "type": "address"
        }],
        "name": "createPair",
        "outputs": [{
            "internalType": "address",
            "name": "pair",
            "type": "address"
        }],
        "stateMutability": "nonpayable",
        "type": "function"
    }, {
        "anonymous": false,
        "inputs": [{
            "indexed": true,
            "internalType": "address",
            "name": "token0",
            "type": "address"
        }, {
            "indexed": true,
            "internalType": "address",
            "name": "token1",
            "type": "address"
        }, {
            "indexed": false,
            "internalType": "address",
            "name": "pair",
            "type": "address"
        }, {
            "indexed": false,
            "internalType": "uint256",
            "name": "",
            "type": "uint256"
        }],
        "name": "PairCreated",
        "type": "event"
    }]"#
);

abigen!(
    IUniswapV2Pair,
    r#"[{
        "inputs": [],
        "name": "token0",
        "outputs": [{
            "internalType": "address",
            "name": "",
            "type": "address"
        }],
        "stateMutability": "view",
        "type": "function"
    }, {
        "inputs": [],
        "name": "token1",
        "outputs": [{
            "internalType": "address",
            "name": "",
            "type": "address"
        }],
        "stateMutability": "view",
        "type": "function"
    }, {
        "inputs": [],
        "name": "getReserves",
        "outputs": [{
            "internalType": "uint112",
            "name": "reserve0",
            "type": "uint112"
        }, {
            "internalType": "uint112",
            "name": "reserve1",
            "type": "uint112"
        }, {
            "internalType": "uint32",
            "name": "blockTimestampLast",
            "type": "uint32"
        }],
        "stateMutability": "view",
        "type": "function"
    }, {
        "inputs": [],
        "name": "price0CumulativeLast",
        "outputs": [{
            "internalType": "uint256",
            "name": "",
            "type": "uint256"
        }],
        "stateMutability": "view",
        "type": "function"
    }, {
        "inputs": [],
        "name": "price1CumulativeLast",
        "outputs": [{
            "internalType": "uint256",
            "name": "",
            "type": "uint256"
        }],
        "stateMutability": "view",
        "type": "function"
    }, {
        "inputs": [],
        "name": "kLast",
        "outputs": [{
            "internalType": "uint256",
            "name": "",
            "type": "uint256"
        }],
        "stateMutability": "view",
        "type": "function"
    }, {
        "inputs": [{
            "internalType": "address",
            "name": "to",
            "type": "address"
        }],
        "name": "mint",
        "outputs": [{
            "internalType": "uint256",
            "name": "liquidity",
            "type": "uint256"
        }],
        "stateMutability": "nonpayable",
        "type": "function"
    }, {
        "inputs": [{
            "internalType": "address",
            "name": "to",
            "type": "address"
        }],
        "name": "burn",
        "outputs": [{
            "internalType": "uint256",
            "name": "amount0",
            "type": "uint256"
        }, {
            "internalType": "uint256",
            "name": "amount1",
            "type": "uint256"
        }],
        "stateMutability": "nonpayable",
        "type": "function"
    }, {
        "inputs": [{
            "internalType": "uint256",
            "name": "amount0Out",
            "type": "uint256"
        }, {
            "internalType": "uint256",
            "name": "amount1Out",
            "type": "uint256"
        }, {
            "internalType": "address",
            "name": "to",
            "type": "address"
        }, {
            "internalType": "bytes",
            "name": "data",
            "type": "bytes"
        }],
        "name": "swap",
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
    }, {
        "inputs": [{
            "internalType": "address",
            "name": "to",
            "type": "address"
        }],
        "name": "skim",
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
    }, {
        "inputs": [],
        "name": "sync",
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
    }, {
        "anonymous": false,
        "inputs": [{
            "indexed": true,
            "internalType": "address",
            "name": "sender",
            "type": "address"
        }, {
            "indexed": false,
            "internalType": "uint256",
            "name": "amount0In",
            "type": "uint256"
        }, {
            "indexed": false,
            "internalType": "uint256",
            "name": "amount1In",
            "type": "uint256"
        }, {
            "indexed": false,
            "internalType": "uint256",
            "name": "amount0Out",
            "type": "uint256"
        }, {
            "indexed": false,
            "internalType": "uint256",
            "name": "amount1Out",
            "type": "uint256"
        }, {
            "indexed": true,
            "internalType": "address",
            "name": "to",
            "type": "address"
        }],
        "name": "Swap",
        "type": "event"
    }, {
        "anonymous": false,
        "inputs": [{
            "indexed": true,
            "internalType": "address",
            "name": "sender",
            "type": "address"
        }, {
            "indexed": false,
            "internalType": "uint256",
            "name": "amount0",
            "type": "uint256"
        }, {
            "indexed": false,
            "internalType": "uint256",
            "name": "amount1",
            "type": "uint256"
        }],
        "name": "Mint",
        "type": "event"
    }, {
        "anonymous": false,
        "inputs": [{
            "indexed": true,
            "internalType": "address",
            "name": "sender",
            "type": "address"
        }, {
            "indexed": false,
            "internalType": "uint256",
            "name": "amount0",
            "type": "uint256"
        }, {
            "indexed": false,
            "internalType": "uint256",
            "name": "amount1",
            "type": "uint256"
        }, {
            "indexed": true,
            "internalType": "address",
            "name": "to",
            "type": "address"
        }],
        "name": "Burn",
        "type": "event"
    }, {
        "anonymous": false,
        "inputs": [{
            "indexed": false,
            "internalType": "uint112",
            "name": "reserve0",
            "type": "uint112"
        }, {
            "indexed": false,
            "internalType": "uint112",
            "name": "reserve1",
            "type": "uint112"
        }],
        "name": "Sync",
        "type": "event"
    }]"#
);

abigen!(
    IPoolManager,
    r#"[{
        "inputs": [{"internalType": "bytes", "name": "data", "type": "bytes"}],
        "name": "lock",
        "outputs": [{"internalType": "bytes", "name": "", "type": "bytes"}],
        "stateMutability": "nonpayable",
        "type": "function"
    }, {
        "inputs": [
            {"internalType": "address", "name": "currency0", "type": "address"},
            {"internalType": "address", "name": "currency1", "type": "address"},
            {"internalType": "uint24", "name": "fee", "type": "uint24"},
            {"internalType": "int24", "name": "tickSpacing", "type": "int24"},
            {"internalType": "address", "name": "hooks", "type": "address"},
            {"internalType": "uint160", "name": "sqrtPriceX96", "type": "uint160"},
            {"internalType": "bytes", "name": "hookData", "type": "bytes"}
        ],
        "name": "initialize",
        "outputs": [{"internalType": "address", "name": "pool", "type": "address"}],
        "stateMutability": "nonpayable",
        "type": "function"
    }, {
        "inputs": [
            {"internalType": "address", "name": "currency0", "type": "address"},
            {"internalType": "address", "name": "currency1", "type": "address"},
            {"internalType": "uint24", "name": "fee", "type": "uint24"},
            {"internalType": "int24", "name": "tickSpacing", "type": "int24"},
            {"internalType": "address", "name": "hooks", "type": "address"}
        ],
        "name": "getPool",
        "outputs": [{"internalType": "address", "name": "pool", "type": "address"}],
        "stateMutability": "view",
        "type": "function"
    }, {
        "inputs": [
            {"internalType": "address", "name": "currency0", "type": "address"},
            {"internalType": "address", "name": "currency1", "type": "address"},
            {"internalType": "uint24", "name": "fee", "type": "uint24"},
            {"internalType": "int24", "name": "tickSpacing", "type": "int24"},
            {"internalType": "address", "name": "hooks", "type": "address"},
            {"internalType": "bool", "name": "zeroForOne", "type": "bool"},
            {"internalType": "int256", "name": "amountSpecified", "type": "int256"},
            {"internalType": "uint160", "name": "sqrtPriceLimitX96", "type": "uint160"},
            {"internalType": "bytes", "name": "hookData", "type": "bytes"}
        ],
        "name": "swap",
        "outputs": [
            {"internalType": "int256", "name": "amount0", "type": "int256"},
            {"internalType": "int256", "name": "amount1", "type": "int256"}
        ],
        "stateMutability": "nonpayable",
        "type": "function"
    }, {
        "inputs": [
            {"internalType": "address", "name": "currency0", "type": "address"},
            {"internalType": "address", "name": "currency1", "type": "address"},
            {"internalType": "uint24", "name": "fee", "type": "uint24"},
            {"internalType": "int24", "name": "tickSpacing", "type": "int24"},
            {"internalType": "address", "name": "hooks", "type": "address"},
            {"internalType": "int24", "name": "tickLower", "type": "int24"},
            {"internalType": "int24", "name": "tickUpper", "type": "int24"},
            {"internalType": "int256", "name": "liquidityDelta", "type": "int256"},
            {"internalType": "bytes", "name": "hookData", "type": "bytes"}
        ],
        "name": "modifyPosition",
        "outputs": [
            {"internalType": "int256", "name": "amount0", "type": "int256"},
            {"internalType": "int256", "name": "amount1", "type": "int256"}
        ],
        "stateMutability": "nonpayable",
        "type": "function"
    }, {
        "inputs": [
            {"internalType": "address", "name": "currency0", "type": "address"},
            {"internalType": "address", "name": "currency1", "type": "address"},
            {"internalType": "uint24", "name": "fee", "type": "uint24"},
            {"internalType": "int24", "name": "tickSpacing", "type": "int24"},
            {"internalType": "address", "name": "hooks", "type": "address"},
            {"internalType": "uint256", "name": "amount0Desired", "type": "uint256"},
            {"internalType": "uint256", "name": "amount1Desired", "type": "uint256"},
            {"internalType": "bytes", "name": "hookData", "type": "bytes"}
        ],
        "name": "donate",
        "outputs": [
            {"internalType": "int256", "name": "amount0", "type": "int256"},
            {"internalType": "int256", "name": "amount1", "type": "int256"}
        ],
        "stateMutability": "nonpayable",
        "type": "function"
    }, {
        "inputs": [{"internalType": "address", "name": "currency", "type": "address"}],
        "name": "settle",
        "outputs": [{"internalType": "uint256", "name": "paid", "type": "uint256"}],
        "stateMutability": "nonpayable",
        "type": "function"
    }, {
        "inputs": [{"internalType": "address", "name": "currency", "type": "address"}],
        "name": "currencyKey",
        "outputs": [{"internalType": "bytes32", "name": "", "type": "bytes32"}],
        "stateMutability": "pure",
        "type": "function"
    }, {
        "anonymous": false,
        "inputs": [
            {"indexed": true, "internalType": "address", "name": "pool", "type": "address"},
            {"indexed": true, "internalType": "address", "name": "currency0", "type": "address"},
            {"indexed": true, "internalType": "address", "name": "currency1", "type": "address"},
            {"indexed": false, "internalType": "uint24", "name": "fee", "type": "uint24"},
            {"indexed": false, "internalType": "int24", "name": "tickSpacing", "type": "int24"},
            {"indexed": false, "internalType": "address", "name": "hooks", "type": "address"}
        ],
        "name": "PoolInitialized",
        "type": "event"
    }, {
        "anonymous": false,
        "inputs": [
            {"indexed": true, "internalType": "address", "name": "pool", "type": "address"},
            {"indexed": true, "internalType": "address", "name": "sender", "type": "address"},
            {"indexed": false, "internalType": "int256", "name": "amount0", "type": "int256"},
            {"indexed": false, "internalType": "int256", "name": "amount1", "type": "int256"},
            {"indexed": false, "internalType": "uint160", "name": "sqrtPriceX96", "type": "uint160"},
            {"indexed": false, "internalType": "uint128", "name": "liquidity", "type": "uint128"},
            {"indexed": false, "internalType": "int24", "name": "tick", "type": "int24"}
        ],
        "name": "Swap",
        "type": "event"
    }, {
        "anonymous": false,
        "inputs": [
            {"indexed": true, "internalType": "address", "name": "pool", "type": "address"},
            {"indexed": true, "internalType": "address", "name": "owner", "type": "address"},
            {"indexed": false, "internalType": "int24", "name": "tickLower", "type": "int24"},
            {"indexed": false, "internalType": "int24", "name": "tickUpper", "type": "int24"},
            {"indexed": false, "internalType": "int256", "name": "liquidityDelta", "type": "int256"},
            {"indexed": false, "internalType": "int256", "name": "amount0", "type": "int256"},
            {"indexed": false, "internalType": "int256", "name": "amount1", "type": "int256"}
        ],
        "name": "ModifyPosition",
        "type": "event"
    }]"#
);

abigen!(
    IPool,
    r#"[{
        "inputs": [],
        "name": "slot0",
        "outputs": [
            {"internalType": "uint160", "name": "sqrtPriceX96", "type": "uint160"},
            {"internalType": "int24", "name": "tick", "type": "int24"},
            {"internalType": "uint16", "name": "observationIndex", "type": "uint16"},
            {"internalType": "uint16", "name": "observationCardinality", "type": "uint16"},
            {"internalType": "uint16", "name": "observationCardinalityNext", "type": "uint16"},
            {"internalType": "bool", "name": "unlocked", "type": "bool"}
        ],
        "stateMutability": "view",
        "type": "function"
    }, {
        "inputs": [],
        "name": "liquidity",
        "outputs": [{"internalType": "uint128", "name": "", "type": "uint128"}],
        "stateMutability": "view",
        "type": "function"
    }, {
        "inputs": [{"internalType": "int24", "name": "tick", "type": "int24"}],
        "name": "ticks",
        "outputs": [
            {"internalType": "uint128", "name": "liquidityGross", "type": "uint128"},
            {"internalType": "int128", "name": "liquidityNet", "type": "int128"},
            {"internalType": "uint256", "name": "feeGrowthOutside0X128", "type": "uint256"},
            {"internalType": "uint256", "name": "feeGrowthOutside1X128", "type": "uint256"},
            {"internalType": "int56", "name": "tickCumulativeOutside", "type": "int56"},
            {"internalType": "uint160", "name": "secondsPerLiquidityOutsideX128", "type": "uint160"},
            {"internalType": "uint32", "name": "secondsOutside", "type": "uint32"},
            {"internalType": "bool", "name": "initialized", "type": "bool"}
        ],
        "stateMutability": "view",
        "type": "function"
    }, {
        "inputs": [{"internalType": "bytes32", "name": "key", "type": "bytes32"}],
        "name": "positions",
        "outputs": [
            {"internalType": "uint128", "name": "liquidity", "type": "uint128"},
            {"internalType": "uint256", "name": "feeGrowthInside0LastX128", "type": "uint256"},
            {"internalType": "uint256", "name": "feeGrowthInside1LastX128", "type": "uint256"},
            {"internalType": "uint128", "name": "tokensOwed0", "type": "uint128"},
            {"internalType": "uint128", "name": "tokensOwed1", "type": "uint128"}
        ],
        "stateMutability": "view",
        "type": "function"
    }, {
        "inputs": [],
        "name": "protocolFees",
        "outputs": [
            {"internalType": "uint128", "name": "token0", "type": "uint128"},
            {"internalType": "uint128", "name": "token1", "type": "uint128"}
        ],
        "stateMutability": "view",
        "type": "function"
    }, {
        "inputs": [],
        "name": "feeGrowthGlobal0X128",
        "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
        "stateMutability": "view",
        "type": "function"
    }, {
        "inputs": [],
        "name": "feeGrowthGlobal1X128",
        "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
        "stateMutability": "view",
        "type": "function"
    }]"#
);

abigen!(
    IBaseHook,
    r#"[{
        "inputs": [],
        "name": "getHooksCalls",
        "outputs": [{
            "components": [
                {"internalType": "bool", "name": "beforeInitialize", "type": "bool"},
                {"internalType": "bool", "name": "afterInitialize", "type": "bool"},
                {"internalType": "bool", "name": "beforeModifyPosition", "type": "bool"},
                {"internalType": "bool", "name": "afterModifyPosition", "type": "bool"},
                {"internalType": "bool", "name": "beforeSwap", "type": "bool"},
                {"internalType": "bool", "name": "afterSwap", "type": "bool"},
                {"internalType": "bool", "name": "beforeDonate", "type": "bool"},
                {"internalType": "bool", "name": "afterDonate", "type": "bool"}
            ],
            "internalType": "struct Hooks.Calls",
            "name": "",
            "type": "tuple"
        }],
        "stateMutability": "pure",
        "type": "function"
    }]"#
);

abigen!(
    ICurrency,
    r#"[{
        "inputs": [
            {"internalType": "address", "name": "account", "type": "address"},
            {"internalType": "address", "name": "currency", "type": "address"}
        ],
        "name": "balanceOf",
        "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
        "stateMutability": "view",
        "type": "function"
    }, {
        "inputs": [
            {"internalType": "address", "name": "currency", "type": "address"},
            {"internalType": "address", "name": "to", "type": "address"},
            {"internalType": "uint256", "name": "amount", "type": "uint256"}
        ],
        "name": "transfer",
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
    }]"#
);

abigen!(
    IStakingRewards,
    r#"[
        {
            "inputs": [],
            "name": "totalSupply",
            "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [{"internalType": "address", "name": "account", "type": "address"}],
            "name": "balanceOf",
            "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [{"internalType": "address", "name": "account", "type": "address"}],
            "name": "earned",
            "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "getRewardForDuration",
            "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "lastTimeRewardApplicable",
            "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "rewardPerToken",
            "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [{"internalType": "address", "name": "", "type": "address"}],
            "name": "rewards",
            "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [{"internalType": "address", "name": "", "type": "address"}],
            "name": "userRewardPerTokenPaid",
            "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "stakingToken",
            "outputs": [{"internalType": "contract IERC20", "name": "", "type": "address"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "rewardsToken",
            "outputs": [{"internalType": "contract IERC20", "name": "", "type": "address"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "rewardRate",
            "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "rewardsDuration",
            "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "periodFinish",
            "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [{"internalType": "uint256", "name": "amount", "type": "uint256"}],
            "name": "stake",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [{"internalType": "uint256", "name": "amount", "type": "uint256"}],
            "name": "withdraw",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "getReward",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "exit",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "anonymous": false,
            "inputs": [{"indexed": false, "internalType": "uint256", "name": "reward", "type": "uint256"}],
            "name": "RewardAdded",
            "type": "event"
        },
        {
            "anonymous": false,
            "inputs": [{"indexed": true, "internalType": "address", "name": "user", "type": "address"}, {"indexed": false, "internalType": "uint256", "name": "amount", "type": "uint256"}],
            "name": "Staked",
            "type": "event"
        },
        {
            "anonymous": false,
            "inputs": [{"indexed": true, "internalType": "address", "name": "user", "type": "address"}, {"indexed": false, "internalType": "uint256", "name": "amount", "type": "uint256"}],
            "name": "Withdrawn",
            "type": "event"
        },
        {
            "anonymous": false,
            "inputs": [{"indexed": true, "internalType": "address", "name": "user", "type": "address"}, {"indexed": false, "internalType": "uint256", "name": "reward", "type": "uint256"}],
            "name": "RewardPaid",
            "type": "event"
        }
    ]"#
);

abigen!(
    IMasterChef,
    r#"[
        {
            "inputs": [],
            "name": "poolLength",
            "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "totalAllocPoint",
            "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [{"internalType": "uint256", "name": "_from", "type": "uint256"}, {"internalType": "uint256", "name": "_to", "type": "uint256"}],
            "name": "getMultiplier",
            "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [{"internalType": "uint256", "name": "_pid", "type": "uint256"}, {"internalType": "address", "name": "_user", "type": "address"}],
            "name": "pendingReward",
            "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [{"internalType": "uint256", "name": "", "type": "uint256"}, {"internalType": "address", "name": "", "type": "address"}],
            "name": "userInfo",
            "outputs": [{"internalType": "uint256", "name": "amount", "type": "uint256"}, {"internalType": "uint256", "name": "rewardDebt", "type": "uint256"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [{"internalType": "uint256", "name": "_pid", "type": "uint256"}, {"internalType": "uint256", "name": "_amount", "type": "uint256"}],
            "name": "deposit",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [{"internalType": "uint256", "name": "_pid", "type": "uint256"}, {"internalType": "uint256", "name": "_amount", "type": "uint256"}],
            "name": "withdraw",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [{"internalType": "uint256", "name": "_pid", "type": "uint256"}],
            "name": "emergencyWithdraw",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
            "name": "poolInfo",
            "outputs": [
                {"internalType": "contract IERC20", "name": "lpToken", "type": "address"},
                {"internalType": "uint256", "name": "allocPoint", "type": "uint256"},
                {"internalType": "uint256", "name": "lastRewardBlock", "type": "uint256"},
                {"internalType": "uint256", "name": "accRewardPerShare", "type": "uint256"}
            ],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "rewardPerBlock",
            "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "anonymous": false,
            "inputs": [
                {"indexed": true, "internalType": "address", "name": "user", "type": "address"},
                {"indexed": true, "internalType": "uint256", "name": "pid", "type": "uint256"},
                {"indexed": false, "internalType": "uint256", "name": "amount", "type": "uint256"}
            ],
            "name": "Deposit",
            "type": "event"
        },
        {
            "anonymous": false,
            "inputs": [
                {"indexed": true, "internalType": "address", "name": "user", "type": "address"},
                {"indexed": true, "internalType": "uint256", "name": "pid", "type": "uint256"},
                {"indexed": false, "internalType": "uint256", "name": "amount", "type": "uint256"}
            ],
            "name": "Withdraw",
            "type": "event"
        },
        {
            "anonymous": false,
            "inputs": [
                {"indexed": true, "internalType": "address", "name": "user", "type": "address"},
                {"indexed": true, "internalType": "uint256", "name": "pid", "type": "uint256"},
                {"indexed": false, "internalType": "uint256", "name": "amount", "type": "uint256"}
            ],
            "name": "EmergencyWithdraw",
            "type": "event"
        }
    ]"#
);

abigen!(
    IFarmFactory,
    r#"[
        {
            "inputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
            "name": "allFarms",
            "outputs": [{"internalType": "address", "name": "", "type": "address"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "allFarmsLength",
            "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [{"internalType": "address", "name": "stakingToken", "type": "address"}],
            "name": "getFarmsByStakingToken",
            "outputs": [{"internalType": "address[]", "name": "", "type": "address[]"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [
                {"internalType": "address", "name": "stakingToken", "type": "address"},
                {"internalType": "address", "name": "rewardToken", "type": "address"},
                {"internalType": "uint256", "name": "rewardsDuration", "type": "uint256"}
            ],
            "name": "createFarm",
            "outputs": [{"internalType": "address", "name": "", "type": "address"}],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "anonymous": false,
            "inputs": [
                {"indexed": true, "internalType": "address", "name": "farmAddress", "type": "address"},
                {"indexed": true, "internalType": "address", "name": "stakingToken", "type": "address"},
                {"indexed": true, "internalType": "address", "name": "rewardToken", "type": "address"},
                {"indexed": false, "internalType": "uint256", "name": "rewardsDuration", "type": "uint256"}
            ],
            "name": "FarmCreated",
            "type": "event"
        }
    ]"#
);

abigen!(
    IERC20,
    r#"[
        {
            "inputs": [{"internalType": "address", "name": "account", "type": "address"}],
            "name": "balanceOf",
            "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [
                {"internalType": "address", "name": "recipient", "type": "address"},
                {"internalType": "uint256", "name": "amount", "type": "uint256"}
            ],
            "name": "transfer",
            "outputs": [{"internalType": "bool", "name": "", "type": "bool"}],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [
                {"internalType": "address", "name": "spender", "type": "address"},
                {"internalType": "uint256", "name": "amount", "type": "uint256"}
            ],
            "name": "approve",
            "outputs": [{"internalType": "bool", "name": "", "type": "bool"}],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [
                {"internalType": "address", "name": "owner", "type": "address"},
                {"internalType": "address", "name": "spender", "type": "address"}
            ],
            "name": "allowance",
            "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "totalSupply",
            "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "decimals",
            "outputs": [{"internalType": "uint8", "name": "", "type": "uint8"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "symbol",
            "outputs": [{"internalType": "string", "name": "", "type": "string"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "name",
            "outputs": [{"internalType": "string", "name": "", "type": "string"}],
            "stateMutability": "view",
            "type": "function"
        }
    ]"#
);

abigen!(
    IMultiRewardsFarm,
    r#"[
        {
            "inputs": [],
            "name": "totalSupply",
            "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [{"internalType": "address", "name": "account", "type": "address"}],
            "name": "balanceOf",
            "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [{"internalType": "address", "name": "account", "type": "address"}, {"internalType": "address", "name": "rewardToken", "type": "address"}],
            "name": "earned",
            "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [{"internalType": "address", "name": "rewardToken", "type": "address"}],
            "name": "getRewardForDuration",
            "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [{"internalType": "address", "name": "rewardToken", "type": "address"}],
            "name": "lastTimeRewardApplicable",
            "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [{"internalType": "address", "name": "rewardToken", "type": "address"}],
            "name": "rewardPerToken",
            "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [{"internalType": "uint256", "name": "amount", "type": "uint256"}],
            "name": "stake",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [{"internalType": "uint256", "name": "amount", "type": "uint256"}],
            "name": "withdraw",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [{"internalType": "address", "name": "rewardToken", "type": "address"}],
            "name": "getReward",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "getAllRewards",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "exit",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "rewardTokens",
            "outputs": [{"internalType": "address[]", "name": "", "type": "address[]"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "anonymous": false,
            "inputs": [
                {"indexed": true, "internalType": "address", "name": "user", "type": "address"},
                {"indexed": false, "internalType": "uint256", "name": "amount", "type": "uint256"}
            ],
            "name": "Staked",
            "type": "event"
        },
        {
            "anonymous": false,
            "inputs": [
                {"indexed": true, "internalType": "address", "name": "user", "type": "address"},
                {"indexed": false, "internalType": "uint256", "name": "amount", "type": "uint256"}
            ],
            "name": "Withdrawn",
            "type": "event"
        },
        {
            "anonymous": false,
            "inputs": [
                {"indexed": true, "internalType": "address", "name": "user", "type": "address"},
                {"indexed": true, "internalType": "address", "name": "rewardToken", "type": "address"},
                {"indexed": false, "internalType": "uint256", "name": "reward", "type": "uint256"}
            ],
            "name": "RewardPaid",
            "type": "event"
        }
    ]"#
);
