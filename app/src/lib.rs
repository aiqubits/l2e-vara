#![no_std]
use gstd::{collections::HashMap, format, msg, Decode, Encode, String, TypeInfo, Vec};
use sails_rs::prelude::*;

pub type TokenId = U256;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct L2eTop {
    // spenderid -> <(ownerid, vara balance, token balance)> total balance can be mutli stage claim.
    balances: HashMap<ActorId, Vec<(ActorId, u128, U256)>>,
    // ownerid -> <(spenderid, nft tokenid, claimed true/false)>
    nfts: HashMap<ActorId, Vec<(ActorId, TokenId, bool)>>,
    erc20_address: Vec<ActorId>,
    erc721_address: Vec<ActorId>,
    // nft token id num
    token_id_num: U256,
    admin_address: Vec<ActorId>,
    auth_token_owner: Vec<ActorId>,
}

#[derive(Encode, Decode, TypeInfo)]
pub enum Event {
    BalancesAlreadyApproved,
    NoAuthToMintL2ENFT,
    NoAuthToApproveL2EToken,
    TransactionAlreadySend,
    TransactionFailed,
    NoExistVaraApprove,
    NoExistTokenApprove,
    NoExistNFTApprove,
    InsufficientApproveVaras,
    InsufficientApproveTokens,
    InsufficientOwnerDepositTokens,
    NoExistNFT,
    NoClaimedNFT,
    NoAuthorityAddContractAddress,
    NoAuthorityAddAuthTokenOwner,
    AlreadyExistTokenAddress,
    AlreadyExistNFTAddress,
    AlreadyExistAuthAddress,
}

#[service(events = Event)]
impl L2eTop {
    pub fn new(erc20: ActorId, erc721: ActorId) -> Self {
        // only test environment
        // let default_bal: Vec<(AccountId, Balance, Balance)> = Vec::new();
        let default_bal_map = HashMap::new();
        // default_bal_map.insert(AccountId::from([0x00; 32]), &default_bal);

        // let default_nft: Vec<(AccountId, TokenId)> = Vec::new();
        let default_nft_map = HashMap::new();
        // default_nft_map.insert(AccountId::from([0x00; 32]), &default_nft);

        let mut erc20_address: Vec<ActorId> = Vec::new();
        // 5CvYPNqkGfBHnXg4dcq64wH8UPzErkgLi4AxNuvk5kU6PonN
        // let def_erc20 = "5CvYPNqkGfBHnXg4dcq64wH8UPzErkgLi4AxNuvk5kU6PonN";
        // let def_erc20_address = AccountId32::from_ss58check(0def_erc20).unwrap();
        erc20_address.push(erc20);
        let mut erc721_address: Vec<ActorId> = Vec::new();
        // let def_erc721 = "5HiNjbd3BAVwbGFwtZgsZcphi3HZv1yFZqAVWQwASjEARzuC";
        // let def_erc721_address = AccountId32::from_ss58check(def_erc20).unwrap();
        erc721_address.push(erc721);

        let token_id_num = U256::from(10000);

        let self_address = msg::source();
        let mut admin_address: Vec<ActorId> = Vec::new();
        admin_address.push(self_address);

        let mut auth_token_owner: Vec<ActorId> = Vec::new();
        auth_token_owner.push(self_address);

        Self {
            balances: default_bal_map,
            nfts: default_nft_map,
            erc20_address,
            erc721_address,
            token_id_num,
            admin_address,
            auth_token_owner,
        }
    }

    // Service's method (command)
    pub fn get_erc20_address(&self) -> Vec<ActorId> {
        gstd::debug!("erc20_address: {:?}", self.erc20_address.clone());
        self.erc20_address.clone()
    }

    pub fn get_erc721_address(&self) -> Vec<ActorId> {
        gstd::debug!("erc721_address: {:?}", self.erc721_address.clone());
        self.erc721_address.clone()
    }

