import os
from typing import List, Dict, Any, Optional
from pinecone import Pinecone, ServerlessSpec
from langchain_pinecone import PineconeVectorStore
from langchain_core.embeddings import Embeddings
from langchain_core.documents import Document
from langchain_community.embeddings import HuggingFaceEmbeddings
from langchain_core.tools import tool
from dotenv import load_dotenv

# Load environment variables
load_dotenv()

class PineconeConnector:
    """Connector for Pinecone vector database."""
    
    def __init__(self, embedding_model: Optional[Embeddings] = None):
        """Initialize Pinecone connection."""
        self.api_key = os.getenv("PINECONE_API_KEY", "your-api-key")
        self.environment = os.getenv("PINECONE_ENVIRONMENT", "gcp-starter")
        self.index_name = os.getenv("PINECONE_INDEX", "healthcare-insurance")
        
        # Use provided embedding model or initialize default
        self.embedding_model = embedding_model or HuggingFaceEmbeddings(
            model_name="sentence-transformers/all-MiniLM-L6-v2"
        )
        
        # Initialize Pinecone client
        self.pc = Pinecone(api_key=self.api_key)
        
    def create_index(self, dimension: int = 384):
        """Create a Pinecone index if it doesn't exist."""
        # Check if index exists
        if self.index_name not in self.pc.list_indexes().names():
            # Create a serverless index
            self.pc.create_index(
                name=self.index_name,
                dimension=dimension,
                metric="cosine",
                spec=ServerlessSpec(
                    cloud="aws",
                    region="us-west-2"
                )
            )
            return f"Created new index: {self.index_name}"
        return f"Index {self.index_name} already exists"
    
    def get_vector_store(self) -> PineconeVectorStore:
        """Get a LangChain vector store for the Pinecone index."""
        # Get the index
        index = self.pc.Index(self.index_name)
        
        # Create and return the vector store
        return PineconeVectorStore(
            index=index,
            embedding=self.embedding_model,
            text_key="text"
        )
    
    def add_documents(self, documents: List[Document]) -> str:
        """Add documents to the Pinecone index."""
        vector_store = self.get_vector_store()
        vector_store.add_documents(documents)
        return f"Added {len(documents)} documents to index {self.index_name}"
    
    def similarity_search(self, query: str, k: int = 4) -> List[Document]:
        """Perform a similarity search on the Pinecone index."""
        vector_store = self.get_vector_store()
        return vector_store.similarity_search(query, k=k)

# Tool functions for LangChain
@tool
def search_insurance_documents(query: str, k: int = 4) -> str:
    """
    Search for information in insurance documents using semantic search.
    
    Args:
        query: The search query
        k: Number of results to return (default: 4)
        
    Returns:
        Relevant information from insurance documents
    """
    connector = PineconeConnector()
    
    try:
        documents = connector.similarity_search(query, k=k)
        
        if not documents:
            return f"No relevant documents found for query: {query}"
        
        response = f"Search results for: {query}\n\n"
        
        for i, doc in enumerate(documents, 1):
            response += f"Result {i}:\n"
            response += f"{doc.page_content}\n"
            if doc.metadata:
                response += f"Source: {doc.metadata.get('source', 'Unknown')}\n"
                if 'page' in doc.metadata:
                    response += f"Page: {doc.metadata['page']}\n"
            response += "\n"
        
        return response
    except Exception as e:
        return f"Error searching documents: {str(e)}"

@tool
def hybrid_search(query: str, k: int = 4) -> str:
    """
    Perform a hybrid search combining graph and vector search results.
    
    Args:
        query: The search query
        k: Number of results to return from each source (default: 4)
        
    Returns:
        Combined results from graph and vector search
    """
    # Import here to avoid circular imports
    from backend.tools.neo4j_connector import search_insurance_knowledge_graph
    
    # Get results from both sources
    graph_results = search_insurance_knowledge_graph(query)
    vector_results = search_insurance_documents(query, k=k)
    
    # Combine the results
    response = f"Hybrid search results for: {query}\n\n"
    response += "Graph Database Results:\n"
    response += "-" * 40 + "\n"
    response += graph_results + "\n\n"
    
    response += "Document Search Results:\n"
    response += "-" * 40 + "\n"
    response += vector_results
    
    return response