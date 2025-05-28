import os
from typing import List, Dict, Any, Optional, TypedDict, Annotated, Sequence, Literal, Union
from langchain_core.messages import HumanMessage, AIMessage, SystemMessage, BaseMessage
from langchain_core.prompts import ChatPromptTemplate, MessagesPlaceholder
from langchain_core.output_parsers import StrOutputParser
from langchain_google_genai import ChatGoogleGenerativeAI
from langgraph.graph import StateGraph, END
from langgraph.prebuilt import ToolNode
from langgraph.prebuilt import ToolExecutor
from langgraph.prebuilt import ToolInvocation

import operator
from dotenv import load_dotenv

# Import tools
from backend.tools.neo4j_connector import query_insurance_plan, query_coverage_for_procedure, search_insurance_knowledge_graph
from backend.tools.pinecone_connector import search_insurance_documents, hybrid_search

# Load environment variables
load_dotenv()

# Define state
class AgentState(TypedDict):
    messages: Annotated[Sequence[BaseMessage], operator.add]
    tools: List[Dict[str, Any]]

# Initialize LLM
def get_llm():
    """Initialize the Gemini LLM."""
    api_key = os.getenv("GOOGLE_API_KEY", "your-api-key")
    return ChatGoogleGenerativeAI(
        model="gemini-pro",
        google_api_key=api_key,
        temperature=0.2,
        convert_system_message_to_human=True
    )

# System prompt
SYSTEM_PROMPT = """You are a helpful Healthcare Insurance Assistant. 
Your job is to help users understand their healthcare insurance plans, coverage, claims, and processes.

You have access to the following tools:
1. query_insurance_plan: Get information about a specific insurance plan
2. query_coverage_for_procedure: Check if a specific medical procedure is covered
3. search_insurance_knowledge_graph: Search the insurance knowledge graph
4. search_insurance_documents: Search for information in insurance documents
5. hybrid_search: Perform a combined search using both graph and vector search

Always be professional, accurate, and helpful. If you don't know the answer, say so clearly.
When providing information about coverage, be clear about what is covered and what is excluded.
"""

# Tool selection prompt
TOOL_SELECTION_PROMPT = ChatPromptTemplate.from_messages([
    SystemMessage(content=SYSTEM_PROMPT),
    MessagesPlaceholder(variable_name="messages"),
    SystemMessage(content="""Given the conversation above, decide if you need to use a tool to answer the user's question.
If you need to use a tool, select the most appropriate tool and provide the necessary input.
If you don't need to use a tool, respond directly to the user.

Available tools:
{tools}

To use a tool, respond in the following format:
```json
{{"tool": "tool_name", "tool_input": "input_value"}}
```

If no tool is needed, respond with:
```json
{{"tool": null, "tool_input": null}}
```
""")
])

# Response generation prompt
RESPONSE_PROMPT = ChatPromptTemplate.from_messages([
    SystemMessage(content=SYSTEM_PROMPT),
    MessagesPlaceholder(variable_name="messages"),
    SystemMessage(content="""Now, provide a comprehensive and helpful response to the user based on the conversation and any tool outputs.
Make sure your response is clear, accurate, and addresses the user's question directly.
If the tool provided useful information, incorporate it into your response.
If the tool didn't provide useful information, do your best to answer based on your general knowledge, but be clear about any limitations.
""")
])

# Define the agent nodes
def should_use_tool(state: AgentState) -> Union[Literal["use_tool"], Literal["generate_response"]]:
    """Determine if a tool should be used."""
    llm = get_llm()
    tools_str = "\n".join([f"{tool['name']}: {tool['description']}" for tool in state["tools"]])

    messages = state["messages"]
    response = llm.invoke(
        TOOL_SELECTION_PROMPT.format(
            messages=messages,
            tools=tools_str
        )
    )

    # Parse the response to extract tool and input
    response_text = response.content

    # Check if the response contains a tool selection
    if "```json" in response_text:
        import json
        import re

        # Extract JSON from the response
        json_match = re.search(r"```json\s*(.*?)\s*```", response_text, re.DOTALL)
        if json_match:
            json_str = json_match.group(1)
            try:
                tool_selection = json.loads(json_str)
                if tool_selection.get("tool"):
                    # Add the tool selection as a message
                    state["messages"].append(AIMessage(content=f"I'll use the {tool_selection['tool']} tool to help answer your question."))
                    return "use_tool"
            except json.JSONDecodeError:
                pass

    return "generate_response"

