import assert from "assert";
import * as anchor from "@coral-xyz/anchor";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";
import { Metaplex, keypairIdentity } from "@metaplex-foundation/js";
import type { SagaDrop } from "../target/types/saga_drop";
import { Keypair, LAMPORTS_PER_SOL, PublicKey, Transaction, sendAndConfirmTransaction } from "@solana/web3.js";
import { airdropToMultiple, createNewMintTransaction } from "./utils/utils";

describe("Test", () => {
  // Configure the client to use the local cluster
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.SagaDrop as anchor.Program<SagaDrop>;
  
  // Initialize keypairs
  let payer = Keypair.generate();
  const mintKeypair = Keypair.generate();
  const mint = mintKeypair.publicKey;

  // deprecated metaplex setup b/c Solpg doesn't have umi yet.
  const metaplex = Metaplex.make(program.provider.connection).use(
    keypairIdentity(payer)
  );

  // Saga5xJVLEAvm23n5NB3bCTsyFvEWWc3Rdgjz3zHUXt DO NOT USE IN PRODUCTION
  const nftCollection = Keypair.fromSecretKey(
    new Uint8Array([
      191, 170, 71, 55, 85, 150, 40, 39, 218, 201, 198, 76, 152, 145, 86, 140,
      141, 238, 180, 96, 107, 23, 239, 234, 22, 53, 40, 158, 2, 142, 203, 0, 6,
      141, 154, 216, 146, 247, 205, 152, 135, 246, 228, 182, 98, 107, 150, 250,
      250, 232, 105, 106, 175, 51, 158, 229, 31, 61, 226, 70, 57, 142, 236, 91,
    ])
  );
  const nftMint = Keypair.generate();
  const nftTokenAccount = getAssociatedTokenAddressSync(
    nftMint.publicKey,
    payer.publicKey
  );
  let nftMetadata;
  const ineligibleWallet = Keypair.generate();
  const ineligibleTokenAccount = getAssociatedTokenAddressSync(
    mint,
    ineligibleWallet.publicKey
  );
  const sourceTokenAccount = getAssociatedTokenAddressSync(
    mint,
    payer.publicKey
  );
  const SAGA_COLLECTION = nftCollection.publicKey;
  const [dropPda] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("drop"),
      SAGA_COLLECTION.toBuffer(),
      mint.toBuffer(),
      payer.publicKey.toBuffer(),
    ],
    program.programId
  );
  const dropEscrowAccount = getAssociatedTokenAddressSync(mint, dropPda, true);

  const numDecimals = 9;
  const numTokens = 40_000_000;
  // Define Player PDAs and ATAs
  const [programState] = PublicKey.findProgramAddressSync(
    [Buffer.from("program_state")],
    program.programId
  );

  const [feeVault] = PublicKey.findProgramAddressSync(
    [Buffer.from("fee_vault")],
    program.programId
  );
  const [claimReceipt] = PublicKey.findProgramAddressSync(
    [Buffer.from("claim"), dropPda.toBuffer(), nftMint.publicKey.toBuffer()],
    program.programId
  );

  before(async () => {
    // Request and confirm airdrops
    await airdropToMultiple([ineligibleWallet.publicKey, payer.publicKey], program.provider.connection, 100*LAMPORTS_PER_SOL);

    try {
      const { nft: collectionNft } = await metaplex.nfts().create(
        {
          useNewMint: nftCollection,
          uri: "https://api.underdog-data.com/jsondata/46pcSL5gmjBrPqGKFaLbbCmR6iVuLJbnQy13hAe7s6CC",
          name: "Saga Collection",
          symbol: "SAGAGEN",
          sellerFeeBasisPoints: 0,
          isCollection: true,
          collectionAuthority: payer,
        },
        { commitment: "processed", confirmOptions: { skipPreflight: true } }
      );
      console.log("Collection: ", collectionNft.address.toBase58());
    } catch {
      console.log("Collection NFT Already minted");
    }

    const { nft: sagaNft } = await metaplex.nfts().create(
      {
        useNewMint: nftMint,
        collection: nftCollection.publicKey,
        uri: "https://updg8.com/jsondata/Fy79Aii43rLGS4k5U2oW7fn6CgN9tsjBr2FnssY5ZqRR",
        name: "Saga genesis token",
        symbol: "SAGA GE",
        sellerFeeBasisPoints: 0,
      },
      { commitment: "processed", confirmOptions: { skipPreflight: true } }
    );
    console.log("NFT: ", sagaNft.address.toBase58());
    nftMetadata = sagaNft.metadataAddress;
    await metaplex.nfts().verifyCollection(
      {
        mintAddress: sagaNft.address,
        collectionMintAddress: nftCollection.publicKey,
      },
      { commitment: "processed", confirmOptions: { skipPreflight: true } }
    );

    const newMintTransaction: Transaction = await createNewMintTransaction(
      program.provider.connection,
      payer,
      mintKeypair,
      payer.publicKey,
      payer.publicKey,
      payer.publicKey,
      numDecimals,
      numTokens
    );

    let { lastValidBlockHeight, blockhash } =
      await program.provider.connection.getLatestBlockhash("finalized");
    newMintTransaction.recentBlockhash = blockhash;
    newMintTransaction.lastValidBlockHeight = lastValidBlockHeight;
    newMintTransaction.feePayer = payer.publicKey;
    const transactionId = await sendAndConfirmTransaction(
      program.provider.connection,
      newMintTransaction,
      [payer, mintKeypair]
    );
    console.log("New Mint Success! ", transactionId);
  });

  it("Init Program", async () => {
    let currentProgramState;
    try {
      currentProgramState = await program.account.programState.fetch(
        programState
      );
    } catch (error) {
      currentProgramState = null;
    }

    if (currentProgramState) {
      // Program state exists, skip initialization
      return;
    }
    try {
      // Send transaction
      const txHash = await program.methods
        .initialize()
        .accounts({
          programState,
          signer: payer.publicKey,
        })
        .signers([payer])
        .rpc({ skipPreflight: true });
      // Confirm transaction
      await program.provider.connection.confirmTransaction(txHash);
      const currentProgramState = await program.account.programState.fetch(
        programState
      );

      assert(currentProgramState.active, "Program should be Active.");
    } catch (error) {
      assert.fail(`Error in transaction: ${error}`);
    }
  });
  it("Create Drop", async () => {
    try {
      const args = {
        dropSize: new anchor.BN(2000),
        startEpoch: new anchor.BN(5),
        endEpoch: new anchor.BN(8),
      };
      // Send transaction
      const txHash = await program.methods
        .createDrop(args)
        .accounts({
          authority: payer.publicKey,
          programState,
          dropPda,
          mint,
          dropEscrowAccount,
          feeVault,
          sourceTokenAccount,
        })
        .signers([payer])
        .rpc({ skipPreflight: true });
      // Confirm transaction
      await program.provider.connection.confirmTransaction(txHash);
    } catch (error) {
      assert.fail(`Error in transaction: ${error}`);
    }
  });
  it("Cannot Create Same Drop Again", async () => {
    let didThrow = false;
    try {
      const args = {
        dropSize: new anchor.BN(2000),
        startEpoch: new anchor.BN(5),
        endEpoch: new anchor.BN(8),
      };
      // Send transaction
      const txHash = await program.methods
        .createDrop(args)
        .accounts({
          authority: payer.publicKey,
          programState,
          dropPda,
          mint,
          dropEscrowAccount,
          feeVault,
          sourceTokenAccount,
        })
        .signers([payer])
        .rpc({ skipPreflight: true });

      // Confirm transaction
      await program.provider.connection.confirmTransaction(txHash);
    } catch (error) {
      didThrow = true;
    } finally {
      assert(didThrow, "Transaction should have thrown an error but didn't.");
    }
  });
  it("Wallet without Saga Cannot Claim Drop", async () => {
    let didThrow = false;

    try {
      const txHash = await program.methods
        .claim()
        .accounts({
          claimer: ineligibleWallet.publicKey,
          programState,
          dropPda,
          tokenMint: mint,
          escrowTokenAccount: dropEscrowAccount,
          destination: ineligibleTokenAccount,
          claimReceipt,
          nftMint: nftMint.publicKey,
          nftTokenAccount,
          nftMetadata,
        })
        .signers([ineligibleWallet])
        .rpc({ skipPreflight: true });

      // Confirm transaction
      await program.provider.connection.confirmTransaction(txHash);
    } catch (error) {
      didThrow = true;
    } finally {
      assert(didThrow, "Transaction should have thrown an error but didn't.");
    }
  });
  it("Claim Drop", async () => {
    try {
      const txHash = await program.methods
        .claim()
        .accounts({
          claimer: payer.publicKey,
          programState,
          dropPda,
          tokenMint: mint,
          escrowTokenAccount: dropEscrowAccount,
          destination: sourceTokenAccount,
          claimReceipt,
          nftMint: nftMint.publicKey,
          nftTokenAccount,
          nftMetadata,
        })
        .signers([payer])
        .rpc({ skipPreflight: true });

      // Confirm transaction
      await program.provider.connection.confirmTransaction(txHash);
    } catch (error) {
      assert.fail(`Error in transaction: ${error}`);
    }
  });
  it("Cannot Claim Drop Again", async () => {
    let didThrow = false;
    try {
      const txHash = await program.methods
        .claim()
        .accounts({
          claimer: payer.publicKey,
          programState,
          dropPda,
          tokenMint: mint,
          escrowTokenAccount: dropEscrowAccount,
          destination: sourceTokenAccount,
          claimReceipt,
          nftMint: nftMint.publicKey,
          nftTokenAccount,
          nftMetadata,
        })
        .signers([payer])
        .rpc({ skipPreflight: true });

      // Confirm transaction
      await program.provider.connection.confirmTransaction(txHash);
    } catch (error) {
      didThrow = true;
    } finally {
      assert(didThrow, "Transaction should have thrown an error but didn't.");
    }
  });
});


