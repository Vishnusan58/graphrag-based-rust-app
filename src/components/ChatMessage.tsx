import React from 'react';
import ReactMarkdown from 'react-markdown';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { atomDark } from 'react-syntax-highlighter/dist/cjs/styles/prism';
import { FaUser, FaRobot } from 'react-icons/fa';

type ChatMessageProps = {
  message: {
    role: 'user' | 'assistant';
    content: string;
  };
};

const ChatMessage: React.FC<ChatMessageProps> = ({ message }) => {
  const isUser = message.role === 'user';

  return (
    <div className={`flex w-full ${isUser ? 'justify-end' : 'justify-start'} mb-4`}>
      <div
        className={`flex max-w-[80%] ${
          isUser
            ? 'bg-blue-600 text-white rounded-tl-lg rounded-tr-lg rounded-bl-lg'
            : 'bg-gray-200 dark:bg-gray-700 text-black dark:text-white rounded-tl-lg rounded-tr-lg rounded-br-lg'
        } p-4 shadow-md`}
      >
        <div className="flex-shrink-0 mr-3 mt-1">
          {isUser ? (
            <FaUser className="h-5 w-5 text-white" />
          ) : (
            <FaRobot className="h-5 w-5 text-gray-400 dark:text-gray-300" />
          )}
        </div>
        <div className="flex-1 overflow-hidden">
          <ReactMarkdown
            components={{
              code({ node, inline, className, children, ...props }: { node?: any; inline?: boolean; className?: string; children?: React.ReactNode; [key: string]: any }) {
                const match = /language-(\w+)/.exec(className || '');
                return !inline && match ? (
                  <SyntaxHighlighter
                    style={atomDark}
                    language={match[1]}
                    PreTag="div"
                    {...props}
                  >
                    {String(children).replace(/\n$/, '')}
                  </SyntaxHighlighter>
                ) : (
                  <code className={className} {...props}>
                    {children}
                  </code>
                );
              },
              p: ({ children }: { children?: React.ReactNode }) => <p className="mb-2">{children}</p>,
              ul: ({ children }: { children?: React.ReactNode }) => <ul className="list-disc ml-4 mb-2">{children}</ul>,
              ol: ({ children }: { children?: React.ReactNode }) => <ol className="list-decimal ml-4 mb-2">{children}</ol>,
              li: ({ children }: { children?: React.ReactNode }) => <li className="mb-1">{children}</li>,
              h1: ({ children }: { children?: React.ReactNode }) => <h1 className="text-xl font-bold mb-2">{children}</h1>,
              h2: ({ children }: { children?: React.ReactNode }) => <h2 className="text-lg font-bold mb-2">{children}</h2>,
              h3: ({ children }: { children?: React.ReactNode }) => <h3 className="text-md font-bold mb-2">{children}</h3>,
              a: ({ href, children }: { href?: string; children?: React.ReactNode }) => (
                <a href={href} className="text-blue-400 hover:underline" target="_blank" rel="noopener noreferrer">
                  {children}
                </a>
              ),
            }}
          >
            {message.content}
          </ReactMarkdown>
        </div>
      </div>
    </div>
  );
};

export default ChatMessage;

