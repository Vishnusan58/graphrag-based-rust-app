import os
from typing import List, Dict, Any, Optional
from neo4j import GraphDatabase
from langchain_core.tools import tool
from dotenv import load_dotenv

# Load environment variables
load_dotenv()

class Neo4jConnector:
    """Connector for Neo4j graph database."""
    
    def __init__(self):
        """Initialize Neo4j connection."""
        self.uri = os.getenv("NEO4J_URI", "bolt://localhost:7687")
        self.username = os.getenv("NEO4J_USERNAME", "neo4j")
        self.password = os.getenv("NEO4J_PASSWORD", "password")
        self.driver = None
        
    def connect(self):
        """Connect to Neo4j database."""
        if not self.driver:
            self.driver = GraphDatabase.driver(
                self.uri, 
                auth=(self.username, self.password)
            )
        return self.driver
    
    def close(self):
        """Close the Neo4j connection."""
        if self.driver:
            self.driver.close()
            self.driver = None
    
    def run_query(self, query: str, params: Dict[str, Any] = None) -> List[Dict[str, Any]]:
        """Run a Cypher query and return the results."""
        driver = self.connect()
        with driver.session() as session:
            result = session.run(query, params or {})
            return [record.data() for record in result]
    
    def create_schema(self):
        """Create the schema for healthcare insurance data."""
        # Define constraints and indexes
        constraints = [
            "CREATE CONSTRAINT plan_id IF NOT EXISTS FOR (p:Plan) REQUIRE p.id IS UNIQUE",
            "CREATE CONSTRAINT policy_id IF NOT EXISTS FOR (p:Policy) REQUIRE p.id IS UNIQUE",
            "CREATE CONSTRAINT claim_id IF NOT EXISTS FOR (c:Claim) REQUIRE c.id IS UNIQUE",
            "CREATE CONSTRAINT benefit_id IF NOT EXISTS FOR (b:Benefit) REQUIRE b.id IS UNIQUE",
            "CREATE CONSTRAINT exclusion_id IF NOT EXISTS FOR (e:Exclusion) REQUIRE e.id IS UNIQUE"
        ]
        
        for constraint in constraints:
            self.run_query(constraint)
            
        return "Schema created successfully"

# Tool functions for LangChain
@tool
def query_insurance_plan(plan_name: str) -> str:
    """
    Query information about a specific insurance plan.
    
    Args:
        plan_name: The name of the insurance plan to query
        
    Returns:
        Information about the plan
    """
    connector = Neo4jConnector()
    query = """
    MATCH (p:Plan {name: $plan_name})
    OPTIONAL MATCH (p)-[:INCLUDES]->(b:Benefit)
    OPTIONAL MATCH (p)-[:EXCLUDES]->(e:Exclusion)
    RETURN p, collect(distinct b) as benefits, collect(distinct e) as exclusions
    """
    results = connector.run_query(query, {"plan_name": plan_name})
    connector.close()
    
    if not results:
        return f"No information found for plan: {plan_name}"
    
    # Format the results
    plan = results[0]["p"]
    benefits = results[0]["benefits"]
    exclusions = results[0]["exclusions"]
    
    response = f"Plan: {plan['name']}\n"
    response += f"Description: {plan.get('description', 'No description available')}\n\n"
    
    response += "Benefits:\n"
    if benefits:
        for benefit in benefits:
            response += f"- {benefit['name']}: {benefit.get('description', 'No description')}\n"
    else:
        response += "- No specific benefits listed\n"
    
    response += "\nExclusions:\n"
    if exclusions:
        for exclusion in exclusions:
            response += f"- {exclusion['name']}: {exclusion.get('description', 'No description')}\n"
    else:
        response += "- No specific exclusions listed\n"
    
    return response

@tool
def query_coverage_for_procedure(procedure: str) -> str:
    """
    Query whether a specific medical procedure is covered.
    
    Args:
        procedure: The medical procedure to check coverage for
        
    Returns:
        Information about coverage for the procedure
    """
    connector = Neo4jConnector()
    query = """
    MATCH (b:Benefit)-[:COVERS]->(p:Procedure {name: $procedure})
    OPTIONAL MATCH (plan:Plan)-[:INCLUDES]->(b)
    RETURN p, collect(distinct b) as benefits, collect(distinct plan) as plans
    """
    results = connector.run_query(query, {"procedure": procedure})
    
    # Also check exclusions
    exclusion_query = """
    MATCH (e:Exclusion)-[:EXCLUDES]->(p:Procedure {name: $procedure})
    OPTIONAL MATCH (plan:Plan)-[:EXCLUDES]->(e)
    RETURN p, collect(distinct e) as exclusions, collect(distinct plan) as plans
    """
    exclusion_results = connector.run_query(exclusion_query, {"procedure": procedure})
    connector.close()
    
    if not results and not exclusion_results:
        return f"No information found for procedure: {procedure}"
    
    response = f"Coverage information for: {procedure}\n\n"
    
    if results:
        procedure_info = results[0]["p"]
        benefits = results[0]["benefits"]
        plans = results[0]["plans"]
        
        response += "Covered under the following benefits:\n"
        for benefit in benefits:
            response += f"- {benefit['name']}: {benefit.get('description', 'No description')}\n"
        
        response += "\nIncluded in these plans:\n"
        for plan in plans:
            response += f"- {plan['name']}\n"
    
    if exclusion_results:
        exclusions = exclusion_results[0]["exclusions"]
        excluded_plans = exclusion_results[0]["plans"]
        
        response += "\nExcluded under the following conditions:\n"
        for exclusion in exclusions:
            response += f"- {exclusion['name']}: {exclusion.get('description', 'No description')}\n"
        
        response += "\nExcluded in these plans:\n"
        for plan in excluded_plans:
            response += f"- {plan['name']}\n"
    
    return response

@tool
def search_insurance_knowledge_graph(query: str) -> str:
    """
    Perform a general search on the insurance knowledge graph.
    
    Args:
        query: The search query
        
    Returns:
        Relevant information from the knowledge graph
    """
    # This is a simplified implementation
    # In a real system, we would use more sophisticated NLP to parse the query
    # and generate appropriate Cypher queries
    
    connector = Neo4jConnector()
    
    # Create a parameterized query that searches across multiple node types
    cypher_query = """
    MATCH (n)
    WHERE n.name CONTAINS $query OR n.description CONTAINS $query
    RETURN n, labels(n) as type
    LIMIT 5
    """
    
    results = connector.run_query(cypher_query, {"query": query})
    connector.close()
    
    if not results:
        return f"No results found for query: {query}"
    
    response = f"Search results for: {query}\n\n"
    
    for result in results:
        node = result["n"]
        node_type = result["type"][0]  # Get the first label
        
        response += f"{node_type}: {node['name']}\n"
        if "description" in node:
            response += f"Description: {node['description']}\n"
        response += "\n"
    
    return response