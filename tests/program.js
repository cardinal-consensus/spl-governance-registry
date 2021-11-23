const anchor = require("@project-serum/anchor");
const web3 = require("@solana/web3.js");
const assert = require("assert");

const REGISTRY_CONTEXT_SEED = "registry-context";
const PDA_PREFIX = "governance-program";

describe("Registry Tests", () => {
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.Governanceregistry;
  const programInstance = web3.Keypair.generate();
  const testName = "test name";

  it("Initializes the registry", async () => {
    const [registryContext, bump] = await web3.PublicKey.findProgramAddress(
      [anchor.utils.bytes.utf8.encode(REGISTRY_CONTEXT_SEED)],
      program.programId
    );

    const tx = await program.rpc.init(
      { bump },
      {
        accounts: {
          registryContext,
          authority: provider.wallet.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        },
      }
    );
    console.log("Your transaction signature", tx);
    const data = await program.account.registryContextAccount.fetch(
      registryContext
    );
    console.log("Found data: ", data);
    assert.equal(
      data.authority.toBase58(),
      provider.wallet.publicKey.toBase58()
    );
  });

  it("Add instance", async () => {
    const [seededPubkey, bump] = await web3.PublicKey.findProgramAddress(
      [
        anchor.utils.bytes.utf8.encode(PDA_PREFIX),
        programInstance.publicKey.toBuffer(),
      ],
      program.programId
    );

    const tx = await program.rpc.registerProgramInstance(
      {
        name: testName,
        programAddress: programInstance.publicKey,
        bump,
        seed: programInstance.publicKey.toBuffer(),
      },
      {
        accounts: {
          programInstance: seededPubkey,
          authority: provider.wallet.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        },
      }
    );
    console.log("Your transaction signature", tx);
    const data = await program.account.governanceProgramAccount.fetch(
      seededPubkey
    );
    console.log("Found data: ", data);
    assert.equal(data.name, testName);
    assert.equal(
      data.programAddress.toBase58(),
      programInstance.publicKey.toBase58()
    );
    assert.equal(data.isVerified, false);
  });

  it("Verify an instance", async () => {
    const [registryContext] = await web3.PublicKey.findProgramAddress(
      [anchor.utils.bytes.utf8.encode(REGISTRY_CONTEXT_SEED)],
      program.programId
    );

    const [seededPubkey] = await web3.PublicKey.findProgramAddress(
      [
        anchor.utils.bytes.utf8.encode(PDA_PREFIX),
        programInstance.publicKey.toBuffer(),
      ],
      program.programId
    );
    const tx = await program.rpc.verifyProgramInstance({
      accounts: {
        programInstance: seededPubkey,
        registryContext,
        authority: provider.wallet.publicKey,
      },
    });
    console.log("Your transaction signature", tx);
    const data = await program.account.governanceProgramAccount.fetch(
      seededPubkey
    );
    console.log("Found data: ", data);
    assert.equal(data.name, testName);
    assert.equal(
      data.programAddress.toBase58(),
      programInstance.publicKey.toBase58()
    );
    assert.equal(data.isVerified, true);
  });

  it("Remove an instance", async () => {
    const [registryContext] = await web3.PublicKey.findProgramAddress(
      [anchor.utils.bytes.utf8.encode(REGISTRY_CONTEXT_SEED)],
      program.programId
    );

    const [seededPubkey] = await web3.PublicKey.findProgramAddress(
      [
        anchor.utils.bytes.utf8.encode(PDA_PREFIX),
        programInstance.publicKey.toBuffer(),
      ],
      program.programId
    );
    const tx = await program.rpc.removeProgramInstance({
      accounts: {
        registryContext,
        programInstance: seededPubkey,
        authority: provider.wallet.publicKey,
      },
    });
    console.log("Your transaction signature", tx);
    try {
      await program.account.governanceProgramAccount.fetch(seededPubkey);
      throw Error("Expected to get an error");
    } catch (e) {
      // TODO check the error type somehow
    }
  });

  it("Add instance back", async () => {
    const [seededPubkey, bump] = await web3.PublicKey.findProgramAddress(
      [
        anchor.utils.bytes.utf8.encode(PDA_PREFIX),
        programInstance.publicKey.toBuffer(),
      ],
      program.programId
    );

    const tx = await program.rpc.registerProgramInstance(
      {
        name: "test2",
        programAddress: programInstance.publicKey,
        bump,
        seed: programInstance.publicKey.toBuffer(),
      },
      {
        accounts: {
          programInstance: seededPubkey,
          authority: provider.wallet.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        },
      }
    );
    console.log("Your transaction signature", tx);
    const data = await program.account.governanceProgramAccount.fetch(
      seededPubkey
    );
    console.log("Found data: ", data);
    assert.equal(data.name, "test2");
    assert.equal(
      data.programAddress.toBase58(),
      programInstance.publicKey.toBase58()
    );
    assert.equal(data.isVerified, false);
  });

  it("Cannot add instance again", async () => {
    const [seededPubkey, bump] = await web3.PublicKey.findProgramAddress(
      [
        anchor.utils.bytes.utf8.encode(PDA_PREFIX),
        programInstance.publicKey.toBuffer(),
      ],
      program.programId
    );
    try {
      const tx = await program.rpc.registerProgramInstance(
        {
          name: "test3",
          programAddress: programInstance.publicKey,
          bump,
          seed: programInstance.publicKey.toBuffer(),
        },
        {
          accounts: {
            programInstance: seededPubkey,
            authority: provider.wallet.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          },
        }
      );
      throw Error("Expected to get an error");
    } catch (e) {
      // TODO check the error type somehow
    }
  });
});