    pub fn get_admin_address(&self) -> Vec<ActorId> {
        gstd::debug!("erc721_address: {:?}", self.admin_address.clone());
        self.admin_address.clone()
    }

    pub fn get_auth_token_owner_address(&self) -> Vec<ActorId> {
        gstd::debug!("erc721_address: {:?}", self.auth_token_owner.clone());
        self.auth_token_owner.clone()
    }

    // AccountId: spender address
    // bool: if bool is true, then spender already claim nft.
    pub fn get_all_spender_claimed_for_owner(&self) -> Option<Vec<(ActorId, TokenId, bool)>> {
        let owner = msg::source();
        gstd::debug!(
            "get_all_spender_claimed_for_owner self.nfts: {:?}",
            self.nfts
        );
        // let mut claimed_reault: (Vec<(AccountId, TokenId, bool)>, Vec<(AccountId, Balance, Balance)>);
        if self.nfts.contains_key(&owner) {
            let spender_nftid_claim = self.nfts.get(&owner);
            gstd::debug!("spender_nftid_claim: {:?}", spender_nftid_claim);
            if let Some(vecs) = spender_nftid_claim {
                let mut result_vecs = Vec::new();
                for vec in vecs {
                    result_vecs.push(*vec);
                }
                return Some(result_vecs);
            }
        }

        gstd::debug!("get_all_spender_claimed_for_owner over");
        None
    }

    // AccountId: owner address Vec<(AccountId, Balance, Balance)>
    pub fn get_all_owner_rewards_for_spender(&self) -> Option<Vec<(ActorId, u128, U256)>> {
        let spender = msg::source();
        if self.balances.contains_key(&spender) {
            let owner_address = self.balances.get(&spender);
            if let Some(vecs) = owner_address {
                return Some(vecs.iter().map(|&v| (v.0, v.1, v.2)).collect());
            }
        }

        gstd::debug!("get_all_owner_rewards_for_spender over");
        None
    }

    pub fn get_spender_vara_allowances(&self, owner: ActorId) -> Option<u128> {
        let spender = msg::source();

        if self.balances.contains_key(&spender) {
            let balances = self.balances.get(&spender);

            if let Some(vec) = balances {
                let value = vec.iter().find(|&v| v.0 == owner);
                if let Some(v) = value {
                    return Some(v.1);
                }
            }
        }
        None
    }

    pub async fn get_spender_token_allowances(
        &self,
        owner: ActorId,
        erc20_num: u32,
    ) -> Option<U256> {
        let spender = msg::source();

        let mut current_erc20 = self.erc20_address[0];

        if (self.erc20_address.len() as u32)
            > erc20_num.checked_add(1).expect("Failed to add erc20_num")
        {
            current_erc20 = self.erc20_address[erc20_num as usize];
        }

        if self.balances.contains_key(&spender) {
            // cross contract call
            let call_payload = counter::io::allowance::encode_call();
            let reply_bytes = gstd::msg::send_bytes_for_reply(current_erc20, call_payload, 0, 0)
                .expect("Failed to send_bytes_for_reply")
                .await
                .expect("Failed to send_bytes_for_reply await");
            let reply =
                <counter::io::allowance as sails_rs::calls::ActionIo>::decode_reply(&reply_bytes)
                    .expect("Failed to decode_reply");
            return Some(reply);

            // let balances = build_call::<DefaultEnvironment>()
            //     // ERC20 address, gas_limit must be some value when in mainnet
            //     .call(current_erc20)
            //     .call_v1()
            //     .gas_limit(0)
            //     .transferred_value(0)
            //     .exec_input(
            //         ExecutionInput::new(Selector::new(ink::selector_bytes!("allowance")))
            //             .push_arg(owner)
            //             .push_arg(spender),
            //     )
            //     .returns::<Balance>()
            //     .try_invoke()
            //     .expect("Failed to get_spender_token_allowances");
            //     // .map_err(|e| format!("Failed to get_spender_token_allowances: {:?}", e));

            // // ink::env::debug_println!("get_spender_token_allowances balances:{:?}", balances);

            // if let Ok(value) = balances {
            //     return Some(value);
            // }
        }
        None
    }

