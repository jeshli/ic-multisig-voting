import { useEffect, useState } from "react";
import { connect }             from "./wallet";

export default function App() {
  const [actor, setActor]   = useState<any>();
  const [counter, setCnt]   = useState<bigint | null>(null);
  const [loading, setLoad ] = useState(false);

  // 1-click connect
  async function login() {
    const a = await connect();
    setActor(a);
  }

  async function refresh() {
    if (!actor) return;
    setCnt(await actor.get());
  }

  async function addOne() {
    if (!actor) return;
    setLoad(true);
    await actor.increment(BigInt(1));   // Oisy pops consent dialog
    await refresh();
    setLoad(false);
  }

  useEffect(() => { if (actor) refresh(); }, [actor]);

  return (
    <main style={{fontFamily:"sans-serif", padding:"2rem"}}>
      {!actor && <button onClick={login}>Connect Oisy Wallet</button>}
      {actor && (
        <>
          <h1>Counter: {counter?.toString() ?? "--"}</h1>
          <button disabled={loading} onClick={addOne}>
            {loading ? "Waiting for wallet…" : "Increment"}
          </button>
        </>
      )}
    </main>
  );
}
