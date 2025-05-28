# GraphRAG-Powered Healthcare Insurance Assistant

This project is an intelligent AI chatbot that helps users understand their healthcare insurance plans, answers queries about policies, coverage, claims, and processes using GraphRAG for symbolic + semantic reasoning, and integrates a Rust CLI for fast, structured preprocessing of raw documents.

## System Architecture

```
              [User: Web Interface (Next.js)]
                        ↓
            [Backend API: FastAPI (Python)]
                        ↓
            [AI Brain: LangGraph Agent Flow]
              /                          \
 [Graph Reasoning: Neo4j via LangChain]   [Semantic Search: Pinecone]
                        ↑
          [Preprocessed Documents: Output from Rust CLI]
```

## Prerequisites

- Node.js (v18+)
- Python (v3.9+)
- Rust (latest stable)
- Neo4j database (local or cloud)
- Pinecone account
- Google AI API key (for Gemini)

## Setup Instructions

### 1. Clone the repository

```bash
git clone <repository-url>
cd <repository-directory>
```

### 2. Environment Variables Setup

#### Backend (.env file)

Create a `.env` file in the `backend` directory with the following variables:

```
# Neo4j Configuration
NEO4J_URI=bolt://localhost:7687
NEO4J_USERNAME=neo4j
NEO4J_PASSWORD=your-password

# Pinecone Configuration
PINECONE_API_KEY=your-pinecone-api-key
PINECONE_ENVIRONMENT=gcp-starter
PINECONE_INDEX=healthcare-insurance

# Google AI API
GOOGLE_API_KEY=your-google-api-key
```

#### Frontend (.env.local file)

Create a `.env.local` file in the root directory with the following variables:

```
BACKEND_API_URL=http://localhost:8000
```

### 3. Install Dependencies

#### Frontend (Next.js)

```bash
npm install
```

#### Backend (FastAPI)

```bash
cd backend
pip install -r requirements.txt
```

#### Rust Preprocessor

```bash
cd rust-preprocessor
cargo build --release
```

## Running the Application

### 1. Start the Backend Server

```bash
cd backend
uvicorn main:app --reload
```

The backend will be available at http://localhost:8000

### 2. Start the Frontend Development Server

```bash
npm run dev
```

The frontend will be available at http://localhost:3000

### 3. Using the Rust Preprocessor

The Rust preprocessor is a CLI tool for preprocessing raw documents before ingestion into the system.

```bash
cd rust-preprocessor
cargo run -- process --input=input\doc.txt --output=processed\output.json --format=json --doc-type=plan
```

Options:
- `--input`: Path to the input document
- `--output`: Path to the output file
- `--format`: Output format (json or csv)
- `--doc-type`: Document type (plan, policy, claim)

## Project Structure

- `src/`: Next.js frontend
  - `app/`: Next.js app router
  - `components/`: React components
- `backend/`: FastAPI backend
  - `agent/`: LangGraph agent implementation
  - `tools/`: LangChain tools for Neo4j and Pinecone
- `rust-preprocessor/`: Rust CLI for document preprocessing
  - `src/`: Rust source code

## Development Workflow

1. Preprocess documents using the Rust CLI
2. Load the processed data into Neo4j and Pinecone
3. Start the backend server
4. Start the frontend development server
5. Interact with the chatbot through the web interface

## Troubleshooting

- If you encounter connection issues with Neo4j or Pinecone, verify your API keys and connection strings in the `.env` file.
- For Rust preprocessor issues, check that you have the latest stable Rust version installed.
- If the frontend can't connect to the backend, ensure the backend server is running and the `BACKEND_API_URL` is correctly set in `.env.local`.