    pub fn get_spender_nft_allowances(&self, owner: ActorId) -> Option<TokenId> {
        let spender = msg::source();
        gstd::debug!("get_spender_nft_allowances self.nfts: {:?}", self.nfts);
        if self.nfts.contains_key(&owner) {
            let nfts = self.nfts.get(&owner);
            gstd::debug!("nfts: {:?}", nfts);
            if let Some(vec) = nfts {
                let value = vec.iter().find(|&v| v.0 == spender);
                gstd::debug!("value: {:?}", value);
                if let Some(v) = value {
                    return Some(v.1);
                }
            }
        }
        None
    }

    pub async fn approve_balances(
        &mut self,
        spender: ActorId,
        erc20_num: u32,
        vara_value: U256,
        token_value: U256,
    ) -> Option<(u128, U256)> {
        let owner = msg::source();
        let mut current_value: u128 = 0;
        // vara_value should be transfer value, msg::value() acutal value.
        // frontend control vara_value == msg::value()
        if vara_value > U256::from(0) {
            current_value = msg::value();
        }
        gstd::debug!("current_value-{}", current_value);
        gstd::debug!("token_value:{}", token_value);
        if token_value > U256::from(0) {
            gstd::debug!("token_value>0");
            let mut current_erc20 = self.erc20_address[0];

            if (self.erc20_address.len() as u32)
                > erc20_num.checked_add(1).expect("Failed to add erc20_num")
            {
                current_erc20 = self.erc20_address[erc20_num as usize];
            } else {
                // check auth_token_owner role
                if !self.auth_token_owner.contains(&owner) {
                    let _ = self.notify_on(Event::NoAuthToApproveL2EToken);
                    panic!("NoAuthToApproveL2EToken");
                }
            }

            gstd::debug!("current_erc20:{:?}", current_erc20);
            gstd::debug!("current_erc20:{:?}", self.erc20_address);

            // cross contract call
            let call_payload = counter::io::balanceOf::encode_call();
            let reply_bytes = gstd::msg::send_bytes_for_reply(current_erc20, call_payload, 0, 0)
                .expect("Failed to send_bytes_for_reply")
                .await
                .expect("Failed to send_bytes_for_reply await");
            let result_balance_of =
                <counter::io::balanceOf as sails_rs::calls::ActionIo>::decode_reply(&reply_bytes)
                    .expect("Failed to decode_reply");

            // let result_balance_of = build_call::<DefaultEnvironment>()
            //     // ERC20 address, gas_limit must be some value when in mainnet
            //     .call(current_erc20)
            //     .call_v1()
            //     .gas_limit(0)
            //     .transferred_value(0)
            //     .exec_input(
            //         ExecutionInput::new(Selector::new(ink::selector_bytes!("balanceOf")))
            //             .push_arg(owner),
            //     )
            //     .returns::<Balance>()
            //     .try_invoke()
            //     .expect("Failed to get result_balance_of");
            //     // .map_err(|e| format!("approve_balances failed: {:?}", e));

            gstd::debug!("result_balance_of error:{:?}", result_balance_of);

            if let Ok(balance_of) = result_balance_of {
                if token_value > balance_of / 1000 {
                    let _ = self.notify_on(Event::InsufficientOwnerDepositTokens);
                    panic!("InsufficientOwnerDepositTokens");
                }
            }

            // cross contract call
            let call_payload = counter::io::approve::encode_call();
            let reply_bytes = gstd::msg::send_bytes_for_reply(current_erc20, call_payload, 0, 0)
                .expect("Failed to send_bytes_for_reply")
                .await
                .expect("Failed to send_bytes_for_reply await");
            let result_approve =
                <counter::io::approve as sails_rs::calls::ActionIo>::decode_reply(&reply_bytes)
                    .expect("Failed to decode_reply");

            // let result_approve = build_call::<DefaultEnvironment>()
            //     // ERC20 address, gas_limit must be some value when in mainnet
            //     .call(current_erc20)
            //     .call_v1()
            //     .gas_limit(0)
            //     .transferred_value(0)
            //     .exec_input(
            //         ExecutionInput::new(Selector::new(ink::selector_bytes!("approve")))
            //             .push_arg(spender)
            //             .push_arg(token_value),
            //     )
            //     .returns::<Result<(), Error>>()
            //     .try_invoke()
            //     .map_err(|e| format!("approve_balances failed: {:?}", e));

            gstd::debug!("result_approve error:{:?}", result_approve);
        }
        gstd::debug!("token_value  over");
        if self.balances.contains_key(&spender) {
            gstd::debug!("self.balances.contains(spender)");
            let mut owner_value = self
                .balances
                .get_mut(&spender)
                .expect("failed to take owner value");
            gstd::debug!("owner_value::{:?}", owner_value);
            if owner_value.iter().any(|&(o, _, _)| o == owner) {
                let _ = self.notify_on(Event::BalancesAlreadyApproved);
                panic!("BalancesAlreadyApproved");
            }

            owner_value.push((owner, current_value, token_value));
            self.balances.insert(spender, owner_value.clone());
        } else {
            let mut owner_value = Vec::new();
            owner_value.push((owner, current_value, token_value));
            self.balances.insert(spender, owner_value);
            gstd::debug!("owner_value--{:?}", owner_value);
        }
        gstd::debug!("owner_value--over");
        Some((current_value, token_value))
    }

