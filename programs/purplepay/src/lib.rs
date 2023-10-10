use anchor_lang::prelude::*;

declare_id!("JEBJJZHevjah4H2Xczry1Pztzgy2gvKG1BRchpg5Vds1");

#[program]
pub mod purplepay {
    use super::*;
    
    //TODO: Add init method from wormhole
    //TODO : Add wormhole post and receive message
    //TODO : Add read message


    pub fn store_information(
        ctx: Context<StoreInformation>,
        name: String,
        parent_chain: String,
        data: String
    ) -> Result<()> {
        let crosschain_account = &mut ctx.accounts.crosschain_account;
        crosschain_account.bump = *ctx.bumps.get("crosschain_account").unwrap();
        let name_hash = solabi::encode(&(name, parent_chain.clone()));
        let serialized_data = solabi::encode(&data);
        let hashed_chain = solabi::encode(
            &(parent_chain.clone(), ctx.accounts.signer.key().to_string())
        );
        msg!("1");
        crosschain_account.name_hash = name_hash;
        msg!("2");
        crosschain_account.serialized_data = serialized_data;
        msg!("3");
        crosschain_account.owner = ctx.accounts.signer.key();

        let mut multichain_addresses = vec![];
        multichain_addresses.push(hashed_chain);
        msg!("4");

        crosschain_account.multichain_addresses = multichain_addresses;
        Ok(())
    }
}

// fn convert_fixed<T, const N: usize>(v: Vec<T>) -> [T; N] {
//     v.try_into()
//         .unwrap_or_else(|v: Vec<T>| panic!("Expected a Vec of length {} but it was {}", N, v.len()))
// }
// #[derive(Accounts)]
// pub struct Initialize<'info> {
//     #[account(
//         init,
//         payer = signer,
//         seeds=[b"crosschain_id".as_ref(), signer.key().as_ref()],
//         bump,
//         space = 8 + CrosschainID::LEN
//         )]
//     pub crosschain_account: Account<'info, CrosschainID>,
//     pub system_program: Program<'info, System>,

//     #[account(mut)]
//     pub signer: Signer<'info>,
// }

// #[account]
// pub struct CrosschainID {
//     pub name_hash: Vec<u8>,                // 32 bytes
//     pub owner: Pubkey,                      // 32 bytes
//     pub serialized_data: Vec<u8>,          // 32 bytes
//     pub multichain_addresses: Vec<Vec<u8>>, // 32 * 10
//     pub bump: u8 // 1 byte
// }

// impl CrosschainID {
//     pub const LEN: usize = 5000;
// }
