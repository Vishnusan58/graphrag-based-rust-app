# Quick Start Guide: Testing the Chat Functionality

This guide provides simplified instructions for quickly getting the Healthcare Insurance Assistant up and running to test the chat functionality.

## Prerequisites

Ensure you have:
- Node.js (v18+)
- Python (v3.9+)
- All dependencies installed (see main README.md for detailed setup)

## 1. Start the Backend Server

Open a terminal and run:

```bash
# From the project root directory
python -m uvicorn main:app --reload
```

This will start the FastAPI backend server at http://localhost:8000.

The backend is configured to use the Semantic Kernel implementation by default (see `backend/main.py`).

## 2. Start the Frontend Development Server

Open another terminal and run:

```bash
# From the project root directory
npm run dev
```

This will start the Next.js development server at http://localhost:3000.

## 3. Testing the Chat Interface

1. Open your browser and navigate to http://localhost:3000
2. You should see the Healthcare Insurance Assistant interface with a welcome message
3. Type a question about healthcare insurance in the chat input at the bottom of the screen
4. Press Enter or click the send button (paper airplane icon)
5. Wait for the assistant to respond

## Example Questions to Try

- "What does my insurance plan cover?"
- "Is physical therapy covered under my plan?"
- "How do I file a claim for a recent doctor visit?"
- "What's the difference between in-network and out-of-network providers?"
- "What is my deductible for prescription medications?"

## Troubleshooting

- **Backend Connection Issues**: Make sure the backend server is running and check the terminal for any error messages
- **Frontend Not Loading**: Ensure the npm run dev command completed successfully
- **No Response from Chat**: Check the backend terminal for errors; it might be an issue with the API keys or service connections
- **CORS Errors**: Ensure both servers are running and the proxy in next.config.js is correctly set up

## Switching Agent Implementations

The project includes two different agent implementations:

1. **Semantic Kernel Implementation** (default): Uses Semantic Kernel
2. **LangGraph Implementation**: Uses LangGraph and LangChain

To switch between them, edit the import statement in `backend/main.py`:

```python
# Import the agent implementation
# Uncomment one of the following lines to choose the implementation
from backend.agent.semantic_agent import run_agent  # Semantic Kernel implementation
# from backend.agent.agent import run_agent  # LangGraph implementation
```