    pub async fn mint_approve_nft(&mut self, erc721_num: u32, spender: ActorId) -> bool {
        let owner = msg::source();

        // tokenid u32
        self.token_id_num = self
            .token_id_num
            .checked_add(U256::from(1))
            .expect("Failed to create token_id");
        let token_id: TokenId = self.token_id_num;

        let mut current_erc721 = self.erc721_address[0];
        if (self.erc721_address.len() as u32)
            > erc721_num.checked_add(1).expect("Failed to add erc721_num")
        {
            current_erc721 = self.erc721_address[erc721_num as usize];
        } else {
            // check auth_token_owner role
            if !self.auth_token_owner.contains(&owner) {
                let _ = self.notify_on(Event::NoAuthToMintL2ENFT);
                panic!("NoAuthToMintL2ENFT");
            }
        }

        gstd::debug!("current_erc721:{:?}", current_erc721);
        gstd::debug!("token_id:{:?}", token_id);
        // cross contract call
        // call ERC721 mint function
        let call_payload = counter::io::mint::encode_call();
        let reply_bytes = gstd::msg::send_bytes_for_reply(current_erc721, call_payload, 0, 0)
            .expect("Failed to send_bytes_for_reply")
            .await
            .expect("Failed to send_bytes_for_reply await");
        let mint_nft =
            <counter::io::allowance as sails_rs::calls::ActionIo>::decode_reply(&reply_bytes)
                .expect("Failed to decode_reply");

        // let mint_approve_nft = build_call::<DefaultEnvironment>()
        //     // ERC721
        //     .call(current_erc721)
        //     .call_v1()
        //     .gas_limit(0)
        //     .transferred_value(0)
        //     .exec_input(
        //         ExecutionInput::new(Selector::new(ink::selector_bytes!("mint")))
        //             .push_arg(token_id),
        //     )
        //     .returns::<Result<(), Error>>()
        //     .try_invoke()
        //     // .map_err(|_| Error::FailedMintNFT)?;
        //     .map_err(|e| format!("mint_approve_nft failed: {:?}", e));

        gstd::debug!("mint_approve_nft error:{:?}", mint_nft);

        // cross contract call
        // call ERC721 approve function
        let call_payload = counter::io::approve::encode_call();
        let reply_bytes = gstd::msg::send_bytes_for_reply(current_erc721, call_payload, 0, 0)
            .expect("Failed to send_bytes_for_reply")
            .await
            .expect("Failed to send_bytes_for_reply await");
        let approve_nft =
            <counter::io::approve as sails_rs::calls::ActionIo>::decode_reply(&reply_bytes)
                .expect("Failed to decode_reply");

        // let approve_nft = build_call::<DefaultEnvironment>()
        //     // vara ERC721 address
        //     .call(current_erc721)
        //     .call_v1()
        //     .gas_limit(0)
        //     .transferred_value(0)
        //     .exec_input(
        //         ExecutionInput::new(Selector::new(ink::selector_bytes!("approve")))
        //             .push_arg(spender)
        //             .push_arg(token_id),
        //     )
        //     .returns::<Result<(), Error>>()
        //     .try_invoke()
        //     .map_err(|e| format!("approve_nft failed: {:?}", e));

        gstd::debug!("approve_nft error:{:?}", approve_nft);

        // store nft tokenid and spender address
        if self.nfts.contains_key(&owner) {
            let mut nft_tokens = self
                .nfts
                .get_mut(&owner)
                .expect("Failed to get (spender, token_id)");
            nft_tokens.push((spender, token_id, false));
            self.nfts.insert(owner, nft_tokens.clone());
        } else {
            let mut spender_nftid_claim = Vec::new();
            spender_nftid_claim.push((spender, token_id, false));

            self.nfts.insert(owner, spender_nftid_claim);
        }
        gstd::debug!("mint_approve_nft--over");

        true
    }

