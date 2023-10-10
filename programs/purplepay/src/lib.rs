use anchor_lang::prelude::*;

declare_id!("JEBJJZHevjah4H2Xczry1Pztzgy2gvKG1BRchpg5Vds1");

pub use context::*;
pub use error::*;
pub use message::*;
pub use state::*;

pub mod context;
pub mod error;

pub mod state;


#[program]
pub mod purplepay {
   

    use anchor_lang::solana_program;
    use wormhole_anchor_sdk::wormhole;

    use super::*;

    //TODO : Add read message

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let config = &mut ctx.accounts.config;
        // Set the owner of the config (effectively the owner of the program).
        config.owner = ctx.accounts.owner.key();

        // Set Wormhole related addresses.
        {
            let wormhole = &mut config.wormhole;

            // wormhole::BridgeData (Wormhole's program data).
            wormhole.bridge = ctx.accounts.wormhole_bridge.key();

            // wormhole::FeeCollector (lamports collector for posting
            // messages).
            wormhole.fee_collector = ctx.accounts.wormhole_fee_collector.key();

            // wormhole::SequenceTracker (tracks # of messages posted by this
            // program).
            wormhole.sequence = ctx.accounts.wormhole_sequence.key();
        }

        // Set default values for posting Wormhole messages.
        //
        // Zero means no batching.
        config.batch_id = 0;
        config.finality = wormhole::Finality::Confirmed as u8;

        ctx.accounts.wormhole_emitter.bump = *ctx.bumps
            .get("wormhole_emitter")
            .ok_or(HelloWorldError::BumpNotFound)?;

        {
            let fee = ctx.accounts.wormhole_bridge.fee();
            if fee > 0 {
                solana_program::program::invoke(
                    &solana_program::system_instruction::transfer(
                        &ctx.accounts.owner.key(),
                        &ctx.accounts.wormhole_fee_collector.key(),
                        fee
                    ),
                    &ctx.accounts.to_account_infos()
                )?;
            }
            let wormhole_emitter = &ctx.accounts.wormhole_emitter;
            let config = &ctx.accounts.config;

            let mut payload: Vec<u8> = Vec::new();
            HelloWorldMessage::serialize(
                &(HelloWorldMessage::Alive {
                    program_id: *ctx.program_id,
                }),
                &mut payload
            )?;

            wormhole::post_message(
                CpiContext::new_with_signer(
                    ctx.accounts.wormhole_program.to_account_info(),
                    wormhole::PostMessage {
                        config: ctx.accounts.wormhole_bridge.to_account_info(),
                        message: ctx.accounts.wormhole_message.to_account_info(),
                        emitter: wormhole_emitter.to_account_info(),
                        sequence: ctx.accounts.wormhole_sequence.to_account_info(),
                        payer: ctx.accounts.owner.to_account_info(),
                        fee_collector: ctx.accounts.wormhole_fee_collector.to_account_info(),
                        clock: ctx.accounts.clock.to_account_info(),
                        rent: ctx.accounts.rent.to_account_info(),
                        system_program: ctx.accounts.system_program.to_account_info(),
                    },
                    &[
                        &[
                            SEED_PREFIX_SENT,
                            &wormhole::INITIAL_SEQUENCE.to_le_bytes()[..],
                            &[
                                *ctx.bumps
                                    .get("wormhole_message")
                                    .ok_or(HelloWorldError::BumpNotFound)?,
                            ],
                        ],
                        &[wormhole::SEED_PREFIX_EMITTER, &[wormhole_emitter.bump]],
                    ]
                ),
                config.batch_id,
                payload,
                config.finality.try_into().unwrap()
            )?;
        }