/*

*/

/*
  
ub struct ClaimDrop<'info> {


    // NFT's mint address--validated it qualifies through metadata constraints
    /// Unchecked - we verify mint in subsequent accounts
    pub nft_mint: Account<'info, Mint>,

    //
    #[account(
        constraint = nft_token_account.owner == claimer.key() @ DropError::DoesNotOwnGenesisToken,
        constraint = nft_token_account.amount == 1 @ DropError::DoesNotOwnGenesisToken,
        token::mint = nft_mint,
        // associated_token::authority = claimer @ DropError::DoesNotOwnGenesisToken,
    )]
    pub nft_token_account: Account<'info, TokenAccount>,

    #[account(
        address = mpl_token_metadata::accounts::Metadata::find_pda(&nft_mint.key()).0,
        //https://docs.rs/mpl-token-metadata/3.2.3/mpl_token_metadata/accounts/struct.Metadata.html#method.find_pda
        constraint = nft_metadata.collection.as_ref().unwrap().verified == true @ DropError::NotValidGenesisToken,
        constraint = nft_metadata.collection.as_ref().unwrap().key == drop_pda.wl_collection @ DropError::NotValidGenesisToken    
    )]
    pub nft_metadata: Account<'info, MetadataAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}









  TESTS
1. need to test...somebody else's NFT
2. different collection

   */
