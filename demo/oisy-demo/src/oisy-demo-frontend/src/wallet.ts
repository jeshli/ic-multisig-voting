import { Wallet }            from "@dfinity/oisy-wallet-signer";
import { idlFactory }        from 'declarations/oisy-demo-backend';
import { HttpAgent }         from "@dfinity/agent";

export async function connect() {
  // ➊ Spin up Oisy + II
  const wallet = await Wallet.init({
    identityProvider: import.meta.env.VITE_II_URL,
  });

  // ➋ Ask once for all permissions we’ll need
  await wallet.requestPermissions({
    canisterId: import.meta.env.VITE_CANISTER_ID,
    methodNames: ["increment", "get"],
  });

  // ➌ Build an authenticated actor that pipes every call through Oisy
  return wallet.createActor<typeof idlFactory>(
    import.meta.env.VITE_CANISTER_ID,
    idlFactory,
    {
      agentFactory: () =>
        new HttpAgent({ host: "http://localhost:4943" }), // local replica
    }
  );
}
