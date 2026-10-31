import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';

/** Agent text, formatted. Links open in the system browser through the webview's default. */
export function Markdown({ text }: { text: string }) {
  return (
    <div className="markdown">
      <ReactMarkdown remarkPlugins={[remarkGfm]}>{text}</ReactMarkdown>
    </div>
  );
}