        // Done.
        Ok(())
    }

    pub fn register_emitter(
        ctx: Context<RegisterEmitter>,
        chain: u16,
        address: [u8; 32]
    ) -> Result<()> {
        // Foreign emitter cannot share the same Wormhole Chain ID as the
        // Solana Wormhole program's. And cannot register a zero address.
        require!(
            chain > 0 && chain != wormhole::CHAIN_ID_SOLANA && !address.iter().all(|&x| x == 0),
            HelloWorldError::InvalidForeignEmitter
        );

        // Save the emitter info into the ForeignEmitter account.
        let emitter = &mut ctx.accounts.foreign_emitter;
        emitter.chain = chain;
        emitter.address = address;

        // Done.
        Ok(())
    }
    //TODO: edit so that we dont have to take message directly
    pub fn send_message(ctx: Context<SendMessage>) -> Result<()> {
        // If Wormhole requires a fee before posting a message, we need to
        // transfer lamports to the fee collector. Otherwise
        // `wormhole::post_message` will fail.
        let fee = ctx.accounts.wormhole_bridge.fee();
        if fee > 0 {
            solana_program::program::invoke(
                &solana_program::system_instruction::transfer(
                    &ctx.accounts.payer.key(),
                    &ctx.accounts.wormhole_fee_collector.key(),
                    fee
                ),
                &ctx.accounts.to_account_infos()
            )?;
        }

        let wormhole_emitter = &ctx.accounts.wormhole_emitter;
        let config = &ctx.accounts.config;

        // There is only one type of message that this example uses to
        // communicate with its foreign counterparts (payload ID == 1).
        
        let message: CrosschainID = &ctx.accounts.crosschain_account;
        //TODO : Change accordingly (our message updated to struct)
        let payload: Vec<u8> = (HelloWorldMessage::Hello { message }).try_to_vec()?;
        //can we write like this
        // let payload: Vec<u8> = solabi::encode(message);
        //TODO: Do the equivalent of abi encode

        wormhole::post_message(
            CpiContext::new_with_signer(
                ctx.accounts.wormhole_program.to_account_info(),
                wormhole::PostMessage {
                    config: ctx.accounts.wormhole_bridge.to_account_info(),
                    message: ctx.accounts.wormhole_message.to_account_info(),
                    emitter: wormhole_emitter.to_account_info(),
                    sequence: ctx.accounts.wormhole_sequence.to_account_info(),
                    payer: ctx.accounts.payer.to_account_info(),
                    fee_collector: ctx.accounts.wormhole_fee_collector.to_account_info(),
                    clock: ctx.accounts.clock.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                },
                &[
                    &[
                        SEED_PREFIX_SENT,
                        &ctx.accounts.wormhole_sequence.next_value().to_le_bytes()[..],
                        &[*ctx.bumps.get("wormhole_message").ok_or(HelloWorldError::BumpNotFound)?],
                    ],
                    &[wormhole::SEED_PREFIX_EMITTER, &[wormhole_emitter.bump]],
                ]
            ),
            config.batch_id,
            payload,
            config.finality.try_into().unwrap()
        )?;

        // Done.
        Ok(())
    }
    
    pub fn receive_message(ctx: Context<ReceiveMessage>, vaa_hash: [u8; 32]) -> Result<()> {
        let posted_message = &ctx.accounts.posted;

        if let HelloWorldMessage::Hello { message } = posted_message.data() {
            // HelloWorldMessage cannot be larger than the maximum size of the account.
            let len = CrosschainID::LEN;
            require!(
                len <= MESSAGE_MAX_LENGTH,
                HelloWorldError::InvalidMessage,
            );

            // Save batch ID, keccak256 hash and message payload.
            let received = &mut ctx.accounts.received;
            received.batch_id = posted_message.batch_id();
            received.wormhole_message_hash = vaa_hash;
            received.message = message.clone();

            //TODO: figure out how to decode and store (need to have common data store)

            // Done
            Ok(())
        } else {
            Err(HelloWorldError::InvalidMessage.into())
        }
    }

    pub fn read_received_message(ctx: Context<ReadReceivedMessage>) -> Result<Vec<u8>> {
        let received_account = &ctx.accounts.received;
        let received_message = received_account.message.clone();
        
        msg!("Received message: {:?}", received_message);

        Ok(received_message)
    }
    
    //TODO: remove solabi from here
    pub fn store_information(
        ctx: Context<SendMessage>,
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
