import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Purplepay } from "../target/types/purplepay";
import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  Signer,
  SystemProgram,
  SYSVAR_CLOCK_PUBKEY,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import {ethers} from "ethers";

import { airdropSol } from "./utils";
describe("purplepay", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Purplepay as Program<Purplepay>;
  it("Is initialized!", async () => {
    const user = Keypair.generate();
    await airdropSol(user.publicKey, 10);
    let name = "abhishek";
    let parentChain = "solana";
    let data = JSON.stringify({ email: "email.test@gmail.com" });

    let crosschainAccount = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("crosschain_id"), user.publicKey.toBuffer()],
      program.programId
    )[0];

    const tx = await program.methods
      .initialize(name, parentChain, data)
      .accounts({
        crosschainAccount,
        signer: user.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc({
        skipPreflight: true
      });
    console.log("Your transaction signature", tx);
    const crosschain_account_data = await program.account.crosschainId.fetch(crosschainAccount);
    console.log("crosschain_account: ",crosschain_account_data);
    let decoded = ethers.utils.defaultAbiCoder.decode(["string"],crosschain_account_data.nameHash);
    console.log("name_decoded: ",decoded);
  });
});