    // spender claim balances to his account
    // claim vara
    // claim token, frontend should be transfer 0.000000000001 Unit represent 1 Token.
    pub fn transfer_balances_from(
        &mut self,
        owner: ActorId,
        vara_value: u128,
        token_value: U256,
        erc20_num: u32,
    ) -> bool {
        let spender = msg::source();

        // check nft authorization
        if self.nfts.contains_key(&owner) {
            let spender_nftid_claim = self
                .nfts
                .get(&owner)
                .expect("failed to get nfts spender_value");
            gstd::debug!("transfer_balances_from: {:?}", spender_nftid_claim);
            let spender_value = spender_nftid_claim.iter().find(|&x| x.0 == spender);
            gstd::debug!("spender_value: {:?}", spender_value);
            if let Some(&(_, _, claimed)) = spender_value {
                if !claimed {
                    let _ = self.notify_on(Event::NoClaimedNFT);
                    panic!("NoClaimedNFT");
                }
            } else {
                let _ = self.notify_on(Event::NoExistNFTApprove);
                panic!("NoExistNFTApprove");
            }
        } else {
            gstd::debug!("nfts not contains owner");
            let _ = self.notify_on(Event::NoExistNFTApprove);
            panic!("NoExistNFTApprove");
        }
        gstd::debug!("check nft authorization--over");

        if !self.balances.contains_key(&spender) {
            // check vara authorization
            if vara_value > 0 {
                let _ = self.notify_on(Event::NoExistVaraApprove);
                panic!("NoExistVaraApprove");
            }
            // check vara authorization
            if token_value > U256::from(0) {
                let _ = self.notify_on(Event::NoExistTokenApprove);
                panic!("NoExistTokenApprove");
            }
        } else {
            let owner_vara_token = self
                .balances
                .get(&spender)
                .expect("failed to get balances owner_value");

            let owner_value = owner_vara_token.iter().find(|&x| x.0 == owner);

            if owner_value.is_none() {
                let _ = self.notify_on(Event::NoExistVaraApprove);
                panic!("NoExistVaraApprove");
            }
            gstd::debug!("owner_value: {:?}", owner_value);
            gstd::debug!(
                "owner_value 1: {:?}",
                owner_value.expect("failed to get owner value").1
            );
            gstd::debug!(
                "owner_value 2: {:?}",
                owner_value.expect("failed to get owner value").2
            );
            if vara_value > 0
                && owner_value.expect("failed to get owner value").1 < vara_value
            {
                let _ = self.notify_on(Event::InsufficientApproveVaras);
                panic!("InsufficientApproveVaras");
                // transfer vara to spender account, gas fee will be deducted from spender account.
            } else if gstd::msg::send_with_gas(spender, Event::TransactionAlreadySend, 1, vara_value).is_err() {
                let _ = self.notify_on(Event::TransactionFailed);
                panic!("TransactionFailed");
            }

            let mut current_erc20 = self.erc20_address[0];

            if (self.erc20_address.len() as u32)
                > erc20_num.checked_add(1).expect("Failed to add erc20_num")
            {
                current_erc20 = self.erc20_address[erc20_num as usize];
            }
            gstd::debug!("vara_value:{}---token_value:{}", vara_value, token_value);
            if token_value > U256::from(0)
                && owner_value.expect("failed to get value").2 < token_value
            {
                // if InsufficientApproveTokens is true, frontend should not call erc20 transferFrom function.
                let _ = self.notify_on(Event::InsufficientApproveTokens);
                panic!("InsufficientApproveTokens");
            }

            // // spender claim straight through ERC20 claim token
            // let _ = build_call::<DefaultEnvironment>()
            //     .call(current_erc20)
            //     .call_v1()
            //     .gas_limit(0)
            //     .transferred_value(0)
            //     .exec_input(
            //         ExecutionInput::new(Selector::new(ink::selector_bytes!("transferFrom")))
            //             .push_arg(owner)
            //             .push_arg(spender)
            //             .push_arg(token),
            //     )
            //     .returns::<Vec<u8>>()
            //     .try_invoke()
            //     .map_err(|_| Error::TransactionTokenCallFailed)?;
        }

        gstd::debug!("check vara authorization--over");

        let owner_vara_token = self
            .balances
            .get_mut(&spender)
            .expect("failed to take owner value");
        // subtract approve value
        for (k, vara_v, token_v) in owner_vara_token.iter_mut() {
            if k == &owner {
                *vara_v = (*vara_v)
                    .checked_sub(vara_value)
                    .expect("subtract transfer vara failed");

                *token_v = (*token_v)
                    .checked_sub(token_value)
                    .expect("subtract transfer token failed");
                break;
            }
        }
        // self.balances.insert(spender, owner_vara_token.clone());
        gstd::debug!(
            "transfer_balances_from over owner_vara_token::{:?}",
            owner_vara_token
        );

        true
    }

