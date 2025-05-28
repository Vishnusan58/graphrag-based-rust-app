from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
from typing import List, Dict, Any, Optional
import os
from dotenv import load_dotenv

# Load environment variables
load_dotenv()

# Initialize FastAPI app
app = FastAPI(title="Healthcare Insurance Assistant API")

# Add CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # In production, replace with specific origins
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Define request and response models
class ChatMessage(BaseModel):
    role: str  # 'user' or 'assistant'
    content: str

class ChatRequest(BaseModel):
    messages: List[ChatMessage]
    user_id: Optional[str] = None

class ChatResponse(BaseModel):
    response: str
    sources: Optional[List[Dict[str, Any]]] = None

# Import the agent implementation
# Uncomment one of the following lines to choose the implementation
from backend.agent.semantic_agent import run_agent  # Semantic Kernel implementation
# from backend.agent.agent import run_agent  # LangGraph implementation

async def process_with_agent(messages: List[ChatMessage], user_id: Optional[str] = None):
    """Process messages with the agent (Semantic Kernel or LangGraph)."""
    # Convert ChatMessage objects to dictionaries for the agent
    message_dicts = [{"role": msg.role, "content": msg.content} for msg in messages]

    # Run the agent
    result = await run_agent(message_dicts, user_id)

    # Return the response
    return result

@app.get("/")
async def root():
    return {"message": "Healthcare Insurance Assistant API is running"}

@app.post("/chat", response_model=ChatResponse)
async def chat(request: ChatRequest):
    try:
        result = await process_with_agent(request.messages, request.user_id)
        return result
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

# Run with: uvicorn main:app --reload
if __name__ == "__main__":
    import uvicorn
    uvicorn.run("main:app", host="0.0.0.0", port=8000, reload=True)
