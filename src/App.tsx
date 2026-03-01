import { BrowserRouter, Routes, Route } from "react-router-dom";
import { Toaster } from "sonner";
import { AppProvider } from "./context/AppContext";
import { Layout } from "./components/Layout";
import { Dashboard } from "./views/Dashboard";
import { MySkills } from "./views/MySkills";
import { InstallSkills } from "./views/InstallSkills";
import { Settings } from "./views/Settings";

function App() {
  return (
    <AppProvider>
      <BrowserRouter>
        <Routes>
          <Route element={<Layout />}>
            <Route path="/" element={<Dashboard />} />
            <Route path="/my-skills" element={<MySkills />} />
            <Route path="/install" element={<InstallSkills />} />
            <Route path="/settings" element={<Settings />} />
          </Route>
        </Routes>
      </BrowserRouter>
      <Toaster
        theme="dark"
        position="bottom-right"
        toastOptions={{
          style: {
            background: "#1C1C1C",
            border: "1px solid #2A2A2A",
            color: "#FAFAFA",
          },
        }}
      />
    </AppProvider>
  );
}

export default App;