    // spender claim nft to his account
    pub fn transfer_nft_from(&mut self, owner: ActorId, erc721_num: u32) -> bool {
        let spender = msg::source();

        if !self.nfts.contains_key(&owner) {
            let _ = self.notify_on(Event::NoExistNFTApprove);
            panic!("NoExistNFTApprove");
        }

        let spender_nftid_claim = self.nfts.get(&owner).expect("failed to get owner_value");

        let spender_value = spender_nftid_claim.iter().find(|&x| x.0 == spender);

        if spender_value.is_none() {
            let _ = self.notify_on(Event::NoExistNFTApprove);
            panic!("NoExistNFTApprove");
        }

        let token_id = spender_value.expect("failed to get spender value").1;
        gstd::debug!(
            "transfer_nft_from spender_token_id_vec::{:?}",
            spender_nftid_claim
        );
        if token_id == U256::from(0) {
            let _ = self.notify_on(Event::InsufficientApproveTokens);
            panic!("InsufficientApproveTokens");
        }
        gstd::debug!("token_id: {:?}", token_id);
        let mut current_erc721 = self.erc721_address[0];
        if (self.erc721_address.len() as u32)
            > erc721_num.checked_add(1).expect("Failed to add erc721_num")
        {
            current_erc721 = self.erc721_address[erc721_num as usize];
        }

        // // spender claim straight through ERC721 claim nft
        // let transfer_nft = build_call::<DefaultEnvironment>()
        //     .call(current_erc721)
        //     .call_v1()
        //     .gas_limit(0)
        //     .transferred_value(0)
        //     .exec_input(
        //         ExecutionInput::new(Selector::new(ink::selector_bytes!("transferFrom")))
        //             .push_arg(self.admin_address[0])
        //             .push_arg(spender)
        //             .push_arg(token_id),
        //     )
        //     .returns::<()>()
        //     .try_invoke()
        //     // .map_err(|_| Error::TransactionNFTCallFailed)?;
        //     .map_err(|e| format!("transfer_nft failed: {:?}", e));

        // ink::env::debug_println!("transfer_nft error3:{:?}", transfer_nft);

        let spender_nftid_claim = self
            .nfts
            .get_mut(&owner)
            .expect("failed to take owner value");
        // Set already claim nft to true
        if let Some(index) = spender_nftid_claim.iter().position(|&x| x.0 == spender) {
            spender_nftid_claim[index].2 = true;
            // self.nfts.insert(owner, spender_nftid_claim.clone());
            gstd::debug!("Element found and modified: {:?}", spender_nftid_claim);
        } else {
            gstd::debug!("Element not found");
        }
        // spender_nftid_claim.retain(|&x| x.0 == spender && x.1 == token_id);
        gstd::debug!(
            "transfer_nft_from spender_nftid_claim::{:?}",
            spender_nftid_claim
        );
        gstd::debug!("transfer_nft_from self.nfts:{:?}", self.nfts);
        if self.nfts.contains_key(&owner) {
            let spender_nftid_claim = self
                .nfts
                .get(&owner)
                .expect("Failed to try get_approved_balances_for_owner");
            gstd::debug!("spender_nftid_claim: {:?}", spender_nftid_claim);
        }
        gstd::debug!("transfer_nft_from over");

        true
    }

