# Agent Implementations

This directory contains two different implementations of the Healthcare Insurance Assistant agent:

1. **LangGraph Implementation** (`agent.py`): Uses LangGraph and LangChain to create a state machine-based agent that can help users with healthcare insurance questions.

2. **Semantic Kernel Implementation** (`semantic_agent.py`): Uses Semantic Kernel to create an agent with the same functionality as the LangGraph implementation.

## Switching Between Implementations

You can switch between the two implementations by editing the import statement in `backend/main.py`:

```python
# Import the agent implementation
# Uncomment one of the following lines to choose the implementation
from backend.agent.semantic_agent import run_agent  # Semantic Kernel implementation
# from backend.agent.agent import run_agent  # LangGraph implementation
```

## Implementation Details

### LangGraph Implementation

The LangGraph implementation uses a state machine approach with nodes for:
- Deciding whether to use a tool
- Using a tool
- Generating a response

It uses Google's Gemini Pro model as the LLM and has access to several tools for querying insurance plans, coverage, and searching knowledge graphs and documents.

### Semantic Kernel Implementation

The Semantic Kernel implementation provides the same functionality as the LangGraph implementation but uses Semantic Kernel's API instead of LangGraph's. It includes:

- A `SemanticKernelAgent` class that initializes the Semantic Kernel, registers tools, and provides methods for determining if a tool should be used, using a tool, and generating a response.
- The same system prompt, tool selection prompt, and response generation prompt as the LangGraph implementation.
- A `run_agent` function that serves as the main entry point, similar to the LangGraph implementation.

## Dependencies

Both implementations require their respective dependencies to be installed:

- LangGraph Implementation: `langgraph`, `langchain`, `langchain-core`, etc.
- Semantic Kernel Implementation: `semantic-kernel`

These dependencies are listed in `backend/requirements.txt`.