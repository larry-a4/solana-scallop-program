import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { Scallop } from '../target/types/scallop';

describe('scallop', () => {

  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.Scallop as Program<Scallop>;

  it('Is initialized!', async () => {
    // Add your test here.
    const tx = await program.rpc.initialize({});
    console.log("Your transaction signature", tx);
  });
});
