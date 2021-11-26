const anchor = require("@project-serum/anchor");
const web3 = require("@solana/web3.js");
const assert = require("assert");

const REGISTRY_CONTEXT_SEED = "registry-context";
const ENTRY_SEED = "governance-program";

describe("Registry Tests", () => {
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.PermissionlessVerifiableRegistry;
  const programInstance = web3.Keypair.generate();
  const testData = "https://kforkofrk";

  it("Initializes the registry", async () => {
    const [registryContext, bump] = await web3.PublicKey.findProgramAddress(
      [anchor.utils.bytes.utf8.encode(REGISTRY_CONTEXT_SEED)],
      program.programId
    );

    const tx = await program.rpc.init(
      { bump, entrySeed: ENTRY_SEED, permissionless_add: true },
      {
        accounts: {
          registryContext,
          authority: provider.wallet.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        },
      }
    );
    console.log("Your transaction signature", tx);
    const data = await program.account.registryContext.fetch(registryContext);
    console.log("Found data: ", data);
    assert.equal(
      data.authority.toBase58(),
      provider.wallet.publicKey.toBase58()
    );
    assert.equal(data.entrySeed, ENTRY_SEED);
  });

  it("Add entry", async () => {
    const [registryContext] = await web3.PublicKey.findProgramAddress(
      [anchor.utils.bytes.utf8.encode(REGISTRY_CONTEXT_SEED)],
      program.programId
    );

    const [seededPubkey, bump] = await web3.PublicKey.findProgramAddress(
      [
        anchor.utils.bytes.utf8.encode(ENTRY_SEED),
        programInstance.publicKey.toBuffer(),
      ],
      program.programId
    );

    const tx = await program.rpc.addEntry(
      {
        bump,
        data: testData,
        address: programInstance.publicKey,
      },
      {
        accounts: {
          registryContext,
          entry: seededPubkey,
          creator: provider.wallet.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        },
      }
    );
    console.log("Your transaction signature", tx);
    const entry = await program.account.entryData.fetch(seededPubkey);
    console.log("Found data: ", entry);
    assert.equal(entry.data, testData);
    assert.equal(
      entry.address.toBase58(),
      programInstance.publicKey.toBase58()
    );
    assert.equal(entry.isVerified, false);
  });

  it("Verify an entry", async () => {
    const [registryContext] = await web3.PublicKey.findProgramAddress(
      [anchor.utils.bytes.utf8.encode(REGISTRY_CONTEXT_SEED)],
      program.programId
    );

    const [seededPubkey] = await web3.PublicKey.findProgramAddress(
      [
        anchor.utils.bytes.utf8.encode(ENTRY_SEED),
        programInstance.publicKey.toBuffer(),
      ],
      program.programId
    );
    const tx = await program.rpc.verifyEntry({
      accounts: {
        registryContext,
        entry: seededPubkey,
        authority: provider.wallet.publicKey,
      },
    });
    console.log("Your transaction signature", tx);
    const entry = await program.account.entryData.fetch(seededPubkey);
    console.log("Found data: ", entry);
    assert.equal(entry.data, testData);
    assert.equal(
      entry.address.toBase58(),
      programInstance.publicKey.toBase58()
    );
    assert.equal(entry.isVerified, true);
  });

  it("Remove an entry", async () => {
    const [registryContext] = await web3.PublicKey.findProgramAddress(
      [anchor.utils.bytes.utf8.encode(REGISTRY_CONTEXT_SEED)],
      program.programId
    );

    const [seededPubkey] = await web3.PublicKey.findProgramAddress(
      [
        anchor.utils.bytes.utf8.encode(ENTRY_SEED),
        programInstance.publicKey.toBuffer(),
      ],
      program.programId
    );
    const tx = await program.rpc.removeEntry({
      accounts: {
        registryContext,
        entry: seededPubkey,
        authority: provider.wallet.publicKey,
      },
    });
    console.log("Your transaction signature", tx);
    try {
      await program.account.entryData.fetch(seededPubkey);
      throw Error("Expected to get an error");
    } catch (e) {
      // TODO check the error type
    }
  });

  it("Add instance back", async () => {
    const data = "https://fkrok";
    const [registryContext] = await web3.PublicKey.findProgramAddress(
      [anchor.utils.bytes.utf8.encode(REGISTRY_CONTEXT_SEED)],
      program.programId
    );

    const [seededPubkey, bump] = await web3.PublicKey.findProgramAddress(
      [
        anchor.utils.bytes.utf8.encode(ENTRY_SEED),
        programInstance.publicKey.toBuffer(),
      ],
      program.programId
    );

    const tx = await program.rpc.addEntry(
      {
        bump,
        data,
        address: programInstance.publicKey,
      },
      {
        accounts: {
          registryContext,
          entry: seededPubkey,
          creator: provider.wallet.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        },
      }
    );
    console.log("Your transaction signature", tx);
    const entry = await program.account.entryData.fetch(seededPubkey);
    console.log("Found data: ", entry);
    assert.equal(entry.data, data);
    assert.equal(
      entry.address.toBase58(),
      programInstance.publicKey.toBase58()
    );
    assert.equal(entry.isVerified, false);
  });

  it("Cannot add entry again", async () => {
    const data = "https://fkrok";
    const [registryContext] = await web3.PublicKey.findProgramAddress(
      [anchor.utils.bytes.utf8.encode(REGISTRY_CONTEXT_SEED)],
      program.programId
    );

    const [seededPubkey, bump] = await web3.PublicKey.findProgramAddress(
      [
        anchor.utils.bytes.utf8.encode(ENTRY_SEED),
        programInstance.publicKey.toBuffer(),
      ],
      program.programId
    );
    try {
      const tx = await program.rpc.addEntry(
        {
          bump,
          data,
          address: programInstance.publicKey,
        },
        {
          accounts: {
            registryContext,
            entry: seededPubkey,
            authority: provider.wallet.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          },
        }
      );
      throw Error("Expected to get an error");
    } catch (e) {
      // TODO check the error type
    }
  });

  it("Cannot verify entry if not authority", async () => {
    const [registryContext] = await web3.PublicKey.findProgramAddress(
      [anchor.utils.bytes.utf8.encode(REGISTRY_CONTEXT_SEED)],
      program.programId
    );

    const [seededPubkey] = await web3.PublicKey.findProgramAddress(
      [
        anchor.utils.bytes.utf8.encode(ENTRY_SEED),
        programInstance.publicKey.toBuffer(),
      ],
      program.programId
    );

    const nonAuthority = web3.Keypair.generate();

    try {
      const tx = await program.rpc.verifyEntry({
        accounts: {
          registryContext,
          entry: seededPubkey,
          authority: nonAuthority.publicKey,
        },
        signers: [nonAuthority],
      });
      throw Error("Expected to get an error");
    } catch (e) {
      assert.equal(e.code, 300);
    }
  });
});
