import os
from typing import List, Dict, Any, Optional, TypedDict, Annotated, Sequence, Literal, Union
import json
import re
from dotenv import load_dotenv

# Import Semantic Kernel
import semantic_kernel as sk
from semantic_kernel.connectors.ai.google.google_ai.services.google_ai_chat_completion import GoogleAIChatCompletion
from semantic_kernel.functions.kernel_function import KernelFunction
from semantic_kernel.functions.kernel_function_metadata import KernelFunctionMetadata
from semantic_kernel.functions.kernel_arguments import KernelArguments

# Import tools
from backend.tools.neo4j_connector import query_insurance_plan, query_coverage_for_procedure, search_insurance_knowledge_graph
from backend.tools.pinecone_connector import search_insurance_documents, hybrid_search

# Load environment variables
load_dotenv()

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
TOOL_SELECTION_PROMPT = """Given the conversation above, decide if you need to use a tool to answer the user's question.
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
"""

# Response generation prompt
RESPONSE_PROMPT = """Now, provide a comprehensive and helpful response to the user based on the conversation and any tool outputs.
Make sure your response is clear, accurate, and addresses the user's question directly.
If the tool provided useful information, incorporate it into your response.
If the tool didn't provide useful information, do your best to answer based on your general knowledge, but be clear about any limitations.
"""

class SemanticKernelAgent:
    """Agent implementation using Semantic Kernel."""

    def __init__(self):
        """Initialize the Semantic Kernel agent."""
        self.kernel = sk.Kernel()

        # Initialize Google Generative AI
        api_key = os.getenv("GOOGLE_API_KEY", "your-api-key")
        self.kernel.add_service(
            GoogleAIChatCompletion(
                gemini_model_id="gemini-pro",
                api_key=api_key,
                service_id="gemini"
            )
        )

        # Register tools
        self._register_tools()

    def _register_tools(self):
        """Register tools with the kernel."""
        tools = [
            query_insurance_plan,
            query_coverage_for_procedure,
            search_insurance_knowledge_graph,
            search_insurance_documents,
            hybrid_search
        ]

        for tool in tools:
            # Create function metadata
            metadata = KernelFunctionMetadata(
                name=tool.__name__,
                description=tool.__doc__,
                parameters=[
                    {
                        "name": param_name,
                        "description": param_name,
                        "default_value": "",
                        "required": True
                    } for param_name in tool.__code__.co_varnames[:tool.__code__.co_argcount]
                ],
                is_semantic=False
            )

            # Create and register the function
            function = KernelFunction.from_native_method(
                method=tool,
                metadata=metadata
            )

            self.kernel.add_function(function)

    async def should_use_tool(self, messages: List[Dict[str, str]]) -> Dict[str, Any]:
        """Determine if a tool should be used."""
        # Format the conversation history
        conversation = self._format_conversation(messages)

        # Get available tools
        tools_str = self._get_tools_description()

        # Create the prompt
        prompt = f"{SYSTEM_PROMPT}\n\n{conversation}\n\n{TOOL_SELECTION_PROMPT.format(tools=tools_str)}"

        # Invoke the LLM
        result = await self.kernel.invoke_prompt(
            prompt=prompt,
            service_id="gemini"
        )

        response_text = result.result

        # Check if the response contains a tool selection
        if "```json" in response_text:
            json_match = re.search(r"```json\s*(.*?)\s*```", response_text, re.DOTALL)
            if json_match:
                json_str = json_match.group(1)
                try:
                    tool_selection = json.loads(json_str)
                    return tool_selection
                except json.JSONDecodeError:
                    pass

        return {"tool": None, "tool_input": None}

    async def use_tool(self, tool_name: str, tool_input: str) -> str:
        """Use the selected tool."""
        if not tool_name:
            return ""

        try:
            # Invoke the tool
            result = await self.kernel.invoke_function(
                function_name=tool_name,
                arguments=KernelArguments(input=tool_input)
            )

            return str(result)
        except Exception as e:
            return f"Error using tool {tool_name}: {str(e)}"

    async def generate_response(self, messages: List[Dict[str, str]], tool_result: Optional[str] = None) -> str:
        """Generate a response to the user."""
        # Format the conversation history
        conversation = self._format_conversation(messages)

        # Add tool result if available
        if tool_result:
            conversation += f"\nTool Result: {tool_result}\n"

        # Create the prompt
        prompt = f"{SYSTEM_PROMPT}\n\n{conversation}\n\n{RESPONSE_PROMPT}"

        # Invoke the LLM
        result = await self.kernel.invoke_prompt(
            prompt=prompt,
            service_id="gemini"
        )

        return result.result

    def _format_conversation(self, messages: List[Dict[str, str]]) -> str:
        """Format the conversation history for the prompt."""
        conversation = ""
        for message in messages:
            role = message["role"]
            content = message["content"]
            if role == "user":
                conversation += f"User: {content}\n"
            elif role == "assistant":
                conversation += f"Assistant: {content}\n"
        return conversation

    def _get_tools_description(self) -> str:
        """Get a description of available tools."""
        tools = []
        for function in self.kernel.functions.get_functions_view():
            if not function.is_semantic:
                tools.append(f"{function.name}: {function.description}")

        return "\n".join(tools)

# Function to run the agent
async def run_agent(messages: List[Dict[str, str]], user_id: Optional[str] = None) -> Dict[str, Any]:
    """Run the agent with the given messages."""
    # Create the agent
    agent = SemanticKernelAgent()

    # Determine if a tool should be used
    tool_selection = await agent.should_use_tool(messages)

    tool_name = tool_selection.get("tool")
    tool_input = tool_selection.get("tool_input")

    # Use the tool if needed
    tool_result = None
    if tool_name and tool_input:
        # Add a message indicating the tool being used
        messages.append({
            "role": "assistant",
            "content": f"I'll use the {tool_name} tool to help answer your question."
        })

        tool_result = await agent.use_tool(tool_name, tool_input)

    # Generate a response
    response = await agent.generate_response(messages, tool_result)

    # Add the response to the messages
    messages.append({
        "role": "assistant",
        "content": response
    })

    # Return the response
    return {
        "response": response,
        "sources": []  # In a real implementation, we would extract sources from tool outputs
    }