def use_tool(state: AgentState) -> AgentState:
    """Use the selected tool."""
    llm = get_llm()
    tools_str = "\n".join([f"{tool['name']}: {tool['description']}" for tool in state["tools"]])

    messages = state["messages"]
    response = llm.invoke(
        TOOL_SELECTION_PROMPT.format(
            messages=messages,
            tools=tools_str
        )
    )

    # Parse the response to extract tool and input
    response_text = response.content

    # Initialize tool executor
    tool_executor = ToolExecutor(state["tools"])

    # Extract JSON from the response
    import json
    import re

    json_match = re.search(r"```json\s*(.*?)\s*```", response_text, re.DOTALL)
    if json_match:
        json_str = json_match.group(1)
        try:
            tool_selection = json.loads(json_str)
            tool_name = tool_selection.get("tool")
            tool_input = tool_selection.get("tool_input")

            if tool_name and tool_input:
                # Execute the tool
                tool_result = tool_executor.invoke(
                    ToolInvocation(
                        tool=tool_name,
                        tool_input=tool_input
                    )
                )

                # Add the tool result as a message
                state["messages"].append(
                    AIMessage(content=f"Tool Result: {tool_result}")
                )
        except json.JSONDecodeError:
            # If JSON parsing fails, continue without using a tool
            pass

    return state

def generate_response(state: AgentState) -> AgentState:
    """Generate a response to the user."""
    llm = get_llm()

    response = llm.invoke(
        RESPONSE_PROMPT.format(
            messages=state["messages"]
        )
    )

    # Add the response to the messages
    state["messages"].append(response)

    return state

# Create the agent graph
def create_agent():
    """Create the LangGraph agent."""
    # Define the tools
    tools = [
        query_insurance_plan,
        query_coverage_for_procedure,
        search_insurance_knowledge_graph,
        search_insurance_documents,
        hybrid_search
    ]

    # Convert tools to the format expected by the agent
    formatted_tools = []
    for tool in tools:
        formatted_tools.append({
            "name": tool.__name__,
            "description": tool.__doc__,
            "func": tool
        })

    # Create the graph
    workflow = StateGraph(AgentState)

    # Add nodes
    workflow.add_node("should_use_tool", should_use_tool)
    workflow.add_node("use_tool", use_tool)
    workflow.add_node("generate_response", generate_response)

    # Add edges
    workflow.add_conditional_edges(
        "should_use_tool",
        {
            "use_tool": "use_tool",
            "generate_response": "generate_response"
        }
    )
    workflow.add_edge("use_tool", "should_use_tool")
    workflow.add_edge("generate_response", END)

    # Set the entry point
    workflow.set_entry_point("should_use_tool")

    # Compile the graph
    return workflow.compile(), formatted_tools

# Function to run the agent
async def run_agent(messages: List[Dict[str, str]], user_id: Optional[str] = None) -> Dict[str, Any]:
    """Run the agent with the given messages."""
    # Create the agent
    agent, tools = create_agent()

    # Convert messages to LangChain format
    lc_messages = []
    for message in messages:
        if message["role"] == "user":
            lc_messages.append(HumanMessage(content=message["content"]))
        elif message["role"] == "assistant":
            lc_messages.append(AIMessage(content=message["content"]))

    # Run the agent
    result = agent.invoke({
        "messages": lc_messages,
        "tools": tools
    })

    # Extract the final response
    final_message = result["messages"][-1]

    # Return the response
    return {
        "response": final_message.content,
        "sources": []  # In a real implementation, we would extract sources from tool outputs
    }
