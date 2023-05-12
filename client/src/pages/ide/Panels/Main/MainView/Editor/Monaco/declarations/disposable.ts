import * as monaco from "monaco-editor";
import { Idl } from "@project-serum/anchor";

import { declareModule } from "./helper";
import {
  PgDisposable,
  PgProgramInfo,
  PgWallet,
} from "../../../../../../../../utils/pg";

/**
 * Declare types that can change based on outside events.
 *
 * For example, `pg.wallet` will be `undefined` when the wallet is not connected.
 *
 * @returns a dispose function to dispose all
 */
export const declareDisposableTypes = (): PgDisposable => {
  addLib("default", require("./raw/pg.raw.d.ts"));

  // Program id
  const programIdChange = PgProgramInfo.onDidChangePk((programId) => {
    addLib(
      "program-id",
      `/** Your program public key from playground */\nconst PROGRAM_ID: ${
        programId ? "web3.PublicKey" : "undefined"
      };`
    );
  });

  // Anchor program
  const idlChange = PgProgramInfo.onDidChangeIdl((idl) => {
    if (idl) {
      addLib(
        "program",
        `/** Your Anchor program */\nconst program: anchor.Program<${JSON.stringify(
          convertIdl(idl)
        )}>;`
      );
    }
  });

  // Playground wallet
  const walletChange = PgWallet.onDidUpdateConnection((connected) => {
    addLib(
      "wallet",
      `
  /**
   * Playground wallet.
   *
   * NOTE: You can toggle connection with \`connect\` command.
   */
  const wallet: ${connected ? "PgWallet" : " undefined"};
`
    );
  });

  return {
    dispose: () => {
      programIdChange.dispose();
      idlChange.dispose();
      walletChange.dispose();
    },
  };
};

/** Disposable types */
type DisposableType = "default" | "program-id" | "program" | "wallet";

/** Caching the disposables in order to get rid of the old declarations */
const disposableCache: { [key in DisposableType]?: monaco.IDisposable } = {};

/** Add declaration file and remove the old one if it exists */
const addLib = (disposable: DisposableType, lib: string) => {
  disposableCache[disposable]?.dispose();
  disposableCache[disposable] =
    monaco.languages.typescript.typescriptDefaults.addExtraLib(
      lib.includes("declare module")
        ? lib
        : declareModule("solana-playground", lib)
    );
};

/**
 * Convert Anchor IDL's account names into camelCase to be used accuretely for types
 *
 * @param idl Anchor IDL
 * @returns converted Anchor IDL
 */
const convertIdl = (idl: Idl) => {
  if (!idl.accounts) return idl;

  let newIdl: Idl = { ...idl, accounts: [] };

  for (const account of idl.accounts) {
    newIdl.accounts!.push({
      ...account,
      name: account.name[0].toLowerCase() + account.name.substring(1),
    });
  }

  return newIdl;
};
