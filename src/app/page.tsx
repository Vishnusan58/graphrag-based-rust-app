import Chatbot from "../components/Chatbot";

export default function Home() {
  return (
    <div className="min-h-screen flex flex-col items-center justify-center p-4 sm:p-8 bg-gray-100 dark:bg-gray-800 font-[family-name:var(--font-geist-sans)]">
      <header className="w-full max-w-4xl text-center mb-8">
        <h1 className="text-3xl sm:text-4xl font-bold text-blue-600 mb-2">
          Healthcare Insurance Assistant
        </h1>
        <p className="text-gray-600 dark:text-gray-300 text-lg">
          Powered by GraphRAG with Rust Preprocessing
        </p>
      </header>

      <main className="w-full max-w-4xl flex-1 flex flex-col">
        <div className="flex-1 bg-white dark:bg-gray-900 rounded-lg shadow-xl overflow-hidden">
          <Chatbot />
        </div>
      </main>

      <footer className="w-full max-w-4xl mt-8 text-center text-sm text-gray-500 dark:text-gray-400">
        <p>
          Built with Next.js, FastAPI, LangGraph, Neo4j, Pinecone, and Rust
        </p>
      </footer>
    </div>
  );
}
