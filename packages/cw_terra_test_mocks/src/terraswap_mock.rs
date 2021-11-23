use cosmwasm_std::{from_binary, to_binary, Addr, Binary, Empty, Response, StdResult, Uint128};
use cw20::Cw20ReceiveMsg;
use cw20::{BalanceResponse, TokenInfoResponse};
use cw_storage_plus::Map;
use lazy_static::lazy_static;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::RwLock;
use terra_multi_test::{Contract, ContractWrapper};
use terraswap::asset::{Asset, AssetInfo};

// This lazy static use allows you the dev to set the aust token addr before you use the anchor mock so that you can mock out AUST as needed.
lazy_static! {
    // This lazily made static uses a ReadWrite lock to ensure some form of safety on setting/getting values and means you dont need to wrap the code in an unsafe block which looks icky
    static ref TOKEN_ADDR: RwLock<String> = RwLock::new("string".to_string());
}

// Simple mocked instantiate with no params so devs can use it easily 
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MockInstantiateMsg {}

// PingMsg used to give you a quick helper for Receive operations
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PingMsg {
    pub payload: String,
}

// Mocked ExecuteMsg with some CW20 related functions, maybe these are needed at all but it gives you a bigger mock to play with. 
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MockExecuteMsg {
    Receive(Cw20ReceiveMsg),
    Mint {
        recipient: String,
        amount: Uint128,
    },
    Send {
        contract: String,
        amount: Uint128,
        msg: Binary,
    },
    Burn {
        amount: Uint128,
    },
    Transfer {
        recipient: String,
        amount: Uint128,
    },
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PoolResponse {
    pub assets: [Asset; 2],
    pub total_share: Uint128,
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PairResponse {
    pub asset_infos: [AssetInfo; 2],
    pub contract_addr: String,
    pub liquidity_token: String,
}

// Mocked Query handler, containers both Pair and Pool needed for Terraswap
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MockQueryMsg {
    Pair {},
    Pool {},
    TokenInfo {},
    Balance { address: String },
}

pub const BALANCES: Map<&Addr, Uint128> = Map::new("balance");

pub fn contract_terraswap_mock() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        |deps, _, info, msg: MockExecuteMsg| -> StdResult<Response> {
            match msg {
                MockExecuteMsg::Receive(Cw20ReceiveMsg {
                    sender: _,
                    amount: _,
                    msg,
                }) => {
                    let received: PingMsg = from_binary(&msg)?;
                    Ok(Response::new()
                        .add_attribute("action", "pong")
                        .set_data(to_binary(&received.payload)?))
                }
                MockExecuteMsg::Mint {
                    recipient: _,
                    amount: _,
                } => Ok(Response::new()),
                MockExecuteMsg::Send {
                    contract,
                    amount,
                    msg,
                } => Ok(Response::new().add_message(
                    Cw20ReceiveMsg {
                        sender: info.sender.into(),
                        amount,
                        msg,
                    }
                    .into_cosmos_msg(contract)?,
                )),
                MockExecuteMsg::Burn { amount: _ } => Ok(Response::new()),
                MockExecuteMsg::Transfer { recipient, amount } => {
                    let rcpt_addr = deps.api.addr_validate(&recipient)?;
                    BALANCES.update(
                        deps.storage,
                        &rcpt_addr,
                        |balance: Option<Uint128>| -> StdResult<_> {
                            Ok(balance.unwrap_or_default() + amount)
                        },
                    )?;
                    Ok(Response::new()
                        .add_attribute("action", "transfer")
                        .add_attribute("from", info.sender)
                        .add_attribute("to", recipient)
                        .add_attribute("amount", amount))
                }
            }
        },
        |_, _, _, _: MockInstantiateMsg| -> StdResult<Response> { Ok(Response::default()) },
        |_, _, msg: MockQueryMsg| -> StdResult<Binary> {
            match msg {
                MockQueryMsg::Pair {} => Ok(to_binary(&mock_pair_info())?),
                MockQueryMsg::Pool {} => Ok(to_binary(&mock_pool_info())?),
                MockQueryMsg::TokenInfo {} => Ok(to_binary(&mock_token_info())?),
                MockQueryMsg::Balance { address: _ } => Ok(to_binary(&mock_balance_info())?),
            }
        },
    );
    Box::new(contract)
}

// 
// Mocked funcs to return data
// 

// Return a BalanceResponse with dummy data
pub fn mock_balance_info() -> BalanceResponse {
    let resp: BalanceResponse = BalanceResponse {
        balance: Uint128::new(10),
    };
    return resp;
}

// Acquire a write lock on the static value and then update it
pub fn set_liq_token_addr(new_addr: String) -> String {
    let mut addr = TOKEN_ADDR.write().unwrap();
    *addr = new_addr;
    return addr.to_string();
}

pub fn get_liq_token_addr() -> String {
    return TOKEN_ADDR.read().unwrap().to_string();
}

// Return a PairResponse with dummy data
pub fn mock_pair_info() -> PairResponse {
    let resp: PairResponse = PairResponse {
        asset_infos: [
            AssetInfo::NativeToken {
                denom: "uusd".to_string(),
            },
            AssetInfo::NativeToken {
                denom: "uusd".to_string(),
            },
        ],
        contract_addr: "pair0000".to_string(),
        liquidity_token: get_liq_token_addr(),
    };
    return resp;
}

// Return a PoolResponse with dummy data
pub fn mock_pool_info() {
    to_binary(&PoolResponse {
        assets: [
            Asset {
                amount: Uint128::from(10000u128),
                info: AssetInfo::NativeToken {
                    denom: "token".to_string(),
                },
            },
            Asset {
                amount: Uint128::from(10000u128),
                info: AssetInfo::NativeToken {
                    denom: "uusd".to_string(),
                },
            },
        ],
        total_share: Uint128::from(1000u128),
    })
    .unwrap_or_default();
}

// Return a TokenInfoResponse with dummy data
pub fn mock_token_info() -> TokenInfoResponse {
    // TODO: Maybe make these changable via lazy statics 
    let resp: TokenInfoResponse = TokenInfoResponse {
        name: "MyToken".to_string(),
        symbol: "TOKEN".to_string(),
        decimals: 6,
        total_supply: Uint128::from(100_000_000_000_000u128),
    };
    return resp;
}
