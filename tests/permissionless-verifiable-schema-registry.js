const anchor = require("@project-serum/anchor");
const web3 = require("@solana/web3.js");
const assert = require("assert");
const borsh = require("borsh");

const REGISTRY_CONFIG = "registry-config";
const SCHEMA_SEED = "schema";
const ENTRY_SEED = "entry-seed";

class BorshTokenData {
  token_symbol = "";
  token_name = "";
  token_logo_url = "";
  token_tags = [""];
  token_extensions = [[""]];
  constructor(fields) {
    if (fields != null) {
      this.token_symbol = fields.token_symbol;
      this.token_name = fields.token_name;
      this.token_logo_url = fields.token_logo_url;
      this.token_tags = fields.token_tags;
      this.token_extensions = fields.token_extensions;
    }
  }
}
const BorshTokenDataSchema = new Map([
  [
    BorshTokenData,
    {
      kind: "struct",
      fields: [
        ["token_symbol", "String"],
        ["token_name", "String"],
        ["token_logo_url", "String"],
        ["token_tags", ["String"]],
        ["token_extensions", [["String"]]],
      ],
    },
  ],
]);

describe("Registry Tests", () => {
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.PermissionlessVerifiableSchemaRegistry;
  const programInstance = web3.Keypair.generate();
  const serializedTokenData = borsh.serialize(
    BorshTokenDataSchema,
    new BorshTokenData({
      token_symbol: "TEST",
      token_name: "test",
      token_logo_url: "https://fkrofkr",
      token_tags: ["tag1"],
      token_extensions: [["attr", "value"]],
    })
  );

  it("Initializes the registry", async () => {
    const [registryConfig, bump] = await web3.PublicKey.findProgramAddress(
      [anchor.utils.bytes.utf8.encode(REGISTRY_CONFIG)],
      program.programId
    );

    const tx = await program.rpc.init(
      { bump, entrySeed: ENTRY_SEED, permissionless_add: true },
      {
        accounts: {
          registryConfig,
          authority: provider.wallet.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        },
      }
    );
    console.log("Your transaction signature", tx);
    const data = await program.account.registryConfig.fetch(registryConfig);
    console.log("Found data: ", data);
    assert.equal(
      data.authority.toBase58(),
      provider.wallet.publicKey.toBase58()
    );
    assert.equal(data.entrySeed, ENTRY_SEED);
  });

  it("Add entry", async () => {
    const [registryConfig] = await web3.PublicKey.findProgramAddress(
      [anchor.utils.bytes.utf8.encode(REGISTRY_CONFIG)],
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
        data: serializedTokenData,
        schemaVersion: 0,
        primaryKey: programInstance.publicKey.toBytes(),
      },
      {
        accounts: {
          registryConfig,
          entry: seededPubkey,
          creator: provider.wallet.publicKey,
          authority: provider.wallet.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        },
      }
    );
    console.log("Your transaction signature", tx);
    const entry = await program.account.entryData.fetch(seededPubkey);
    console.log("Found data: ", entry);
    assert.deepStrictEqual(
      borsh.deserialize(BorshTokenDataSchema, BorshTokenData, entry.data),
      borsh.deserialize(
        BorshTokenDataSchema,
        BorshTokenData,
        serializedTokenData
      )
    );
    assert.equal(
      new web3.PublicKey(entry.primaryKey).toBase58(),
      programInstance.publicKey.toBase58()
    );
    assert.equal(entry.isVerified, false);
  });

  it("Verify an entry", async () => {
    const [registryConfig] = await web3.PublicKey.findProgramAddress(
      [anchor.utils.bytes.utf8.encode(REGISTRY_CONFIG)],
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
        registryConfig,
        entry: seededPubkey,
        authority: provider.wallet.publicKey,
      },
    });
    console.log("Your transaction signature", tx);
    const entry = await program.account.entryData.fetch(seededPubkey);
    console.log("Found data: ", entry);
    assert.deepStrictEqual(
      borsh.deserialize(BorshTokenDataSchema, BorshTokenData, entry.data),
      borsh.deserialize(
        BorshTokenDataSchema,
        BorshTokenData,
        serializedTokenData
      )
    );
    assert.equal(
      new web3.PublicKey(entry.primaryKey).toBase58(),
      programInstance.publicKey.toBase58()
    );
    assert.equal(entry.isVerified, true);
  });

  it("Remove an entry", async () => {
    const [registryConfig] = await web3.PublicKey.findProgramAddress(
      [anchor.utils.bytes.utf8.encode(REGISTRY_CONFIG)],
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
        registryConfig,
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

  it("Add entry back", async () => {
    const [registryConfig] = await web3.PublicKey.findProgramAddress(
      [anchor.utils.bytes.utf8.encode(REGISTRY_CONFIG)],
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
        data: serializedTokenData,
        schemaVersion: 0,
        primaryKey: programInstance.publicKey.toBytes(),
      },
      {
        accounts: {
          registryConfig,
          entry: seededPubkey,
          creator: provider.wallet.publicKey,
          authority: provider.wallet.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        },
      }
    );
    console.log("Your transaction signature", tx);
    const entry = await program.account.entryData.fetch(seededPubkey);
    console.log("Found data: ", entry);
    assert.deepStrictEqual(
      borsh.deserialize(BorshTokenDataSchema, BorshTokenData, entry.data),
      borsh.deserialize(
        BorshTokenDataSchema,
        BorshTokenData,
        serializedTokenData
      )
    );
    assert.equal(
      new web3.PublicKey(entry.primaryKey).toBase58(),
      programInstance.publicKey.toBase58()
    );
    assert.equal(entry.isVerified, false);
  });

  it("Cannot add entry again", async () => {
    const data = "https://fkrok";
    const [registryConfig] = await web3.PublicKey.findProgramAddress(
      [anchor.utils.bytes.utf8.encode(REGISTRY_CONFIG)],
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
          data: serializedTokenData,
          schemaVersion: 0,
          primaryKey: programInstance.publicKey.toBytes(),
        },
        {
          accounts: {
            registryConfig,
            entry: seededPubkey,
            creator: provider.wallet.publicKey,
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
    const [registryConfig] = await web3.PublicKey.findProgramAddress(
      [anchor.utils.bytes.utf8.encode(REGISTRY_CONFIG)],
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
          registryConfig,
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

  // it("Add schema for entry", async () => {
  //   const [registryConfig] = await web3.PublicKey.findProgramAddress(
  //     [anchor.utils.bytes.utf8.encode(REGISTRY_CONFIG)],
  //     program.programId
  //   );

  //   const [seededPubkey, bump] = await web3.PublicKey.findProgramAddress(
  //     [anchor.utils.bytes.utf8.encode(SCHEMA_SEED), [1]],
  //     program.programId
  //   );

  //   console.log(JSON.stringify(Array.from(BorshTokenDataSchema.entries())));
  //   console.log(
  //     Buffer.from(JSON.stringify(Array.from(BorshTokenDataSchema.entries())))
  //   );

  //   const tx = await program.rpc.addSchema(
  //     {
  //       bump,
  //       data: Buffer.from(
  //         JSON.stringify(Array.from(BorshTokenDataSchema.entries()))
  //       ),
  //     },
  //     {
  //       accounts: {
  //         registryConfig,
  //         schema: seededPubkey,
  //         creator: provider.wallet.publicKey,
  //         systemProgram: anchor.web3.SystemProgram.programId,
  //       },
  //     }
  //   );
  //   console.log("Your transaction signature", tx);
  //   const schema = await program.account.schemaData.fetch(seededPubkey);
  //   console.log("Found data: ", schema);
  //   console.log(new Map(JSON.parse(schema.data)));

  //   assert.deepStrictEqual(
  //     BorshTokenDataSchema,
  //     new Map(JSON.parse(schema.data))
  //   );
  // });
});
