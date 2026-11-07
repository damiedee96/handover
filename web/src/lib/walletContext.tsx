/**
 * React context that holds the connected wallet address and provides
 * connect/disconnect helpers.
 */
import React, {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useState,
} from "react";
import { connectWallet, freighterInstalled, WalletState } from "./wallet";

interface WalletContextValue {
  walletState: WalletState;
  connect: () => Promise<void>;
  disconnect: () => void;
}

const WalletContext = createContext<WalletContextValue | null>(null);

export function WalletProvider({ children }: { children: React.ReactNode }) {
  const [walletState, setWalletState] = useState<WalletState>({
    status: "disconnected",
  });

  // Check for existing connection on mount.
  useEffect(() => {
    freighterInstalled().then((installed) => {
      if (!installed) {
        setWalletState({
          status: "error",
          message:
            "Freighter wallet extension not found. Install it from https://freighter.app",
        });
      }
    });
  }, []);

  const connect = useCallback(async () => {
    setWalletState({ status: "connecting" });
    try {
      const address = await connectWallet();
      setWalletState({ status: "connected", address });
    } catch (err) {
      setWalletState({
        status: "error",
        message: err instanceof Error ? err.message : "Unknown error",
      });
    }
  }, []);

  const disconnect = useCallback(() => {
    setWalletState({ status: "disconnected" });
  }, []);

  return (
    <WalletContext.Provider value={{ walletState, connect, disconnect }}>
      {children}
    </WalletContext.Provider>
  );
}

export function useWallet(): WalletContextValue {
  const ctx = useContext(WalletContext);
  if (!ctx) throw new Error("useWallet must be used inside <WalletProvider>");
  return ctx;
}