    pub fn add_contract_address(
        &mut self,
        erc20_address: ActorId,
        erc721_address: ActorId,
    ) -> bool {
        let current_caller = msg::source();
        if !self.admin_address.contains(&current_caller) {
            let _ = self.notify_on(Event::NoAuthorityAddContractAddress);
            panic!("NoAuthorityAddContractAddress");
        }

        // add erc20 contract address
        let erc20_address_vec = &mut self.erc20_address;
        if erc20_address_vec.contains(&erc20_address) {
            let _ = self.notify_on(Event::AlreadyExistTokenAddress);
            panic!("AlreadyExistTokenAddress");
        }
        erc20_address_vec.push(erc20_address);

        // add erc721 contract address
        let erc721_address_vec = &mut self.erc721_address;
        if erc721_address_vec.contains(&erc721_address) {
            let _ = self.notify_on(Event::AlreadyExistNFTAddress);
            panic!("AlreadyExistNFTAddress");
        }
        erc721_address_vec.push(erc721_address);

        true
    }

    pub fn add_auth_token_owner(&mut self, owner_address: ActorId) -> bool {
        let current_caller = msg::source();
        if !self.auth_token_owner.contains(&current_caller) {
            let _ = self.notify_on(Event::NoAuthorityAddAuthTokenOwner);
            panic!("NoAuthorityAddAuthTokenOwner");
        }

        // add auth_token_owner address
        let auth_owner_address_vec = &mut self.auth_token_owner;
        if auth_owner_address_vec.contains(&owner_address) {
            let _ = self.notify_on(Event::AlreadyExistAuthAddress);
            panic!("AlreadyExistAuthAddress");
        }
        auth_owner_address_vec.push(owner_address);

        true
    }
}

#[derive(Default)]
pub struct L2eProgram;

#[program]
impl L2eProgram {
    // Program's constructor
    pub fn new(erc20: ActorId, erc721: ActorId) -> Self {
        L2eTop::new(erc20, erc721);
        Self
    }

    // Exposed service
    pub fn l2e(&self) -> L2eTop {
        L2eTop::default()
    }
}
