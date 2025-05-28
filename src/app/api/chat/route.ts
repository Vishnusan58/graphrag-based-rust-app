import { NextRequest, NextResponse } from 'next/server';
import axios from 'axios';

// Define the backend API URL
const BACKEND_API_URL = process.env.BACKEND_API_URL || 'http://localhost:8000';

export async function POST(request: NextRequest) {
  try {
    // Parse the request body
    const body = await request.json();
    
    // Validate the request
    if (!body.messages || !Array.isArray(body.messages)) {
      return NextResponse.json(
        { error: 'Invalid request: messages array is required' },
        { status: 400 }
      );
    }
    
    // Forward the request to the backend API
    const response = await axios.post(`${BACKEND_API_URL}/chat`, {
      messages: body.messages,
      user_id: body.user_id || null
    });
    
    // Return the response from the backend
    return NextResponse.json(response.data);
  } catch (error) {
    console.error('Error in chat API route:', error);
    
    // Handle different types of errors
    if (axios.isAxiosError(error)) {
      const status = error.response?.status || 500;
      const message = error.response?.data?.detail || 'An error occurred while communicating with the backend';
      
      return NextResponse.json(
        { error: message },
        { status }
      );
    }
    
    // Generic error handling
    return NextResponse.json(
      { error: 'An unexpected error occurred' },
      { status: 500 }
    );
  }
}