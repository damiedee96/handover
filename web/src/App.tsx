import { BrowserRouter, Route, Routes } from "react-router-dom";
import { WalletProvider } from "./lib/walletContext";
import { NavBar } from "./components/NavBar";
import { HomePage } from "./pages/HomePage";
import { SenderPage } from "./pages/SenderPage";
import { CourierPage } from "./pages/CourierPage";
import { RecipientPage } from "./pages/RecipientPage";
import { StatusPage } from "./pages/StatusPage";

export function App() {
  return (
    <WalletProvider>
      <BrowserRouter>
        <NavBar />
        <Routes>
          <Route path="/" element={<HomePage />} />
          <Route path="/send" element={<SenderPage />} />
          <Route path="/receive" element={<CourierPage />} />
          <Route path="/recipient" element={<RecipientPage />} />
          <Route path="/status" element={<StatusPage />} />
        </Routes>
      </BrowserRouter>
    </WalletProvider>
  );
}
