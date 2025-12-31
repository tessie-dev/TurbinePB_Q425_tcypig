import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { CapstoneSafeswap } from "../target/types/capstone_safeswap";
import { PublicKey, SystemProgram, LAMPORTS_PER_SOL, Keypair } from "@solana/web3.js";
import { assert } from "chai";
import { u64 } from "@metaplex-foundation/umi/serializers";


function statusKey(s: any): string {
  return Object.keys(s ?? {})[0];
}

function u64ToLeBuffer(n: number): Buffer {
  const buf = Buffer.alloc(8);
  buf.writeBigUInt64LE(BigInt(n));
  return buf;
}

describe("capstone_safeswap", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.capstoneSafeswap as Program<CapstoneSafeswap>;

  const seller = Keypair.generate();
  const buyer = Keypair.generate();
  const stranger = Keypair.generate();

  let escrowPda: PublicKey;
  let vaultPda: PublicKey;
  let bump: number;

  const BN = anchor.BN;
  const amount = new BN(0.2 * LAMPORTS_PER_SOL); // 0.2 SOL
  const expireAt = new BN(Math.floor(Date.now() / 1000) + 3600); // now + 1h
  const listingId = new BN(1);

  async function airdrop(pubkey: PublicKey, sol = 2) {
    const sig = await provider.connection.requestAirdrop(
      pubkey,
      sol * LAMPORTS_PER_SOL
    );

    const latest = await provider.connection.getLatestBlockhash();
    await provider.connection.confirmTransaction(
      { signature: sig, ...latest },
      "confirmed"
    );
  }

  before(async () => {
    // airdrop SOL to accounts
    await airdrop(seller.publicKey, 2);
    await airdrop(buyer.publicKey, 2);
    await airdrop(stranger.publicKey, 2);

    // derive PDA: seeds = [b"escrow", seller.key().as_ref()]
    [escrowPda, bump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("escrow"), 
        seller.publicKey.toBuffer(),
        // u64ToLeBuffer(listingId)
        u64ToLeBuffer(Number(listingId))
      ],
      program.programId
    );

    // vault PDA: seeds = [b"vault", escrow.key().as_ref()]
    [vaultPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), escrowPda.toBuffer()],
      program.programId
    );
  });

  // create escrow
  it("Is initialized!", async () => {
    await program.methods
      .createEscrow(listingId, amount, expireAt)
      .accountsPartial({
        seller: seller.publicKey,
        escrow: escrowPda,
        vault: vaultPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([seller])
      .rpc({ commitment: "confirmed" });
    
    const escrow = await program.account.escrowAccount.fetch(escrowPda);

    assert.equal(
      escrow.seller.toBase58(),
      seller.publicKey.toBase58()
    );
    assert.equal(
      escrow.buyer.toBase58(),
      PublicKey.default.toBase58()
    );
    assert.equal(statusKey(escrow.status), "created");
    assert.equal(escrow.amount.toString(), amount.toString());
  });


  // fund escrow
  // success case
  it("fund_escrow transfers SOL buyer → PDA and sets Funded", async () => {
    const buyerBefore = await provider.connection.getBalance(
      buyer.publicKey
    );
    const vaultBefore = await provider.connection.getBalance(
      vaultPda
    );

    await program.methods
      .fundEscrow(listingId)
      .accountsPartial({
        buyer: buyer.publicKey,
        seller: seller.publicKey,
        escrow: escrowPda,
        vault: vaultPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([buyer])
      .rpc({ commitment: "confirmed" });

    const buyerAfter = await provider.connection.getBalance(
      buyer.publicKey
    );
    const vaultAfter = await provider.connection.getBalance(
      vaultPda
    );

    assert.equal(
      vaultAfter - vaultBefore,
      amount.toNumber()
    );
    assert.isAtMost(
      buyerAfter,
      buyerBefore - amount.toNumber()
    );

    const escrow = await program.account.escrowAccount.fetch(escrowPda);
    assert.equal(
      escrow.buyer.toBase58(),
      buyer.publicKey.toBase58()
    );
    assert.equal(statusKey(escrow.status), "funded");
  });

  // fund escrow
  // failure case: escrow not in Created
  it("fund_escrow fails if escrow is not in Created", async () => {
    try {
      await program.methods
        .fundEscrow(listingId)
        .accountsPartial({
          buyer: buyer.publicKey,
          seller: seller.publicKey,
          escrow: escrowPda,
          vault: vaultPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([buyer])
        .rpc();

      assert.fail("Expected InvalidStatus error");
    } catch (e: any) {
      const ae = anchor.AnchorError.parse(e.logs);
      assert.equal(ae.error.errorCode.code, "InvalidStatus");
    }
  });


  // complete escrow
  it("complete_escrow releases SOL PDA → seller", async () => {
    const sellerBefore = await provider.connection.getBalance(
      seller.publicKey
    );
    const vaultBefore = await provider.connection.getBalance(
      vaultPda
    );

    await program.methods
      .completeEscrow(listingId)
      .accountsPartial({
        buyer: buyer.publicKey,
        seller: seller.publicKey,
        escrow: escrowPda,
        vault: vaultPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([buyer])
      .rpc({ commitment: "confirmed" });

    const sellerAfter = await provider.connection.getBalance(
      seller.publicKey
    );
    const vaultAfter = await provider.connection.getBalance(
      vaultPda
    );

    assert.equal(
      vaultBefore - vaultAfter,
      amount.toNumber()
    );
    assert.isAtLeast(
      sellerAfter,
      sellerBefore + amount.toNumber()
    );

    const escrow = await program.account.escrowAccount.fetch(escrowPda);
    assert.equal(statusKey(escrow.status), "completed");
  });


  // refund escrow
  it("refund_escrow returns SOL vault → buyer and sets Cancelled", async () => {
    const seller2 = Keypair.generate();
    const buyer2 = Keypair.generate();
    const listingId2 = new anchor.BN(2);

    await airdrop(seller2.publicKey, 2);
    await airdrop(buyer2.publicKey, 2);

    const [escrow2] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("escrow"), 
        seller2.publicKey.toBuffer()
        ,listingId2.toArrayLike(Buffer, "le", 8)
      ],
      program.programId
    );

    const [vault2] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), escrow2.toBuffer()],
      program.programId
    );

    // create (init escrow + vault)
    await program.methods
      .createEscrow(listingId2, amount, expireAt)
      .accountsPartial({
        seller: seller2.publicKey,
        escrow: escrow2,
        vault: vault2,
        systemProgram: SystemProgram.programId,
      })
      .signers([seller2])
      .rpc({ commitment: "confirmed" });

    // fund (buyer -> vault)
    await program.methods
      .fundEscrow(listingId2)
      .accountsPartial({
        buyer: buyer2.publicKey,
        seller: seller2.publicKey,
        escrow: escrow2,
        vault: vault2,
        systemProgram: SystemProgram.programId,
      })
      .signers([buyer2])
      .rpc({ commitment: "confirmed" });

    const buyerBefore = await provider.connection.getBalance(buyer2.publicKey, "confirmed");
    const vaultBefore = await provider.connection.getBalance(vault2, "confirmed");

    // refund (vault -> buyer)
    await program.methods
      .refundEscrow(listingId2)
      .accountsPartial({
        buyer: buyer2.publicKey,
        seller: seller2.publicKey,
        escrow: escrow2,
        vault: vault2,
        systemProgram: SystemProgram.programId,
      })
      .signers([buyer2])
      .rpc({ commitment: "confirmed" });

    const buyerAfter = await provider.connection.getBalance(buyer2.publicKey, "confirmed");
    const vaultAfter = await provider.connection.getBalance(vault2, "confirmed");

    // vault should decrease by amount (buyer should increase by ~amount, minus tx fees)
    assert.equal(vaultBefore - vaultAfter, amount.toNumber());
    assert.isAtLeast(buyerAfter, buyerBefore + amount.toNumber() - 50_000);

    const escrow = await program.account.escrowAccount.fetch(escrow2);
    assert.equal(statusKey(escrow.status), "cancelled");
  });


  // cancel escrow
  it("cancel_escrow cancels Created escrow (no SOL movement)", async () => {
    const seller3 = Keypair.generate();
    await airdrop(seller3.publicKey, 2);
    const listingId3 = new anchor.BN(3);

    const [escrow3] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("escrow"), 
        seller3.publicKey.toBuffer(),
        listingId3.toArrayLike(Buffer, "le", 8)
      ],
      program.programId
    );

    const [vault3] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), escrow3.toBuffer()],
      program.programId
    );

    // create
    await program.methods
      .createEscrow(listingId3, amount, expireAt)
      .accountsPartial({
        seller: seller3.publicKey,
        escrow: escrow3,
        vault: vault3,
        systemProgram: SystemProgram.programId,
      })
      .signers([seller3])
      .rpc({ commitment: "confirmed" });

    const vaultBefore = await provider.connection.getBalance(vault3, "confirmed");

    // cancel (seller cancels)
    await program.methods
      .cancelEscrow(listingId3)
      .accountsPartial({
        seller: seller3.publicKey,
        escrow: escrow3,
      })
      .signers([seller3])
      .rpc({ commitment: "confirmed" });

    const vaultAfter = await provider.connection.getBalance(vault3, "confirmed");
    assert.equal(vaultAfter, vaultBefore);

    const escrow = await program.account.escrowAccount.fetch(escrow3);
    assert.equal(statusKey(escrow.status), "cancelled");
  });


});
