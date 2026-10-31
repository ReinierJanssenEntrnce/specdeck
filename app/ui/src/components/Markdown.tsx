import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';

interface MdNode {
  type: string;
  value?: string;
  children?: MdNode[];
}

/** Drops HTML comments, such as the notes in OpenSpec templates; other HTML stays visible as text. */
function remarkDropComments() {
  const drop = (node: MdNode) => {
    if (!node.children) return;
    node.children = node.children.filter((child) => !(child.type === 'html' && child.value?.trim().startsWith('<!--')));
    node.children.forEach(drop);
  };
  return drop;
}

/** Agent text, formatted. Links open in the system browser through the webview's default. */
export function Markdown({ text }: { text: string }) {
  return (
    <div className="markdown">
      <ReactMarkdown remarkPlugins={[remarkGfm, remarkDropComments]}>{text}</ReactMarkdown>
    </div>
  );
}
