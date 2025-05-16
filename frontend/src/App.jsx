import Dashboard from "./pages/Dashboard";

const App = () => {
  return (
    <div className="min-h-screen bg-gray-100">
      <header className="bg-slate-800 text-gray-100 p-4 shadow-md flex items-center justify-center">
        <div className="flex text-center space-x-3">
        <img src="/images/logo.svg" alt="AWP Logo" className="h-8 w-auto" />
        <h1 className="text-xl md:text-2xl font-semibold">
          <a href="/">AWP Cloud Dashboard</a>
        </h1>
   </div>
   </header>
      <main className="max-w-6xl mx-auto py-6">
        <Dashboard />
      </main>
    </div>
  );
};

export default App;